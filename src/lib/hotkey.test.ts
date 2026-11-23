import { describe, expect, it } from 'vitest';

import { formatHotkey } from './hotkey';

function key(
  value: string,
  code: string,
  mods: Partial<{ ctrlKey: boolean; altKey: boolean; shiftKey: boolean; metaKey: boolean }> = {},
) {
  return {
    key: value,
    code,
    ctrlKey: false,
    altKey: false,
    shiftKey: false,
    metaKey: false,
    ...mods,
  };
}

describe('formatHotkey', () => {
  it('formats common modifier combinations', () => {
    expect(formatHotkey(key('v', 'KeyV', { altKey: true }))).toBe('Alt+V');
    expect(formatHotkey(key('A', 'KeyA', { ctrlKey: true, shiftKey: true }))).toBe('Ctrl+Shift+A');
    expect(formatHotkey(key(' ', 'Space', { ctrlKey: true }))).toBe('Ctrl+Space');
    expect(formatHotkey(key('ArrowUp', 'ArrowUp', { ctrlKey: true }))).toBe('Ctrl+Up');
  });

  it('allows function keys and PrintScreen without modifiers', () => {
    expect(formatHotkey(key('F8', 'F8'))).toBe('F8');
    expect(formatHotkey(key('PrintScreen', 'PrintScreen'))).toBe('PrintScreen');
  });

  it('rejects unsafe or incomplete shortcuts', () => {
    expect(formatHotkey(key('Shift', 'ShiftLeft', { shiftKey: true }))).toBeNull();
    expect(formatHotkey(key('a', 'KeyA'))).toBeNull();
    expect(formatHotkey(key('Escape', 'Escape'))).toBeNull();
    expect(formatHotkey(key('Enter', 'Enter'))).toBeNull();
  });

  it('labels the meta key per platform', () => {
    expect(formatHotkey(key('a', 'KeyA', { metaKey: true }), false)).toBe('Super+A');
    expect(formatHotkey(key('a', 'KeyA', { metaKey: true }), true)).toBe('Command+A');
  });
});
