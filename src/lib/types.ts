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
