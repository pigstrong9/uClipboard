export interface HotkeyEvent {
  key: string;
  code?: string;
  ctrlKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
  metaKey: boolean;
}

const MODIFIER_KEYS = new Set([
  'Control',
  'Shift',
  'Alt',
  'Meta',
  'OS',
  'AltGraph',
  'CapsLock',
  'NumLock',
  'ScrollLock',
  'Fn',
]);

const CODE_KEYS: Record<string, string> = {
  Space: 'Space',
  Enter: 'Enter',
  NumpadEnter: 'Enter',
  Tab: 'Tab',
  Backspace: 'Backspace',
  Delete: 'Delete',
  Insert: 'Insert',
  Home: 'Home',
  End: 'End',
  PageUp: 'PageUp',
  PageDown: 'PageDown',
  ArrowUp: 'Up',
  ArrowDown: 'Down',
  ArrowLeft: 'Left',
  ArrowRight: 'Right',
  PrintScreen: 'PrintScreen',
  Pause: 'Pause',
  Backquote: 'Backquote',
  Backslash: 'Backslash',
  BracketLeft: 'BracketLeft',
  BracketRight: 'BracketRight',
  Comma: 'Comma',
  Equal: 'Equal',
  Minus: 'Minus',
  Period: 'Period',
  Quote: 'Quote',
  Semicolon: 'Semicolon',
  Slash: 'Slash',
};

const NAMED_KEYS: Record<string, string> = {
  ArrowUp: 'Up',
  ArrowDown: 'Down',
  ArrowLeft: 'Left',
  ArrowRight: 'Right',
  ' ': 'Space',
};

export function isMacPlatform(): boolean {
  if (typeof navigator === 'undefined') return false;
  return /Mac|iPhone|iPad/.test(navigator.platform || navigator.userAgent);
}

function keyFromEvent(event: HotkeyEvent): string | null {
  const code = event.code ?? '';
  if (/^Key[A-Z]$/.test(code)) return code.slice(3);
  if (/^Digit[0-9]$/.test(code)) return code.slice(5);
  if (/^Numpad[0-9]$/.test(code)) return code;
  if (/^F([1-9]|1\d|2[0-4])$/.test(code)) return code;
  if (CODE_KEYS[code]) return CODE_KEYS[code];
  if (NAMED_KEYS[event.key]) return NAMED_KEYS[event.key];
  if (event.key === 'Escape') return null;
  if (/^[a-zA-Z]$/.test(event.key)) return event.key.toUpperCase();
  if (/^[0-9]$/.test(event.key)) return event.key;
  if (/^F([1-9]|1\d|2[0-4])$/i.test(event.key)) return event.key.toUpperCase();
  if (event.key === 'PrintScreen') return 'PrintScreen';
  return null;
}

/// Turns a keyboard event into the accelerator string understood by the global
/// shortcut plugin. Plain letters/digits are rejected unless a modifier is held, so
/// focusing the recorder and typing normally cannot register a global hotkey by accident.
export function formatHotkey(event: HotkeyEvent, mac = isMacPlatform()): string | null {
  if (MODIFIER_KEYS.has(event.key) || event.key === 'Escape') return null;
  const key = keyFromEvent(event);
  if (!key) return null;

  const mods: string[] = [];
  if (event.ctrlKey) mods.push('Ctrl');
  if (event.altKey) mods.push('Alt');
  if (event.shiftKey) mods.push('Shift');
  if (event.metaKey) mods.push(mac ? 'Command' : 'Super');

  const standalone = /^F([1-9]|1\d|2[0-4])$/.test(key) || key === 'PrintScreen';
  if (mods.length === 0 && !standalone) return null;

  return [...mods, key].join('+');
}
