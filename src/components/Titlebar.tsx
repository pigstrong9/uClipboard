import type { RefObject } from 'react';

import { Icon } from '../lib/icons';

interface Props {
  search: string;
  inputRef: RefObject<HTMLInputElement | null>;
  onSearch: (value: string) => void;
  theme: 'dark' | 'light';
  onToggleTheme: () => void;
  onMinimize: () => void;
}

export function Titlebar({
  search,
  inputRef,
  onSearch,
  theme,
  onToggleTheme,
  onMinimize,
}: Props) {
  return (
    <header className="titlebar" data-tauri-drag-region>
      <div className="brand">
        <span className="brand-name">剪贴板</span>
      </div>
      <label className="search" htmlFor="searchInput">
        <span className="search-icon">
          <Icon name="search" />
        </span>
        <input
          id="searchInput"
          ref={inputRef}
          value={search}
          onChange={(event) => onSearch(event.target.value)}
          placeholder="搜索…"
          autoComplete="off"
        />
      </label>
      <div className="title-actions">
        <button
          className="icon-btn"
          type="button"
          title="切换主题"
          onClick={onToggleTheme}
        >
          <Icon name={theme === 'dark' ? 'sun' : 'moon'} />
        </button>
        <button className="icon-btn" type="button" title="最小化" onClick={onMinimize}>
          <Icon name="minus" />
        </button>
      </div>
    </header>
  );
}
