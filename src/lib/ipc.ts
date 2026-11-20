import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

import type { ClipItem, Settings, TabKey, UpdateInfo } from './types';

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

export function pasteTransferItem(id: string) {
  return invoke<ClipItem>('paste_transfer_item', { id });
}

export function stats() {
  return invoke<number>('stats');
}

export function favoriteCount() {
  return invoke<number>('favorite_count');
}

export function imageFilePath(id: string) {
  return invoke<string>('image_file_path', { id });
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

export function getSettings() {
  return invoke<Settings>('get_settings');
}

export function updateSettings(settings: Settings) {
  return invoke<Settings>('update_settings', { settings });
}

export function toggleMonitoring() {
  return invoke<boolean>('toggle_monitoring');
}

export function hideMain() {
  return invoke<void>('hide_main');
}

export function checkUpdate() {
  return invoke<UpdateInfo | null>('check_update');
}

export function installUpdate() {
  return invoke<void>('install_update');
}

export function onClipboardChanged(callback: () => void): Promise<UnlistenFn> {
  return listen('clipboard-changed', () => callback());
}

export function onTransferChanged(callback: () => void): Promise<UnlistenFn> {
  return listen('transfer-changed', () => callback());
}

export function onSettingsChanged(callback: () => void): Promise<UnlistenFn> {
  return listen('settings-changed', () => callback());
}

export function onMainShown(callback: () => void): Promise<UnlistenFn> {
  return listen('main-shown', () => callback());
}

export function onOpenSettings(callback: () => void): Promise<UnlistenFn> {
  return listen('open-settings', () => callback());
}

export function onRequestClear(callback: () => void): Promise<UnlistenFn> {
  return listen('request-clear', () => callback());
}

export function onToastMessage(callback: (message: string) => void): Promise<UnlistenFn> {
  return listen<string>('toast-message', (event) => callback(event.payload));
}

export function onToastKey(callback: (key: string) => void): Promise<UnlistenFn> {
  return listen<string>('toast-key', (event) => callback(event.payload));
}

export function onUpdateAvailable(callback: (version: string) => void): Promise<UnlistenFn> {
  return listen<string>('update-available', (event) => callback(event.payload));
}
