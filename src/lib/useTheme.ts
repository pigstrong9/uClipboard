import { useEffect, useLayoutEffect, useState } from 'react';

import { useSettings } from '../stores/settings';

export function useTheme() {
  const setting = useSettings((state) => state.settings?.theme) ?? 'system';
  const save = useSettings((state) => state.save);
  const [systemDark, setSystemDark] = useState(
    () => window.matchMedia?.('(prefers-color-scheme: dark)').matches ?? true,
  );

  useEffect(() => {
    const media = window.matchMedia?.('(prefers-color-scheme: dark)');
    if (!media) return;
    const handler = (event: MediaQueryListEvent) => setSystemDark(event.matches);
    setSystemDark(media.matches);
    media.addEventListener('change', handler);
    return () => media.removeEventListener('change', handler);
  }, []);

  const theme = setting === 'system' ? (systemDark ? 'dark' : 'light') : setting;

  useLayoutEffect(() => {
    document.documentElement.dataset.theme = theme;
  }, [theme]);

  const toggleTheme = () => {
    const next = theme === 'dark' ? 'light' : 'dark';
    void save({ theme: next }).catch(() => undefined);
  };

  return { theme, setting, toggleTheme };
}
