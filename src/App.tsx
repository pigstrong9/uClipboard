import { useEffect, useRef, useState } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';

import { ClipboardList } from './components/ClipboardList';
import { PreviewModal } from './components/PreviewModal';
import { SideRail } from './components/SideRail';
import { Tabs } from './components/Tabs';
import { Titlebar } from './components/Titlebar';
import { Toast } from './components/Toast';
import { onClipboardChanged } from './lib/ipc';
import type { ClipItem, TabKey } from './lib/types';
import { useClipboard } from './stores/clipboard';
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
  const openTransfer = useClipboard((state) => state.openTransfer);
  const toggleFavorite = useClipboard((state) => state.toggleFavorite);
  const togglePin = useClipboard((state) => state.togglePin);
  const remove = useClipboard((state) => state.remove);
  const copy = useClipboard((state) => state.copy);
  const paste = useClipboard((state) => state.paste);
  const clear = useClipboard((state) => state.clear);
  const clearError = useClipboard((state) => state.clearError);
  const toast = useUi((state) => state.toast);

  const [theme, setTheme] = useState<'dark' | 'light'>('dark');
  const [search, setSearch] = useState('');
  const [previewItem, setPreviewItem] = useState<ClipItem | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  const selected = items.find((item) => item.id === selectedId) ?? items[0] ?? null;

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
  }, [theme]);

  useEffect(() => {
    void load();
  }, [load]);

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    onClipboardChanged(() => {
      void load();
    })
      .then((fn) => {
        if (disposed) fn();
        else unlisten = fn;
      })
      .catch(() => undefined);
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [load]);

  useEffect(() => {
    if (!error) return;
    toast(error);
    clearError();
  }, [error, toast, clearError]);

  useEffect(() => {
    if (search === query) return;
    const timer = window.setTimeout(() => setQuery(search), 120);
    return () => window.clearTimeout(timer);
  }, [search, query, setQuery]);

  const run = async (action: () => Promise<unknown>, message?: string) => {
    try {
      await action();
      if (message) toast(message);
    } catch (cause) {
      toast(String(cause));
    }
  };

  const copyCurrent = () => {
    if (selected) void run(() => copy(selected.id), '已复制');
  };
  const favoriteCurrent = () => {
    if (selected) {
      void run(
        () => toggleFavorite(selected.id),
        selected.favorite ? '已取消收藏' : '已收藏',
      );
    }
  };
  const pinCurrent = () => {
    if (selected) {
      void run(() => togglePin(selected.id), selected.pinned ? '已取消置顶' : '已置顶');
    }
  };
  const deleteCurrent = () => {
    if (selected) void run(() => remove(selected.id), '已删除');
  };
  const clearAll = () => {
    if (window.confirm('确定清空全部剪贴板历史？')) void run(clear, '历史已清空');
  };
  const minimize = () => {
    void getCurrentWindow()
      .minimize()
      .catch(() => undefined);
  };

  const handleSelect = (id: string) => {
    select(id);
    toggleSelection(id);
  };

  const openTransferWindow = () => {
    const ids = selectedIds.length
      ? selectedIds
      : selected
        ? [selected.id]
        : [];
    if (!ids.length) {
      toast('请先选择内容');
      return;
    }
    void run(() => openTransfer(ids));
  };

  useEffect(() => {
    const handler = (event: KeyboardEvent) => {
      const target = event.target as HTMLElement | null;
      const inInput = target?.tagName === 'INPUT' || target?.tagName === 'TEXTAREA';
      const meta = event.metaKey || event.ctrlKey;

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

      const index = Math.max(
        0,
        items.findIndex((item) => item.id === selectedId),
      );
      const current = items[index] ?? null;

      if (event.key === 'ArrowDown') {
        event.preventDefault();
        const next = items[Math.min(index + 1, items.length - 1)];
        if (next) select(next.id);
        return;
      }
      if (event.key === 'ArrowUp') {
        event.preventDefault();
        const prev = items[Math.max(index - 1, 0)];
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
        if (current) void run(() => paste(current.id));
        return;
      }
      if (meta && event.key.toLowerCase() === 'c') {
        event.preventDefault();
        if (current) void run(() => copy(current.id), '已复制');
        return;
      }
      if (meta && event.key.toLowerCase() === 'd') {
        event.preventDefault();
        if (current) {
          void run(
            () => toggleFavorite(current.id),
            current.favorite ? '已取消收藏' : '已收藏',
          );
        }
        return;
      }
      if (meta && event.key.toLowerCase() === 'p') {
        event.preventDefault();
        if (current) {
          void run(
            () => togglePin(current.id),
            current.pinned ? '已取消置顶' : '已置顶',
          );
        }
        return;
      }
      if (event.key === 'Delete' || (event.metaKey && event.key === 'Backspace')) {
        event.preventDefault();
        if (current) void run(() => remove(current.id), '已删除');
        return;
      }
      if (event.key === ' ') {
        event.preventDefault();
        if (current) setPreviewItem(current);
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
    tab,
    copy,
    paste,
    toggleFavorite,
    togglePin,
    remove,
    select,
    setQuery,
    setTab,
    toast,
  ]);

  return (
    <main className="app">
      <Titlebar
        search={search}
        inputRef={inputRef}
        onSearch={setSearch}
        theme={theme}
        onToggleTheme={() => setTheme((value) => (value === 'dark' ? 'light' : 'dark'))}
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
          onCopy={(id) => void run(() => copy(id), '已复制')}
          onPaste={(id) => void run(() => paste(id))}
          onFavorite={(id) => void run(() => toggleFavorite(id))}
          onDelete={(id) => void run(() => remove(id), '已删除')}
        />
        <SideRail
          hasSelection={Boolean(selected)}
          selectionCount={selectedIds.length}
          onCopy={copyCurrent}
          onFavorite={favoriteCurrent}
          onTransfer={openTransferWindow}
          onSettings={() => toast('设置将在 M3 提供')}
          onClear={clearAll}
        />
        <div className="status-float">监控中 · 共 {totalCount} 条</div>
      </div>
      {previewItem && (
        <PreviewModal item={previewItem} onClose={() => setPreviewItem(null)} />
      )}
      <Toast />
    </main>
  );
}
