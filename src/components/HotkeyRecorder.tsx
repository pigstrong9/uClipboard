import { useCallback, useEffect, useRef, useState } from 'react';

import { formatHotkey, isMacPlatform } from '../lib/hotkey';
import * as ipc from '../lib/ipc';
import { useI18n } from '../lib/useI18n';

interface Props {
  value: string;
  onCommit: (value: string) => void;
}

export function HotkeyRecorder({ value, onCommit }: Props) {
  const [recording, setRecording] = useState(false);
  const [expired, setExpired] = useState(false);
  const beginRef = useRef<Promise<void> | null>(null);
  const timeoutRef = useRef<number | null>(null);
  const buttonRef = useRef<HTMLButtonElement | null>(null);
  const { t } = useI18n();

  const clearTimer = useCallback(() => {
    if (timeoutRef.current !== null) {
      window.clearTimeout(timeoutRef.current);
      timeoutRef.current = null;
    }
  }, []);

  const endCapture = useCallback(async () => {
    const begin = beginRef.current;
    beginRef.current = null;
    if (begin) await begin.catch(() => undefined);
    await ipc.endHotkeyCapture().catch(() => undefined);
  }, []);

  const finish = useCallback(
    async (captured: string | null) => {
      clearTimer();
      setRecording(false);
      await endCapture();
      if (captured) onCommit(captured);
    },
    [clearTimer, endCapture, onCommit],
  );

  const start = useCallback(() => {
    if (recording) return;
    setRecording(true);
    setExpired(false);
    beginRef.current = ipc.beginHotkeyCapture().catch(() => undefined);
    clearTimer();
    timeoutRef.current = window.setTimeout(() => {
      void finish(null);
      // Leave the reason visible long enough to read, then restore the shortcut.
      setExpired(true);
      timeoutRef.current = window.setTimeout(() => setExpired(false), 4_000);
    }, 10_000);
    buttonRef.current?.focus();
  }, [recording, clearTimer, finish]);

  useEffect(
    () => () => {
      clearTimer();
      void endCapture();
    },
    [clearTimer, endCapture],
  );

  // Keys are captured at the window level: WKWebView on macOS does not focus a
  // button when it is clicked, so a handler bound to the button never fires there.
  useEffect(() => {
    if (!recording) return;
    const handler = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault();
        event.stopPropagation();
        void finish(null);
        return;
      }
      if (event.key === 'Tab') {
        void finish(null);
        return;
      }
      event.preventDefault();
      event.stopPropagation();
      if (event.repeat) return;
      const formatted = formatHotkey(event, isMacPlatform());
      if (formatted) void finish(formatted);
    };
    window.addEventListener('keydown', handler, true);
    return () => window.removeEventListener('keydown', handler, true);
  }, [recording, finish]);

  // Recording swallows every key, and the focus that `onBlur` would catch a click-away
  // on never arrives in the first place on macOS, so a click outside the button is the
  // only visible way to leave recording before the timeout.
  useEffect(() => {
    if (!recording) return;
    const handler = (event: PointerEvent) => {
      const button = buttonRef.current;
      if (button && button.contains(event.target as Node | null)) return;
      void finish(null);
    };
    window.addEventListener('pointerdown', handler, true);
    return () => window.removeEventListener('pointerdown', handler, true);
  }, [recording, finish]);

  return (
    <>
      <button
        ref={buttonRef}
        type="button"
        className={`settings-input hotkey-recorder ${recording ? 'recording' : ''}`}
        title={recording ? t('hotkeyRecordingHint') : value || t('hotkeyRecordingHint')}
        onClick={start}
        onBlur={() => {
          if (recording) void finish(null);
        }}
      >
        {recording ? t('hotkeyRecording') : value || t('hotkeyRecordingHint')}
      </button>
      {expired && <span className="settings-note">{t('hotkeyRecordingExpired')}</span>}
    </>
  );
}
