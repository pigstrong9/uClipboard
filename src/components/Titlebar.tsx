import type { RefObject } from 'react';

import logoUrl from '../assets/logo.png';
import { Icon } from '../lib/icons';
import { useI18n } from '../lib/useI18n';

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
  const { t } = useI18n();
  return (
    <header className="titlebar" data-tauri-drag-region>
      <div className="brand">
        <img className="brand-logo" src={logoUrl} alt="" />
        <span className="brand-name">uClipboard</span>
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
          placeholder={t('searchPlaceholder')}
          autoComplete="off"
        />
      </label>
      <div className="title-actions">
        <button
          className="icon-btn"
          type="button"
          title={t('toggleTheme')}
          onClick={onToggleTheme}
        >
          <Icon name={theme === 'dark' ? 'sun' : 'moon'} />
        </button>
        <button className="icon-btn" type="button" title={t('minimize')} onClick={onMinimize}>
          <Icon name="minus" />
        </button>
      </div>
    </header>
  );
}
