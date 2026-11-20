import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

import type { ClipItem, TabKey } from './types';

export function listItems(tab: TabKey, query: string) {
  return invoke<ClipItem[]>('list_items', { tab, query });
}

export function toggleFavorite(id: string) {
  return invoke<ClipItem>('toggle_favorite', { id });
}

export function togglePin(id: string) {
  return invoke<ClipItem>('toggle_pin', { id });
}

export function deleteItem(id: string) {
  return invoke<void>('delete_item', { id });
}

export function clearItems() {
  return invoke<void>('clear_items');
}

export function copyItem(id: string) {
  return invoke<ClipItem>('copy_item', { id });
}

export function pasteItem(id: string) {
  return invoke<ClipItem>('paste_item', { id });
}

export function stats() {
  return invoke<number>('stats');
}

export function favoriteCount() {
  return invoke<number>('favorite_count');
}

export function imageDataUrl(id: string) {
  return invoke<string>('image_data_url', { id });
}

export function openTransfer(ids: string[]) {
  return invoke<ClipItem[]>('open_transfer', { ids });
}

export function transferItems() {
  return invoke<ClipItem[]>('transfer_items');
}

export function copyTransfer() {
  return invoke<number>('copy_transfer');
}

export function favoriteTransfer() {
  return invoke<boolean>('favorite_transfer');
}

export function closeTransfer() {
  return invoke<void>('close_transfer');
}

export function onClipboardChanged(callback: () => void): Promise<UnlistenFn> {
  return listen('clipboard-changed', () => callback());
}

export function onTransferChanged(callback: () => void): Promise<UnlistenFn> {
  return listen('transfer-changed', () => callback());
}
