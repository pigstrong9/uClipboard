import { Icon, type IconName } from '../lib/icons';
import type { TabKey } from '../lib/types';

const TABS: { key: TabKey; label: string; icon: IconName }[] = [
  { key: 'all', label: '全部', icon: 'layers' },
  { key: 'text', label: '文本', icon: 'text' },
  { key: 'image', label: '图像', icon: 'image' },
  { key: 'files', label: '文件', icon: 'files' },
  { key: 'favorite', label: '收藏', icon: 'star' },
];

interface Props {
  tab: TabKey;
  favoriteCount: number;
  onChange: (tab: TabKey) => void;
}

export function Tabs({ tab, favoriteCount, onChange }: Props) {
  return (
    <nav className="tabs" aria-label="分类">
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
          <span>{item.label}</span>
          {item.key === 'favorite' && <span className="tab-count">{favoriteCount}</span>}
        </button>
      ))}
    </nav>
  );
}
