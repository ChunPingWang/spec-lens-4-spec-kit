/**
 * Zustand store for the currently-open project in this window.
 * One window holds exactly one project (see `research.md D-multi-window`).
 */
import { create } from "zustand";

import { invoke, SpecLensIpcError } from "@/lib/tauri";
import type { Project, RecentProject, Uuid } from "@/types/ipc";

interface ProjectState {
  current: Project | null;
  recent: RecentProject[];
  loading: boolean;
  error: SpecLensIpcError | null;
  loadRecent: () => Promise<void>;
  open: (path: string) => Promise<Project>;
  close: () => Promise<void>;
  pin: (id: Uuid, pinned: boolean) => Promise<void>;
  remove: (id: Uuid) => Promise<void>;
  clearError: () => void;
}

export const useProjectStore = create<ProjectState>((set, get) => ({
  current: null,
  recent: [],
  loading: false,
  error: null,

  loadRecent: async () => {
    try {
      const recent = await invoke<RecentProject[]>("project_list_recent");
      set({ recent, error: null });
    } catch (err) {
      // Fallback to empty list while the backend is still a stub.
      if (err instanceof SpecLensIpcError && err.code === "E_INTERNAL") {
        set({ recent: [] });
        return;
      }
      set({ error: err instanceof SpecLensIpcError ? err : null });
    }
  },

  open: async (path) => {
    set({ loading: true, error: null });
    try {
      const project = await invoke<Project>("project_open", { path });
      set({ current: project, loading: false });
      return project;
    } catch (err) {
      const ipcErr = err instanceof SpecLensIpcError ? err : null;
      set({ loading: false, error: ipcErr });
      throw err;
    }
  },

  close: async () => {
    const current = get().current;
    if (!current) return;
    try {
      await invoke("project_close", { id: current.id });
    } finally {
      set({ current: null });
    }
  },

  pin: async (id, pinned) => {
    await invoke("project_pin_recent", { id, pinned });
    await get().loadRecent();
  },

  remove: async (id) => {
    await invoke("project_remove_recent", { id });
    await get().loadRecent();
  },

  clearError: () => set({ error: null }),
}));
