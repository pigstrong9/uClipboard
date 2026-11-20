import { useEffect, useMemo, useRef, useState } from 'react';

import { Toast } from './components/Toast';
import { baseName } from './lib/format';
import { Icon } from './lib/icons';
import * as ipc from './lib/ipc';
import { useI18n } from './lib/useI18n';
import { useTheme } from './lib/useTheme';
import { parseExtra, parseFiles, type ClipItem } from './lib/types';
import { useSettings } from './stores/settings';
import { useUi } from './stores/ui';

function TransferCard({
  item,
  index,
  expanded,
  onToggleExpand,
  onPaste,
}: {
  item: ClipItem;
  index: number;
  expanded: boolean;
  onToggleExpand: (id: string) => void;
  onPaste: (id: string) => void;
}) {
  const { t } = useI18n();
  const extra = parseExtra(item.extra);
  const files = item.kind === 'files' ? parseFiles(item.content) : [];
  const meta =
    item.kind === 'text'
      ? t('chars', { n: item.size })
      : item.kind === 'image'
        ? `${extra.width ?? 0} × ${extra.height ?? 0}`
        : t('filesCount', { n: extra.count ?? files.length });

  return (
    <article
      className={`item transfer-card ${index === 0 ? 'active' : ''}`}
      title={t('doubleClickPaste')}
      onDoubleClick={() => onPaste(item.id)}
    >
      <div className="item-main">
        {item.kind === 'text' && (
          <>
            <div className={`text-preview ${expanded ? 'expanded' : ''}`}>{item.content}</div>
            {item.content.length > 180 && (
              <button
                type="button"
                className="expand-btn"
                onClick={() => onToggleExpand(item.id)}
              >
                {expanded ? t('collapse') : t('expand')}
              </button>
            )}
          </>
        )}
        {item.kind === 'image' && (
          <div className="image-preview">
            {item.thumb_data_url ? (
              <img src={item.thumb_data_url} alt={t('previewImageAlt')} />
            ) : (
              <div className="thumb-placeholder">{t('image')}</div>
            )}
          </div>
        )}
        {item.kind === 'files' && (
          <ul className="file-list">
            {(files.length ? files : [item.preview]).slice(0, 4).map((file, fileIndex) => (
              <li key={`${file}-${fileIndex}`}>
                <span className="file-icon">
                  <Icon name="file" />
                </span>
                <span className="file-name">{baseName(file)}</span>
              </li>
            ))}
          </ul>
        )}
      </div>
      <div className="item-meta">
        <span className="transfer-source">{t('transferSource')}</span>
        <span className="item-size">{meta}</span>
        <span className="item-index">{index + 1}</span>
      </div>
    </article>
  );
}

export default function TransferApp() {
  const [items, setItems] = useState<ClipItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchOpen, setSearchOpen] = useState(false);
  const [search, setSearch] = useState('');
  const [expanded, setExpanded] = useState<Set<string>>(new Set());
  const [copying, setCopying] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const toast = useUi((state) => state.toast);
  const initSettings = useSettings((state) => state.init);
  const { t } = useI18n();
  useTheme();

  const load = async () => {
    try {
      const data = await ipc.transferItems();
      setItems(data);
    } catch (error) {
      toast(String(error));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void initSettings();
    void load();
    let unlisten: (() => void) | undefined;
    ipc
      .onTransferChanged(() => {
        void load();
      })
      .then((fn) => {
        unlisten = fn;
      })
      .catch(() => undefined);
    return () => {
      unlisten?.();
    };
  }, [initSettings]);

  useEffect(() => {
    if (searchOpen) inputRef.current?.focus();
  }, [searchOpen]);

  const filtered = useMemo(() => {
    const keyword = search.trim().toLowerCase();
    if (!keyword) return items;
    return items.filter(
      (item) =>
        item.content.toLowerCase().includes(keyword) ||
        item.preview.toLowerCase().includes(keyword),
    );
  }, [items, search]);

  const copyAll = async () => {
    if (!items.length || copying) return;
    setCopying(true);
    try {
      const count = await ipc.copyTransfer();
      toast(t('copiedCount', { n: count }));
    } catch (error) {
      toast(String(error));
    } finally {
      setCopying(false);
    }
  };

  const favoriteAll = async () => {
    if (!items.length) return;
    try {
      const favorited = await ipc.favoriteTransfer();
      toast(favorited ? t('favoriteAdded') : t('favoriteRemoved'));
      await load();
    } catch (error) {
      toast(String(error));
    }
  };

  const closeWindow = async () => {
    try {
      await ipc.closeTransfer();
    } catch (error) {
      toast(String(error));
    }
  };

  const toggleExpand = (id: string) => {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const pasteItem = async (id: string) => {
    try {
      await ipc.pasteTransferItem(id);
      toast(t('pasted'));
    } catch (error) {
      toast(String(error));
    }
  };

  return (
    <div className="transfer-window">
      <div className="transfer-head" data-tauri-drag-region>
        <span className="transfer-pin" title={t('transferTitle')}>
          <Icon name="pin" />
        </span>
        <button
          type="button"
          className="icon-btn"
          title={copying ? t('copying') : t('copyAll')}
          onClick={copyAll}
          disabled={!items.length || copying}
        >
          <Icon name="copy" />
        </button>
        <button
          type="button"
          className="icon-btn"
          title={t('favoriteAll')}
          onClick={favoriteAll}
          disabled={!items.length}
        >
          <Icon name="star" />
        </button>
        <span className="transfer-spacer" />
        <button
          type="button"
          className="icon-btn"
          title={t('search')}
          onClick={() => setSearchOpen((value) => !value)}
        >
          <Icon name="search" />
        </button>
        <button type="button" className="icon-btn" title={t('close')} onClick={closeWindow}>
          <Icon name="close" />
        </button>
      </div>

      {copying && <div className="transfer-progress">{t('copying')}</div>}

      {searchOpen && (
        <div className="transfer-search">
          <input
            ref={inputRef}
            value={search}
            onChange={(event) => setSearch(event.target.value)}
            placeholder={t('searchPlaceholderTransfer')}
          />
        </div>
      )}

      <div className="transfer-body">
        {loading && items.length === 0 ? (
          <div className="transfer-empty">{t('loading')}</div>
        ) : filtered.length ? (
          filtered.map((item, index) => (
            <TransferCard
              key={item.id}
              item={item}
              index={index}
              expanded={expanded.has(item.id)}
              onToggleExpand={toggleExpand}
              onPaste={pasteItem}
            />
          ))
        ) : (
          <div className="transfer-empty">
            {items.length ? t('noMatch') : t('transferEmpty')}
          </div>
        )}
      </div>
      <Toast />
    </div>
  );
}
