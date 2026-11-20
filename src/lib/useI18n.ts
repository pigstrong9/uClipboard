import { useEffect, useMemo } from 'react';

import { useSettings } from '../stores/settings';
import { createTranslate, resolveLanguage } from './i18n';

export function useI18n() {
  const setting = useSettings((state) => state.settings?.language);
  const language = resolveLanguage(setting);
  const t = useMemo(() => createTranslate(language), [language]);

  useEffect(() => {
    document.documentElement.lang = language;
  }, [language]);

  return { language, t };
}
