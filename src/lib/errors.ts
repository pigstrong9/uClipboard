import type { Translate } from './i18n';

/// Backend commands return dotted codes for expected failures; anything else (SQLite,
/// IO) is already human-readable and is passed through unchanged.
const ERROR_KEYS: Record<string, string> = {
  'clipboard.write_failed': 'errorClipboardWrite',
  'clipboard.copy_empty': 'errorClipboardCopyEmpty',
  'clipboard.favorite_empty': 'errorClipboardFavoriteEmpty',
  'clipboard.not_image': 'errorClipboardNotImage',
  'clipboard.merge_mixed': 'errorClipboardMergeMixed',
  'paste.send_failed': 'errorPasteSend',
  'paste.init_failed': 'errorPasteInit',
  'paste.accessibility_required': 'errorPasteAccessibility',
  'window.missing': 'errorWindowMissing',
  'settings.save_failed': 'errorSettingsSave',
  'settings.hotkey_failed': 'errorSettingsHotkey',
  'settings.autostart_failed': 'errorSettingsAutostart',
  'logs.open_failed': 'errorLogsOpen',
  'image.too_large': 'errorImageTooLarge',
  'image.merge_too_many': 'errorImageMergeTooMany',
  'image.merge_too_large': 'errorImageMergeTooLarge',
  'image.merge_empty': 'errorImageMergeEmpty',
  'image.invalid_size': 'errorImageInvalid',
  'update.not_available': 'errorUpdateNotAvailable',
  'monitor.settings_failed': 'errorMonitorSettings',
  'monitor.privacy_failed': 'errorMonitorPrivacy',
  'monitor.read_failed': 'errorMonitorRead',
  'monitor.lock_failed': 'errorMonitorLock',
  'monitor.insert_failed': 'errorMonitorInsert',
  'monitor.stopped': 'errorMonitorStopped',
  'monitor.panicked': 'errorMonitorPanicked',
  'screenshot.busy': 'errorScreenshotBusy',
  'screenshot.capture_failed': 'errorScreenshotCapture',
  'screenshot.too_large': 'errorScreenshotTooLarge',
  'screenshot.window_failed': 'errorScreenshotWindow',
  'screenshot.session_missing': 'errorScreenshotSession',
  'screenshot.write_failed': 'errorScreenshotWrite',
  'screenshot.hotkey_fallback': 'errorScreenshotHotkeyFallback',
  'screenshot.hotkey_failed': 'errorScreenshotHotkeyFailed',
  'settings.hotkey_conflict': 'errorHotkeyConflict',
};

export const ERROR_CODES = Object.keys(ERROR_KEYS);

export function rawError(cause: unknown): string {
  if (typeof cause === 'string') return cause;
  if (cause instanceof Error) return cause.message;
  return cause == null ? '' : String(cause);
}

export function errorText(t: Translate, cause: unknown): string {
  const raw = rawError(cause).trim();
  if (!raw) return t('errorUnknown');
  const key = ERROR_KEYS[raw];
  return key ? t(key) : raw;
}
