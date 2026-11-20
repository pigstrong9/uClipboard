import { useEffect, useMemo, useRef, useState } from 'react';

import { Toast } from './components/Toast';
import { baseName } from './lib/format';
import { Icon } from './lib/icons';
import * as ipc from './lib/ipc';
import { parseExtra, parseFiles, type ClipItem } from './lib/types';
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
  const extra = parseExtra(item.extra);
  const files = item.kind === 'files' ? parseFiles(item.content) : [];
  const meta =
    item.kind === 'text'
      ? `${item.size} 字符`
      : item.kind === 'image'
        ? `${extra.width ?? 0} × ${extra.height ?? 0}`
        : `${extra.count ?? files.length} 个文件`;

  return (
    <article
      className={`item transfer-card ${index === 0 ? 'active' : ''}`}
      title="双击粘贴"
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
                {expanded ? '收起' : '展开'}
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
        <span className="transfer-source">临时中转</span>
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
  const inputRef = useRef<HTMLInputElement>(null);
  const toast = useUi((state) => state.toast);

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
  }, []);

  useEffect(() => {
    if (searchOpen) inputRef.current?.focus();
  }, [searchOpen]);

  const filtered = useMemo(() => {
    const keyword = search.trim().toLowerCase();
    if (!keyword) return items;
    return items.filter((item) => item.content.toLowerCase().includes(keyword));
  }, [items, search]);

  const copyAll = async () => {
    if (!items.length) return;
    try {
      const count = await ipc.copyTransfer();
      toast(`已复制 ${count} 条`);
    } catch (error) {
      toast(String(error));
    }
  };

  const favoriteAll = async () => {
    if (!items.length) return;
    try {
      const favorited = await ipc.favoriteTransfer();
      toast(favorited ? '已收藏' : '已取消收藏');
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
      toast('已粘贴');
    } catch (error) {
      toast(String(error));
    }
  };

  return (
    <div className="transfer-window">
      <div className="transfer-head" data-tauri-drag-region>
        <span className="transfer-pin" title="置顶窗口">
          <Icon name="pin" />
        </span>
        <button
          type="button"
          className="icon-btn"
          title="复制全部"
          onClick={copyAll}
          disabled={!items.length}
        >
          <Icon name="copy" />
        </button>
        <button
          type="button"
          className="icon-btn"
          title="收藏全部"
          onClick={favoriteAll}
          disabled={!items.length}
        >
          <Icon name="star" />
        </button>
        <span className="transfer-spacer" />
        <button
          type="button"
          className="icon-btn"
          title="搜索"
          onClick={() => setSearchOpen((value) => !value)}
        >
          <Icon name="search" />
        </button>
        <button type="button" className="icon-btn" title="关闭" onClick={closeWindow}>
          <Icon name="close" />
        </button>
      </div>

      {searchOpen && (
        <div className="transfer-search">
          <input
            ref={inputRef}
            value={search}
            onChange={(event) => setSearch(event.target.value)}
            placeholder="在中转内容中搜索…"
          />
        </div>
      )}

      <div className="transfer-body">
        {loading && items.length === 0 ? (
          <div className="transfer-empty">加载中…</div>
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
            {items.length
              ? '没有匹配的内容'
              : '先在主窗口选择条目，再点击中转按钮'}
          </div>
        )}
      </div>
      <Toast />
    </div>
  );
}
