import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { plural, relativeTime } from './format';
import { createTranslate } from './i18n';

const t = createTranslate('en-US');

describe('plural', () => {
  it('uses the singular form for one item', () => {
    expect(plural(t, 'filesCount', 1)).toBe('1 file');
    expect(plural(t, 'filesCount', 2)).toBe('2 files');
  });

  it('handles singular total and copied counts', () => {
    expect(plural(t, 'totalCount', 1)).toBe('1 item');
    expect(plural(t, 'copiedCount', 1)).toBe('Copied 1 item');
  });
});

describe('relativeTime', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2026-01-02T00:00:00Z'));
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  const now = Date.parse('2026-01-02T00:00:00Z');
  const minute = 60 * 1000;
  const hour = 60 * minute;
  const day = 24 * hour;

  it('uses singular units for one minute, hour and day', () => {
    expect(relativeTime(now - minute, t)).toBe('1 min ago');
    expect(relativeTime(now - hour, t)).toBe('1 hr ago');
    expect(relativeTime(now - day, t)).toBe('1 day ago');
  });

  it('keeps plural units for longer durations', () => {
    expect(relativeTime(now - 3 * day, t)).toBe('3 days ago');
  });
});
