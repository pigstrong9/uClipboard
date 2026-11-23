import { describe, expect, it } from 'vitest';

import { ERROR_CODES, errorText } from './errors';
import { createTranslate } from './i18n';

const t = createTranslate('en-US');

describe('errorText', () => {
  it('translates known backend error codes', () => {
    expect(errorText(t, 'clipboard.write_failed')).toBe('Failed to write to clipboard');
    expect(errorText(t, 'paste.accessibility_required')).toBe(
      'Accessibility permission is required for automatic paste',
    );
  });

  it('passes through unknown details unchanged', () => {
    expect(errorText(t, 'database is locked')).toBe('database is locked');
    expect(errorText(t, new Error('boom'))).toBe('boom');
  });

  it('falls back to a translated message for empty errors', () => {
    expect(errorText(t, '')).toBe('Something went wrong');
    expect(errorText(t, undefined)).toBe('Something went wrong');
  });

  it('has a translation for every known backend code', () => {
    for (const code of ERROR_CODES) {
      const text = errorText(t, code);
      expect(text, code).not.toBe(code);
      expect(text, code).not.toMatch(/^error[A-Z]/);
    }
  });
});
