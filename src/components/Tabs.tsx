import { Icon, type IconName } from '../lib/icons';
import { useI18n } from '../lib/useI18n';
import type { TabKey } from '../lib/types';

const TABS: { key: TabKey; labelKey: string; icon: IconName }[] = [
  { key: 'all', labelKey: 'all', icon: 'layers' },
  { key: 'text', labelKey: 'text', icon: 'text' },
  { key: 'image', labelKey: 'imageTab', icon: 'image' },
  { key: 'files', labelKey: 'filesTab', icon: 'files' },
  { key: 'favorite', labelKey: 'favorites', icon: 'star' },
];

interface Props {
  tab: TabKey;
  favoriteCount: number;
  onChange: (tab: TabKey) => void;
}

export function Tabs({ tab, favoriteCount, onChange }: Props) {
  const { t } = useI18n();
  return (
    <nav className="tabs" aria-label={t('categories')}>
      {TABS.map((item) => (
        <button
          key={item.key}
          type="button"
          className={`tab ${tab === item.key ? 'active' : ''}`}
          onClick={() => onChange(item.key)}
          role="tab"
          aria-selected={tab === item.key}
        >
          <span className="tab-icon">
            <Icon name={item.icon} />
          </span>
          <span>{t(item.labelKey)}</span>
          {item.key === 'favorite' && <span className="tab-count">{favoriteCount}</span>}
        </button>
      ))}
    </nav>
  );
}
