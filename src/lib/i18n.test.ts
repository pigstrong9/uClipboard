import { describe, expect, it } from 'vitest';

import { dictionaries, translate } from './i18n';

describe('translations', () => {
  it('keeps Chinese and English key sets in sync', () => {
    const zh = Object.keys(dictionaries['zh-CN']).sort();
    const en = Object.keys(dictionaries['en-US']).sort();
    expect(en).toEqual(zh);
  });

  it('never leaves a translation empty', () => {
    for (const dictionary of Object.values(dictionaries)) {
      for (const [key, value] of Object.entries(dictionary)) {
        expect(value, key).not.toBe('');
      }
    }
  });

  it('falls back to Chinese and keeps unknown placeholders', () => {
    expect(translate('en-US', 'filesCount', { n: 2 })).toBe('2 files');
    expect(translate('en-US', 'missing.key')).toBe('missing.key');
    expect(translate('en-US', 'filesCount')).toBe('{n} files');
  });
});
