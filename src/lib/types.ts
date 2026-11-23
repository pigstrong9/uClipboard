export type ClipKind = 'text' | 'image' | 'files';
export type TabKey = 'all' | 'text' | 'image' | 'files' | 'favorite';

export interface ClipItem {
  id: string;
  kind: ClipKind;
  content: string;
  extra: string | null;
  preview: string;
  size: number;
  source_app: string | null;
  favorite: boolean;
  pinned: boolean;
  copy_count: number;
  created_at: number;
  updated_at: number;
  thumb_path?: string | null;
}

/**
 * The list row shape. Long text and HTML stay on the backend until the preview needs them.
 */
export interface ClipSummary {
  id: string;
  kind: ClipKind;
  preview: string;
  text: string;
  truncated: boolean;
  files: string[];
  width: number | null;
  height: number | null;
  file_count: number | null;
  size: number;
  source_app: string | null;
  favorite: boolean;
  pinned: boolean;
  copy_count: number;
  created_at: number;
  updated_at: number;
  thumb_path?: string | null;
}

export interface ItemExtra {
  html?: string;
  width?: number;
  height?: number;
  count?: number;
}

export function parseExtra(extra: string | null): ItemExtra {
  if (!extra) return {};
  try {
    return JSON.parse(extra) as ItemExtra;
  } catch {
    return {};
  }
}

export function parseFiles(content: string): string[] {
  try {
    const files = JSON.parse(content);
    return Array.isArray(files) ? files.filter((f): f is string => typeof f === 'string') : [];
  } catch {
    return [];
  }
}

export type ThemeSetting = 'system' | 'light' | 'dark';
export type LanguageSetting = 'system' | 'zh-CN' | 'en-US';

export interface Settings {
  hotkey: string;
  screenshotHotkey: string;
  autoStart: boolean;
  hideOnBlur: boolean;
  theme: ThemeSetting;
  language: LanguageSetting;
  historyLimit: number;
  retentionDays: number;
  recordText: boolean;
  recordImages: boolean;
  recordFiles: boolean;
  minTextLength: number;
  ignoredApps: string[];
  pastePlainText: boolean;
  autoUpdate: boolean;
  monitorPaused: boolean;
  maxTextBytes: number;
  maxImageBytes: number;
}

/**
 * One overlay's capture. Each display gets its own image at its own pixel resolution,
 * so the CSS size of the window and the size of this image give the crop scale.
 */
export interface ScreenshotSession {
  imagePath: string;
  width: number;
  height: number;
}

export interface UpdateInfo {
  version: string;
  currentVersion: string;
  body: string | null;
  date: string | null;
}
