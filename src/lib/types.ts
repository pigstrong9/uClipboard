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
  thumb_data_url?: string;
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

export interface UpdateInfo {
  version: string;
  currentVersion: string;
  body: string | null;
  date: string | null;
}
