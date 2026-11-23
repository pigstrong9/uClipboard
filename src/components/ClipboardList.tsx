import { convertFileSrc } from '@tauri-apps/api/core';
import { memo, useCallback, useEffect, useRef, useState } from 'react';

import { baseName, plural, relativeTime } from '../lib/format';
import { Icon } from '../lib/icons';
import * as ipc from '../lib/ipc';
import { useI18n } from '../lib/useI18n';
import type { ClipSummary } from '../lib/types';

interface Props {
  items: ClipSummary[];
  selectedId: string | null;
  selectedIds: string[];
  loading: boolean;
  onSelect: (id: string) => void;
  onCopy: (id: string) => void;
  onPaste: (id: string) => void;
  onFavorite: (id: string) => void;
  onPin: (id: string) => void;
  onDelete: (id: string) => void;
}

interface RowProps {
  item: ClipSummary;
  index: number;
  selected: boolean;
  picked: boolean;
  expanded: boolean;
  fullText?: string;
  onSelect: (id: string) => void;
  onCopy: (id: string) => void;
  onPaste: (id: string) => void;
  onFavorite: (id: string) => void;
  onPin: (id: string) => void;
  onDelete: (id: string) => void;
  onToggleExpand: (item: ClipSummary) => void;
  onMenu: (item: ClipSummary, x: number, y: number) => void;
}

const Row = memo(function Row({
  item,
  index,
  selected,
  picked,
  expanded,
  fullText,
  onSelect,
  onCopy,
  onPaste,
  onFavorite,
  onPin,
  onDelete,
  onToggleExpand,
  onMenu,
}: RowProps) {
  const { t } = useI18n();
  const meta =
    item.kind === 'text'
      ? plural(t, 'chars', item.size)
      : item.kind === 'image'
        ? `${item.width ?? 0} × ${item.height ?? 0}`
        : plural(t, 'filesCount', item.file_count ?? item.files.length);
  const text = expanded && fullText !== undefined ? fullText : item.text;
  const thumb = item.thumb_path ? convertFileSrc(item.thumb_path) : null;

  return (
    <article
      id={`clip-${item.id}`}
      role="option"
      aria-selected={picked}
      className={`item ${selected ? 'selected' : ''} ${picked ? 'picked' : ''}`}
      data-id={item.id}
      onClick={() => onSelect(item.id)}
      onDoubleClick={() => onPaste(item.id)}
      onContextMenu={(event) => {
        event.preventDefault();
        onMenu(item, event.clientX, event.clientY);
      }}
    >
      <div className="item-main">
        {item.kind === 'text' && (
          <>
            <div className={`text-preview ${expanded ? 'expanded' : ''}`}>{text}</div>
            {item.size > 180 && (
              <button
                type="button"
                className="expand-btn"
                onClick={(event) => {
                  event.stopPropagation();
                  onToggleExpand(item);
                }}
              >
                {expanded ? t('collapse') : t('expand')}
              </button>
            )}
          </>
        )}
        {item.kind === 'image' && (
          <div className="image-preview">
            {thumb ? (
              <img
                src={thumb}
                alt={t('previewImageAlt')}
                loading="lazy"
                decoding="async"
              />
            ) : (
              <div className="thumb-placeholder">{t('image')}</div>
            )}
          </div>
        )}
        {item.kind === 'files' &&
          (item.files.length > 0 ? (
            <ul className="file-list">
              {item.files.slice(0, 4).map((file, fileIndex) => (
                <li key={`${file}-${fileIndex}`}>
                  <span className="file-icon">
                    <Icon name="file" />
                  </span>
                  <span className="file-name">{baseName(file)}</span>
                </li>
              ))}
            </ul>
          ) : (
            <div className="text-preview">
              {plural(t, 'filesCount', item.file_count ?? 0)}
            </div>
          ))}
      </div>
      <div className="item-meta">
        <span className="item-time">{relativeTime(item.updated_at, t)}</span>
        {item.source_app && (
          <span className="item-source" title={t('sourceApp')}>
            {item.source_app}
          </span>
        )}
        <span className="item-size">{meta}</span>
        {item.copy_count > 1 && (
          <span className="item-copy-count" title={t('copyCount')}>
            {item.copy_count}×
          </span>
        )}
        {index < 9 && <span className="item-index">{index + 1}</span>}
      </div>
      <div className="row-actions">
        <button
          type="button"
          className="icon-btn"
          title={t('copy')}
          aria-label={t('copy')}
          onClick={(event) => {
            event.stopPropagation();
            onCopy(item.id);
          }}
        >
          <Icon name="copy" />
        </button>
        <button
          type="button"
          className={`icon-btn ${item.favorite ? 'active' : ''}`}
          title={t('favorite')}
          aria-label={item.favorite ? t('unfavorite') : t('favorite')}
          onClick={(event) => {
            event.stopPropagation();
            onFavorite(item.id);
          }}
        >
          <Icon name="star" />
        </button>
        <button
          type="button"
          className="icon-btn"
          title={t('delete')}
          aria-label={t('delete')}
          onClick={(event) => {
            event.stopPropagation();
            onDelete(item.id);
          }}
        >
          <Icon name="trash" />
        </button>
      </div>
      {item.pinned && (
        <span className="pin-badge">
          <Icon name="pin" />
        </span>
      )}
    </article>
  );
});

