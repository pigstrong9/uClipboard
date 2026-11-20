import { useState } from 'react';

import { baseName, relativeTime } from '../lib/format';
import { Icon } from '../lib/icons';
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
  onDelete,
}: Props) {
  const [expanded, setExpanded] = useState<Set<string>>(new Set());

  if (loading && items.length === 0) {
    return (
      <section className="list" tabIndex={0} aria-label="剪贴板历史">
        <div className="empty">
          <div className="empty-icon">
            <Icon name="clipboard" />
          </div>
          <p>加载中…</p>
        </div>
      </section>
    );
  }

  if (items.length === 0) {
    return (
      <section className="list" tabIndex={0} aria-label="剪贴板历史">
        <div className="empty">
          <div className="empty-icon">
            <Icon name="clipboard" />
          </div>
          <p>还没有剪贴板记录</p>
          <span>复制任意内容后会自动出现在这里</span>
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

  return (
    <section className="list" tabIndex={0} aria-label="剪贴板历史">
      {items.map((item, index) => {
        const extra = parseExtra(item.extra);
        const files = item.kind === 'files' ? parseFiles(item.content) : [];
        const meta =
          item.kind === 'text'
            ? `${item.size} 字符`
            : item.kind === 'image'
              ? `${extra.width ?? 0} × ${extra.height ?? 0}`
              : `${extra.count ?? files.length} 个文件`;
        const isExpanded = expanded.has(item.id);

        return (
          <article
            key={item.id}
            className={`item ${selectedId === item.id ? 'selected' : ''} ${
              selectedIds.includes(item.id) ? 'picked' : ''
            }`}
            data-id={item.id}
            onClick={() => onSelect(item.id)}
            onDoubleClick={() => onPaste(item.id)}
          >
            <div className="item-main">
              {item.kind === 'text' && (
                <>
                  <div className={`text-preview ${isExpanded ? 'expanded' : ''}`}>
                    {item.content}
                  </div>
                  {item.content.length > 180 && (
                    <button
                      type="button"
                      className="expand-btn"
                      onClick={(event) => {
                        event.stopPropagation();
                        toggleExpanded(item.id);
                      }}
                    >
                      {isExpanded ? '收起' : '展开'}
                    </button>
                  )}
                </>
              )}
              {item.kind === 'image' && (
                <div className="image-preview">
                  {item.thumb_data_url ? (
                    <img src={item.thumb_data_url} alt="剪贴板图片" />
                  ) : (
                    <div className="thumb-placeholder">图片</div>
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
              <span className="item-time">{relativeTime(item.updated_at)}</span>
              <span className="item-size">{meta}</span>
              {index < 9 && <span className="item-index">{index + 1}</span>}
            </div>
            <div className="row-actions">
              <button
                type="button"
                className="icon-btn"
                title="复制"
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
                title="收藏"
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
                title="删除"
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
    </section>
  );
}
