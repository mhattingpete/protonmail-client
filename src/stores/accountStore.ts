import { create } from "zustand";
import * as tauri from "../lib/tauri.ts";

interface AccountState {
  connected: boolean;
  loading: boolean;
  error: string | null;
  email: string;

  checkConnection: () => Promise<void>;
  setBridgePassword: (password: string, email: string) => Promise<void>;
  fetchAccountEmail: () => Promise<void>;
}

export const useAccountStore = create<AccountState>((set) => ({
  connected: false,
  loading: false,
  error: null,
  email: "",

  checkConnection: async () => {
    set({ loading: true, error: null });
    try {
      const connected = await tauri.getConnectionStatus();
      set({ connected, loading: false });
      if (connected) {
        // Also fetch the stored email
        try {
          const email = await tauri.getAccountEmail();
          if (email) set({ email });
        } catch { /* not critical */ }
      }
    } catch {
      set({ connected: false, loading: false });
    }
  },

  setBridgePassword: async (password: string, email: string) => {
    set({ loading: true, error: null });
    try {
      await tauri.setBridgePassword(password, email);
      const connected = await tauri.getConnectionStatus();
      set({ connected, loading: false, email });
    } catch (err) {
      set({
        loading: false,
        error: err instanceof Error ? err.message : "Failed to set bridge password",
      });
    }
  },

  fetchAccountEmail: async () => {
    try {
      const email = await tauri.getAccountEmail();
      if (email) set({ email });
    } catch { /* not critical */ }
  },
}));
