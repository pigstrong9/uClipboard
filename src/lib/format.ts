import type { Translate } from './i18n';

/// Picks the `...One` translation when there is exactly one item. Chinese dictionaries
/// repeat the same string for both keys so callers do not need language checks.
export function plural(t: Translate, key: string, n: number): string {
  return t(n === 1 ? `${key}One` : key, { n });
}

export function relativeTime(timestamp: number, t: Translate): string {
  const diff = Date.now() - timestamp;
  const minute = 60 * 1000;
  const hour = 60 * minute;
  const day = 24 * hour;
  if (diff < minute) return t('justNow');
  if (diff < hour) return plural(t, 'minutesAgo', Math.floor(diff / minute));
  if (diff < day) return plural(t, 'hoursAgo', Math.floor(diff / hour));
  return plural(t, 'daysAgo', Math.floor(diff / day));
}

export function baseName(path: string): string {
  return path.split(/[\\/]/).pop() || path;
}
