import { Icon } from '../lib/icons';

interface Props {
  hasSelection: boolean;
  selectionCount: number;
  onCopy: () => void;
  onFavorite: () => void;
  onTransfer: () => void;
  onSettings: () => void;
  onClear: () => void;
}

export function SideRail({
  hasSelection,
  selectionCount,
  onCopy,
  onFavorite,
  onTransfer,
  onSettings,
  onClear,
}: Props) {
  return (
    <aside className="side-rail" aria-label="快捷操作">
      <div className="quick-rail">
        <button
          type="button"
          className="quick-btn"
          title="复制"
          onClick={onCopy}
          disabled={!hasSelection}
        >
          <Icon name="copy" />
        </button>
        <button
          type="button"
          className="quick-btn"
          title="收藏"
          onClick={onFavorite}
          disabled={!hasSelection}
        >
          <Icon name="star" />
        </button>
        <button
          type="button"
          className="quick-btn"
          title="多选 / 临时中转"
          data-count={selectionCount > 0 ? selectionCount : undefined}
          onClick={onTransfer}
        >
          <Icon name="pin" />
        </button>
      </div>
      <div className="utility-rail">
        <button type="button" className="icon-btn rail-btn" title="设置（M3）" onClick={onSettings}>
          <Icon name="gear" />
        </button>
        <button type="button" className="icon-btn rail-btn" title="清空历史" onClick={onClear}>
          <Icon name="trash" />
        </button>
      </div>
    </aside>
  );
}
