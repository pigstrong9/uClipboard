import { create } from 'zustand';

import * as ipc from '../lib/ipc';
import type { ClipSummary, TabKey } from '../lib/types';

interface ClipboardState {
  items: ClipSummary[];
  tab: TabKey;
  query: string;
  loading: boolean;
  error: string | null;
  selectedId: string | null;
  selectedIds: string[];
  favoriteCount: number;
  totalCount: number;
  load: () => Promise<void>;
  setTab: (tab: TabKey) => void;
  setQuery: (query: string) => void;
  select: (id: string | null) => void;
  toggleSelection: (id: string) => void;
  clearSelection: () => void;
  openTransfer: (ids: string[]) => Promise<void>;
  toggleFavorite: (id: string) => Promise<void>;
  togglePin: (id: string) => Promise<void>;
  remove: (id: string) => Promise<void>;
  copy: (id: string) => Promise<void>;
  paste: (id: string) => Promise<void>;
  clear: () => Promise<void>;
  clearError: () => void;
}

let requestSeq = 0;

export const useClipboard = create<ClipboardState>((set, get) => ({
  items: [],
  tab: 'all',
  query: '',
  loading: true,
  error: null,
  selectedId: null,
  selectedIds: [],
  favoriteCount: 0,
  totalCount: 0,

  load: async () => {
    const seq = ++requestSeq;
    try {
      const [items, favoriteCount, totalCount] = await Promise.all([
        ipc.listItems(get().tab, get().query),
        ipc.favoriteCount(),
        ipc.stats(),
      ]);
      if (seq !== requestSeq) return;
      const current = get().selectedId;
      const selectedId =
        current && items.some((item) => item.id === current) ? current : null;
      set({ items, favoriteCount, totalCount, loading: false, error: null, selectedId });
    } catch (error) {
      if (seq !== requestSeq) return;
      set({ loading: false, error: String(error) });
    }
  },

  setTab: (tab) => {
    set({ tab, selectedId: null, selectedIds: [], loading: true });
    void get().load();
  },

  setQuery: (query) => {
    set({ query, selectedId: null, loading: true });
    void get().load();
  },

  select: (selectedId) => set({ selectedId }),

  toggleSelection: (id) => {
    set((state) => ({
      selectedIds: state.selectedIds.includes(id)
        ? state.selectedIds.filter((value) => value !== id)
        : [...state.selectedIds, id],
    }));
  },

  clearSelection: () => set({ selectedIds: [] }),

  openTransfer: async (ids) => {
    await ipc.openTransfer(ids);
  },

  toggleFavorite: async (id) => {
    await ipc.toggleFavorite(id);
    await get().load();
  },

  togglePin: async (id) => {
    await ipc.togglePin(id);
    await get().load();
  },

  remove: async (id) => {
    await ipc.deleteItem(id);
    set((state) => ({
      selectedIds: state.selectedIds.filter((value) => value !== id),
      selectedId: state.selectedId === id ? null : state.selectedId,
    }));
    await get().load();
  },

  copy: async (id) => {
    await ipc.copyItem(id);
    await get().load();
  },

  paste: async (id) => {
    await ipc.pasteItem(id);
    await get().load();
  },

  clear: async () => {
    await ipc.clearItems();
    set({ selectedId: null, selectedIds: [] });
    await get().load();
  },

  clearError: () => set({ error: null }),
}));
