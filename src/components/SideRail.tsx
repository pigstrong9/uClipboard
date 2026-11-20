import { Icon } from '../lib/icons';
import { useI18n } from '../lib/useI18n';

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
  const { t } = useI18n();
  return (
    <aside className="side-rail" aria-label={t('quickActions')}>
      <div className="quick-rail">
        <button
          type="button"
          className="quick-btn"
          title={t('copy')}
          onClick={onCopy}
          disabled={!hasSelection}
        >
          <Icon name="copy" />
        </button>
        <button
          type="button"
          className="quick-btn"
          title={t('favorite')}
          onClick={onFavorite}
          disabled={!hasSelection}
        >
          <Icon name="star" />
        </button>
        <button
          type="button"
          className="quick-btn"
          title={t('selectTransfer')}
          data-count={selectionCount > 0 ? selectionCount : undefined}
          onClick={onTransfer}
        >
          <Icon name="pin" />
        </button>
      </div>
      <div className="utility-rail">
        <button type="button" className="icon-btn rail-btn" title={t('settings')} onClick={onSettings}>
          <Icon name="gear" />
        </button>
        <button type="button" className="icon-btn rail-btn" title={t('clearHistory')} onClick={onClear}>
          <Icon name="trash" />
        </button>
      </div>
    </aside>
  );
}
