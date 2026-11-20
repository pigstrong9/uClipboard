import { useEffect, useState } from 'react';

import { baseName, relativeTime } from '../lib/format';
import { Icon } from '../lib/icons';
import { useI18n } from '../lib/useI18n';
import { parseExtra, parseFiles, type ClipItem } from '../lib/types';

interface Props {
  items: ClipItem[];
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
  const [expanded, setExpanded] = useState<Set<string>>(new Set());
  const [menu, setMenu] = useState<{ x: number; y: number; item: ClipItem } | null>(null);
  const { t } = useI18n();

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

  const toggleExpanded = (id: string) => {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const picked = new Set(selectedIds);

  return (
    <section className="list" tabIndex={0} aria-label={t('historyLabel')}>
      {items.map((item, index) => {
        const extra = parseExtra(item.extra);
        const files = item.kind === 'files' ? parseFiles(item.content) : [];
        const meta =
          item.kind === 'text'
            ? t('chars', { n: item.size })
            : item.kind === 'image'
              ? `${extra.width ?? 0} × ${extra.height ?? 0}`
              : t('filesCount', { n: extra.count ?? files.length });
        const isExpanded = expanded.has(item.id);

        return (
          <article
            key={item.id}
            className={`item ${selectedId === item.id ? 'selected' : ''} ${
              picked.has(item.id) ? 'picked' : ''
            }`}
            data-id={item.id}
            onClick={() => onSelect(item.id)}
            onDoubleClick={() => onPaste(item.id)}
            onContextMenu={(event) => {
              event.preventDefault();
              setMenu({
                x: Math.min(event.clientX, window.innerWidth - 170),
                y: Math.min(event.clientY, window.innerHeight - 210),
                item,
              });
            }}
          >
            <div className="item-main">
              {item.kind === 'text' && (
                <>
                  <div className={`text-preview ${isExpanded ? 'expanded' : ''}`}>
                    {item.content}
                  </div>
                  {item.size > 180 && (
                    <button
                      type="button"
                      className="expand-btn"
                      onClick={(event) => {
                        event.stopPropagation();
                        toggleExpanded(item.id);
                      }}
                    >
                      {isExpanded ? t('collapse') : t('expand')}
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
              <span className="item-time">{relativeTime(item.updated_at, t)}</span>
              {item.source_app && (
                <span className="item-source" title={t('sourceApp')}>
                  {item.source_app}
                </span>
              )}
              <span className="item-size">{meta}</span>
              {index < 9 && <span className="item-index">{index + 1}</span>}
            </div>
            <div className="row-actions">
              <button
                type="button"
                className="icon-btn"
                title={t('copy')}
                onClick={(event) => {
                  event.stopPropagation();
                  onCopy(item.id);
                }}
              >
                <Icon name="copy" />
              </button>
              <button
                type="button"
                className="icon-btn"
                title={t('favorite')}
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
      })}
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
