import { create } from 'zustand';

let toastTimer: number | undefined;

interface UiState {
  message: string;
  visible: boolean;
  toast: (message: string) => void;
  hide: () => void;
}

export const useUi = create<UiState>((set) => ({
  message: '',
  visible: false,
  toast: (message) => {
    set({ message, visible: true });
    window.clearTimeout(toastTimer);
    toastTimer = window.setTimeout(() => set({ visible: false }), 1800);
  },
  hide: () => set({ visible: false }),
}));
