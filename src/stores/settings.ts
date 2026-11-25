import { create } from 'zustand';

import * as ipc from '../lib/ipc';
import type { Settings } from '../lib/types';

interface SettingsState {
  settings: Settings | null;
  loading: boolean;
  error: string | null;
  init: () => Promise<void>;
  load: () => Promise<void>;
  save: (patch: Partial<Settings>) => Promise<Settings>;
  ignoreApp: (app: string) => Promise<boolean>;
  togglePause: () => Promise<void>;
  clearError: () => void;
}

let started = false;
let saveChain: Promise<void> = Promise.resolve();

export const useSettings = create<SettingsState>((set, get) => ({
  settings: null,
  loading: true,
  error: null,

  init: async () => {
    if (started) return;
    started = true;
    await get().load();
    try {
      await ipc.onSettingsChanged(() => {
        void get().load();
      });
    } catch {
      // Settings stay usable even if the event bridge is unavailable.
    }
  },

  load: async () => {
    try {
      const settings = await ipc.getSettings();
      set({ settings, loading: false, error: null });
    } catch (error) {
      set({ loading: false, error: String(error) });
    }
  },

  save: (patch) => {
    const task = saveChain.then(async () => {
      const current = get().settings;
      if (!current) {
        throw new Error('settings are not loaded');
      }
      const updated = await ipc.updateSettings({ ...current, ...patch });
      set({ settings: updated, error: null });
      return updated;
    });
    saveChain = task.then(
      () => undefined,
      () => undefined,
    );
    return task;
  },

  ignoreApp: (app) => {
    const name = app.trim();
    if (!name) return Promise.resolve(false);
    const task = saveChain.then(async () => {
      const current = get().settings;
      if (!current) {
        throw new Error('settings are not loaded');
      }
      if (
        current.ignoredApps.some((value) => value.toLowerCase() === name.toLowerCase())
      ) {
        return false;
      }
      const updated = await ipc.updateSettings({
        ...current,
        ignoredApps: [...current.ignoredApps, name],
      });
      set({ settings: updated, error: null });
      return true;
    });
    saveChain = task.then(
      () => undefined,
      () => undefined,
    );
    return task;
  },

  togglePause: async () => {
    await ipc.toggleMonitoring();
    await get().load();
  },

  clearError: () => set({ error: null }),
}));