export function ClipboardList({
  items,
  selectedId,
  selectedIds,
  loading,
  onSelect,
  onCopy,
  onPaste,
  onFavorite,
  onPin,
  onDelete,
}: Props) {
  const [expanded, setExpanded] = useState<Set<string>>(() => new Set());
  const [contents, setContents] = useState<Record<string, string>>({});
  const [menu, setMenu] = useState<{ x: number; y: number; item: ClipSummary } | null>(null);
  const listRef = useRef<HTMLElement>(null);
  const expandedRef = useRef(expanded);
  const contentsRef = useRef(contents);
  const { t } = useI18n();

  const toggleExpanded = useCallback((item: ClipSummary) => {
    const next = new Set(expandedRef.current);
    if (next.has(item.id)) next.delete(item.id);
    else next.add(item.id);
    expandedRef.current = next;
    setExpanded(next);

    if (
      next.has(item.id) &&
      item.truncated &&
      contentsRef.current[item.id] === undefined
    ) {
      ipc
        .getItem(item.id)
        .then((full) => {
          contentsRef.current = { ...contentsRef.current, [item.id]: full.content };
          setContents(contentsRef.current);
        })
        .catch(() => undefined);
    }
  }, []);

  const openMenu = useCallback((item: ClipSummary, x: number, y: number) => {
    setMenu({
      x: Math.min(x, window.innerWidth - 170),
      y: Math.min(y, window.innerHeight - 210),
      item,
    });
  }, []);

  useEffect(() => {
    if (!selectedId) return;
    const nodes = listRef.current?.querySelectorAll<HTMLElement>('[data-id]');
    if (!nodes) return;
    for (const node of nodes) {
      if (node.dataset.id === selectedId) {
        node.scrollIntoView({ block: 'nearest' });
        break;
      }
    }
  }, [selectedId]);

  useEffect(() => {
    if (!menu) return;
    const close = () => setMenu(null);
    window.addEventListener('click', close);
    window.addEventListener('scroll', close, true);
    window.addEventListener('blur', close);
    return () => {
      window.removeEventListener('click', close);
      window.removeEventListener('scroll', close, true);
      window.removeEventListener('blur', close);
    };
  }, [menu]);

  if (loading && items.length === 0) {
    return (
      <section className="list" tabIndex={0} aria-label={t('historyLabel')}>
        <div className="empty">
          <div className="empty-icon">
            <Icon name="clipboard" />
          </div>
          <p>{t('loading')}</p>
        </div>
      </section>
    );
  }

  if (items.length === 0) {
    return (
      <section className="list" tabIndex={0} aria-label={t('historyLabel')}>
        <div className="empty">
          <div className="empty-icon">
            <Icon name="clipboard" />
          </div>
          <p>{t('emptyTitle')}</p>
          <span>{t('emptyHint')}</span>
        </div>
      </section>
    );
  }

  const picked = new Set(selectedIds);

  return (
    <section
      className="list"
      tabIndex={0}
      role="listbox"
      aria-multiselectable="true"
      aria-label={t('historyLabel')}
      aria-activedescendant={selectedId ? `clip-${selectedId}` : undefined}
      ref={listRef}
    >
      {items.map((item, index) => (
        <Row
          key={item.id}
          item={item}
          index={index}
          selected={selectedId === item.id}
          picked={picked.has(item.id)}
          expanded={expanded.has(item.id)}
          fullText={contents[item.id]}
          onSelect={onSelect}
          onCopy={onCopy}
          onPaste={onPaste}
          onFavorite={onFavorite}
          onPin={onPin}
          onDelete={onDelete}
          onToggleExpand={toggleExpanded}
          onMenu={openMenu}
        />
      ))}
      {menu && (
        <div
          className="context-menu"
          style={{ left: menu.x, top: menu.y }}
          onClick={(event) => event.stopPropagation()}
        >
          <button
            type="button"
            onClick={() => {
              setMenu(null);
              onPaste(menu.item.id);
            }}
          >
            {t('paste')}
          </button>
          <button
            type="button"
            onClick={() => {
              setMenu(null);
              onCopy(menu.item.id);
            }}
          >
            {t('copy')}
          </button>
          <hr />
          <button
            type="button"
            onClick={() => {
              setMenu(null);
              onFavorite(menu.item.id);
            }}
          >
            {menu.item.favorite ? t('unfavorite') : t('favorite')}
          </button>
          <button
            type="button"
            onClick={() => {
              setMenu(null);
              onPin(menu.item.id);
            }}
          >
            {menu.item.pinned ? t('unpin') : t('pin')}
          </button>
          <hr />
          <button
            type="button"
            className="danger"
            onClick={() => {
              setMenu(null);
              onDelete(menu.item.id);
            }}
          >
            {t('delete')}
          </button>
        </div>
      )}
    </section>
  );
}
