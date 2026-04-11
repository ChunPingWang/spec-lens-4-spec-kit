/**
 * Zustand store for cross-project `AppConfig`. Mirrors the
 * `config.*` IPC commands. Backend is the source of truth; this store
 * caches the latest snapshot for synchronous UI reads.
 */
import { create } from "zustand";

import { invoke } from "@/lib/tauri";
import type { AppConfig, LanguagePreference, ThemePreference } from "@/types/ipc";

interface ConfigState {
  config: AppConfig | null;
  loading: boolean;
  error: string | null;
  load: () => Promise<void>;
  setLanguage: (language: LanguagePreference) => Promise<void>;
  setTheme: (theme: ThemePreference) => Promise<void>;
  /**
   * Phase 9 US7 toggle (FR-070). Persists through `config_set_*` so the
   * backend remains the source of truth. The store always reflects the
   * value the backend last accepted, never an optimistic guess.
   */
  setNotificationsEnabled: (enabled: boolean) => Promise<void>;
}

export const useConfigStore = create<ConfigState>((set) => ({
  config: null,
  loading: false,
  error: null,

  load: async () => {
    set({ loading: true, error: null });
    try {
      const config = await invoke<AppConfig>("config_get");
      set({ config, loading: false });
    } catch (err) {
      set({ loading: false, error: err instanceof Error ? err.message : String(err) });
    }
  },

  setLanguage: async (language) => {
    const config = await invoke<AppConfig>("config_set_language", { language });
    set({ config });
  },

  setTheme: async (theme) => {
    const config = await invoke<AppConfig>("config_set_theme", { theme });
    set({ config });
  },

  setNotificationsEnabled: async (enabled) => {
    const config = await invoke<AppConfig>("config_set_notifications_enabled", { enabled });
    set({ config });
  },
}));
