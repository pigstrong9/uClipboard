import { useCallback, useEffect, useRef, useState } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';

import { ClipboardList } from './components/ClipboardList';
import { ConfirmDialog } from './components/ConfirmDialog';
import { PreviewModal } from './components/PreviewModal';
import { SettingsView } from './components/SettingsView';
import { SideRail } from './components/SideRail';
import { Tabs } from './components/Tabs';
import { Titlebar } from './components/Titlebar';
import { Toast } from './components/Toast';
import { errorText } from './lib/errors';
import { plural } from './lib/format';
import * as ipc from './lib/ipc';
import { useI18n } from './lib/useI18n';
import { useTheme } from './lib/useTheme';
import type { ClipItem, TabKey } from './lib/types';
import { useClipboard } from './stores/clipboard';
import { useSettings } from './stores/settings';
import { useUi } from './stores/ui';

const TAB_ORDER: TabKey[] = ['all', 'text', 'image', 'files', 'favorite'];

export default function App() {
  const items = useClipboard((state) => state.items);
  const tab = useClipboard((state) => state.tab);
  const query = useClipboard((state) => state.query);
  const loading = useClipboard((state) => state.loading);
  const error = useClipboard((state) => state.error);
  const selectedId = useClipboard((state) => state.selectedId);
  const selectedIds = useClipboard((state) => state.selectedIds);
  const favoriteCount = useClipboard((state) => state.favoriteCount);
  const totalCount = useClipboard((state) => state.totalCount);
  const load = useClipboard((state) => state.load);
  const setTab = useClipboard((state) => state.setTab);
  const setQuery = useClipboard((state) => state.setQuery);
  const select = useClipboard((state) => state.select);
  const toggleSelection = useClipboard((state) => state.toggleSelection);
  const clearSelection = useClipboard((state) => state.clearSelection);
  const openTransfer = useClipboard((state) => state.openTransfer);
  const toggleFavorite = useClipboard((state) => state.toggleFavorite);
  const togglePin = useClipboard((state) => state.togglePin);
  const remove = useClipboard((state) => state.remove);
  const copy = useClipboard((state) => state.copy);
  const paste = useClipboard((state) => state.paste);
  const clear = useClipboard((state) => state.clear);
  const clearError = useClipboard((state) => state.clearError);
  const toast = useUi((state) => state.toast);
  const confirm = useUi((state) => state.confirm);
  const ask = useUi((state) => state.ask);
  const answer = useUi((state) => state.answer);
  const settings = useSettings((state) => state.settings);
  const initSettings = useSettings((state) => state.init);
  const settingsError = useSettings((state) => state.error);
  const clearSettingsError = useSettings((state) => state.clearError);
  const { theme, toggleTheme } = useTheme();
  const { t } = useI18n();

  const [search, setSearch] = useState('');
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [previewItem, setPreviewItem] = useState<ClipItem | null>(null);
  const [monitorError, setMonitorError] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  const selected = items.find((item) => item.id === selectedId) ?? null;
  const paused = settings?.monitorPaused ?? false;

  useEffect(() => {
    void initSettings();
  }, [initSettings]);

  useEffect(() => {
    void load();
  }, [load]);

  useEffect(() => {
    const disposers: Array<() => void> = [];
    let disposed = false;

    const bind = (promise: Promise<() => void>) => {
      promise
        .then((unlisten) => {
          if (disposed) unlisten();
          else disposers.push(unlisten);
        })
        .catch(() => undefined);
    };

    bind(ipc.onClipboardChanged(() => void load()));
    bind(
      ipc.onMainShown(() => {
        setSearch('');
        setSettingsOpen(false);
        setPreviewItem(null);
        clearSelection();
        select(null);
        setQuery('');
        inputRef.current?.focus();
      }),
    );
    bind(ipc.onOpenSettings(() => setSettingsOpen(true)));
    bind(ipc.onToastMessage((message) => toast(message)));
    bind(ipc.onToastKey((key) => toast(t(key))));
    bind(ipc.onUpdateAvailable((version) => toast(t('updateAvailable', { version }))));
    bind(ipc.onMonitorError((message) => setMonitorError(message)));
    bind(ipc.onMonitorOk(() => setMonitorError(null)));
    bind(ipc.onScreenshotError((code) => toast(errorText(t, code))));

    return () => {
      disposed = true;
      disposers.forEach((dispose) => dispose());
    };
  }, [load, toast, t, clearSelection, setQuery]);

  useEffect(() => {
    if (!error) return;
    toast(errorText(t, error));
    clearError();
  }, [error, toast, t, clearError]);

  useEffect(() => {
    if (!settingsError) return;
    toast(errorText(t, settingsError));
    clearSettingsError();
  }, [settingsError, toast, t, clearSettingsError]);

  const askClear = useCallback(() => {
    ask({
      message: t('clearConfirm'),
      onConfirm: () => {
        void clear()
          .then(() => toast(t('historyCleared')))
          .catch((cause) => toast(errorText(t, cause)));
      },
    });
  }, [ask, clear, t, toast]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let disposed = false;
    ipc
      .onRequestClear(askClear)
      .then((fn) => {
        if (disposed) fn();
        else unlisten = fn;
      })
      .catch(() => undefined);
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [askClear]);

  useEffect(() => {
    if (search === query) return;
    const timer = window.setTimeout(() => setQuery(search), 120);
    return () => window.clearTimeout(timer);
  }, [search, query, setQuery]);

  const run = useCallback(
    async (action: () => Promise<unknown>, message?: string) => {
      try {
        await action();
        if (message) toast(message);
      } catch (cause) {
        toast(errorText(t, cause));
      }
    },
    [toast, t],
  );

  const copyCurrent = () => {
    if (selected) void run(() => copy(selected.id), t('copied'));
  };
  const favoriteCurrent = () => {
    if (selected) {
      void run(
        () => toggleFavorite(selected.id),
        selected.favorite ? t('favoriteRemoved') : t('favoriteAdded'),
      );
    }
  };
  const pinCurrent = () => {
    if (selected) {
      void run(() => togglePin(selected.id), selected.pinned ? t('unpinned') : t('pinned'));
    }
  };
  const deleteCurrent = () => {
    if (selected) void run(() => remove(selected.id), t('deleted'));
  };
  const minimize = () => {
    void getCurrentWindow()
      .minimize()
      .catch(() => undefined);
  };

  const handleSelect = useCallback(
    (id: string) => {
      select(id);
      toggleSelection(id);
    },
    [select, toggleSelection],
  );
  const handleCopy = useCallback(
    (id: string) => {
      void run(() => copy(id), t('copied'));
    },
    [run, copy, t],
  );
  const handlePaste = useCallback(
    (id: string) => {
      void run(() => paste(id));
    },
    [run, paste],
  );
  const handleFavorite = useCallback(
    (id: string) => {
      void run(() => toggleFavorite(id));
    },
    [run, toggleFavorite],
  );
  const handlePin = useCallback(
    (id: string) => {
      void run(() => togglePin(id));
    },
    [run, togglePin],
  );
  const handleDelete = useCallback(
    (id: string) => {
      void run(() => remove(id), t('deleted'));
    },
    [run, remove, t],
  );

  const openTransferWindow = () => {
    const ids = selectedIds.length
      ? selectedIds
      : selected
        ? [selected.id]
        : [];
    if (!ids.length) {
      toast(t('selectFirst'));
      return;
    }
    void run(() => openTransfer(ids));
  };

  useEffect(() => {
    const handler = (event: KeyboardEvent) => {
      const target = event.target as HTMLElement | null;
      const inInput = target?.tagName === 'INPUT' || target?.tagName === 'TEXTAREA';
      const meta = event.metaKey || event.ctrlKey;

      if (confirm) {
        if (event.key === 'Escape') {
          event.preventDefault();
          answer(false);
        }
        return;
      }

      if (settingsOpen) {
        if (event.key === 'Escape') {
          event.preventDefault();
          setSettingsOpen(false);
        }
        return;
      }

      if (previewItem) {
        if (event.key === 'Escape') {
          event.preventDefault();
          setPreviewItem(null);
        }
        return;
      }

      if (event.key === 'Escape') {
        if (inInput) {
          if (search) {
            setSearch('');
            setQuery('');
          } else {
            target?.blur();
          }
        } else if (search) {
          setSearch('');
          setQuery('');
        } else {
          void ipc.hideMain();
        }
        return;
      }

      if (inInput) {
        if (event.key === 'ArrowDown') {
          event.preventDefault();
          target?.blur();
          if (items[0]) select(items[0].id);
        }
        return;
      }

      const found = items.findIndex((item) => item.id === selectedId);
      const index = Math.max(0, found);
      const current = items[index] ?? null;

      if (event.key === 'ArrowDown') {
        event.preventDefault();
        const next = items[found < 0 ? 0 : Math.min(index + 1, items.length - 1)];
        if (next) select(next.id);
        return;
      }
      if (event.key === 'ArrowUp') {
        event.preventDefault();
        const prev =
          found < 0 ? items[items.length - 1] : items[Math.max(index - 1, 0)];
        if (prev) select(prev.id);
        return;
      }
      if (event.key === 'ArrowLeft' || event.key === 'ArrowRight') {
        event.preventDefault();
        const currentTab = TAB_ORDER.indexOf(tab);
        const offset = event.key === 'ArrowRight' ? 1 : -1;
        const nextTab = TAB_ORDER[(currentTab + offset + TAB_ORDER.length) % TAB_ORDER.length];
        setTab(nextTab);
        return;
      }
      if (event.key === 'Enter') {
        event.preventDefault();
        if (event.repeat) return;
        if (current) void run(() => paste(current.id));
        return;
      }
      if (meta && event.key.toLowerCase() === 'c') {
        event.preventDefault();
        if (event.repeat) return;
        if (current) void run(() => copy(current.id), t('copied'));
        return;
      }
      if (meta && event.key.toLowerCase() === 'd') {
        event.preventDefault();
        if (event.repeat) return;
        if (current) {
          void run(
            () => toggleFavorite(current.id),
            current.favorite ? t('favoriteRemoved') : t('favoriteAdded'),
          );
        }
        return;
      }
      if (meta && event.key.toLowerCase() === 'p') {
        event.preventDefault();
        if (event.repeat) return;
        if (current) {
          void run(
            () => togglePin(current.id),
            current.pinned ? t('unpinned') : t('pinned'),
          );
        }
        return;
      }
      if (event.key === 'Delete' || (event.metaKey && event.key === 'Backspace')) {
        event.preventDefault();
        if (event.repeat) return;
        if (current) void run(() => remove(current.id), t('deleted'));
        return;
      }
      if (event.key === ' ') {
        event.preventDefault();
        if (event.repeat) return;
        if (current) {
          void run(async () => {
            setPreviewItem(await ipc.getItem(current.id));
          });
        }
        return;
      }
      if (/^[1-9]$/.test(event.key)) {
        if (event.repeat) return;
        const targetItem = items[Number(event.key) - 1];
        if (targetItem) {
          event.preventDefault();
          void run(() => paste(targetItem.id));
        }
        return;
      }
      if (event.key === '/') {
        event.preventDefault();
        inputRef.current?.focus();
        return;
      }
      if (!meta && !event.altKey && /^[a-zA-Z]$/.test(event.key)) {
        event.preventDefault();
        inputRef.current?.focus();
        setSearch((value) => value + event.key);
      }
    };

    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [
    items,
    selectedId,
    search,
    previewItem,
    settingsOpen,
    confirm,
    answer,
    tab,
    copy,
    paste,
    toggleFavorite,
    togglePin,
    remove,
    select,
    setQuery,
    setTab,
    run,
    toast,
    t,
  ]);

  return (
    <main className="app">
      <Titlebar
        search={search}
        inputRef={inputRef}
        onSearch={setSearch}
        theme={theme}
        onToggleTheme={toggleTheme}
        onMinimize={minimize}
      />
      <Tabs tab={tab} favoriteCount={favoriteCount} onChange={setTab} />
      <div className="workspace">
        <ClipboardList
          items={items}
          selectedId={selectedId}
          selectedIds={selectedIds}
          loading={loading}
          onSelect={handleSelect}
          onCopy={handleCopy}
          onPaste={handlePaste}
          onFavorite={handleFavorite}
          onPin={handlePin}
          onDelete={handleDelete}
        />
        <SideRail
          hasSelection={Boolean(selected)}
          selectionCount={selectedIds.length}
          onCopy={copyCurrent}
          onFavorite={favoriteCurrent}
          onTransfer={openTransferWindow}
          onSettings={() => setSettingsOpen(true)}
          onClear={askClear}
        />
        <div
          className={`status-float ${monitorError ? 'error' : paused ? 'paused' : ''}`}
          title={monitorError ? errorText(t, monitorError) : undefined}
          role="status"
          aria-live="polite"
        >
          {monitorError ? t('monitorError') : paused ? t('paused') : t('monitoring')} ·{' '}
          {plural(t, 'totalCount', totalCount)}
        </div>
      </div>
      {settingsOpen && <SettingsView onClose={() => setSettingsOpen(false)} />}
      {previewItem && (
        <PreviewModal item={previewItem} onClose={() => setPreviewItem(null)} />
      )}
      {confirm && (
        <ConfirmDialog
          message={confirm.message}
          onCancel={() => answer(false)}
          onConfirm={() => answer(true)}
        />
      )}
      <Toast />
    </main>
  );
}
