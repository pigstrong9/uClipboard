import { create } from 'zustand';

let toastTimer: number | undefined;

export interface ConfirmRequest {
  message: string;
  onConfirm: () => void;
}

interface UiState {
  message: string;
  visible: boolean;
  confirm: ConfirmRequest | null;
  toast: (message: string) => void;
  hide: () => void;
  ask: (request: ConfirmRequest) => void;
  answer: (accepted: boolean) => void;
}

export const useUi = create<UiState>((set, get) => ({
  message: '',
  visible: false,
  confirm: null,
  toast: (message) => {
    set({ message, visible: true });
    window.clearTimeout(toastTimer);
    toastTimer = window.setTimeout(() => set({ visible: false }), 1800);
  },
  hide: () => set({ visible: false }),
  ask: (request) => set({ confirm: request }),
  answer: (accepted) => {
    const request = get().confirm;
    set({ confirm: null });
    if (accepted) request?.onConfirm();
  },
}));
