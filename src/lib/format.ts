import type { Translate } from './i18n';

export function relativeTime(timestamp: number, t: Translate): string {
  const diff = Date.now() - timestamp;
  const minute = 60 * 1000;
  const hour = 60 * minute;
  const day = 24 * hour;
  if (diff < minute) return t('justNow');
  if (diff < hour) return t('minutesAgo', { n: Math.floor(diff / minute) });
  if (diff < day) return t('hoursAgo', { n: Math.floor(diff / hour) });
  return t('daysAgo', { n: Math.floor(diff / day) });
}

export function baseName(path: string): string {
  return path.split(/[\\/]/).pop() || path;
}
