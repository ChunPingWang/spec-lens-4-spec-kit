/**
 * Terminal store — owns a single active PTY session per window, caches the
 * latest batch of output lines for non-xterm consumers (e.g. the overview
 * slice), and exposes connect/disconnect/write helpers backed by the
 * `terminal.*` IPC commands. See `specs/001-speclens-desktop/spec.md §US3`.
 */
import { create } from "zustand";

import { invoke, SpecLensIpcError } from "@/lib/tauri";
import type { OutputLine, PtySessionDescriptor, Uuid } from "@/types/ipc";

interface TerminalState {
  descriptor: PtySessionDescriptor | null;
  connecting: boolean;
  error: SpecLensIpcError | null;
  /** Rolling in-memory mirror of the most recent batch — used by tests and
   * by the OverviewPanel slice. The xterm instance writes its own buffer. */
  recentLines: OutputLine[];

  attach: (projectId: Uuid, windowId?: string) => Promise<PtySessionDescriptor>;
  detach: () => Promise<void>;
  write: (data: string) => Promise<void>;
  resize: (cols: number, rows: number) => Promise<void>;
  appendLines: (lines: OutputLine[]) => void;
  clear: () => void;
}

const MAX_RECENT = 1_000;

export const useTerminalStore = create<TerminalState>((set, get) => ({
  descriptor: null,
  connecting: false,
  error: null,
  recentLines: [],

  attach: async (projectId, windowId) => {
    set({ connecting: true, error: null });
    try {
      const descriptor = await invoke<PtySessionDescriptor>("terminal_attach", {
        projectId,
        windowId,
      });
      set({ descriptor, connecting: false });
      return descriptor;
    } catch (err) {
      set({
        connecting: false,
        error: err instanceof SpecLensIpcError ? err : null,
      });
      throw err;
    }
  },

  detach: async () => {
    const descriptor = get().descriptor;
    if (!descriptor) return;
    try {
      await invoke<void>("terminal_detach", { sessionId: descriptor.sessionId });
    } finally {
      set({ descriptor: null, recentLines: [] });
    }
  },

  write: async (data) => {
    const descriptor = get().descriptor;
    if (!descriptor) return;
    await invoke<void>("terminal_write", {
      sessionId: descriptor.sessionId,
      data,
    });
  },

  resize: async (cols, rows) => {
    const descriptor = get().descriptor;
    if (!descriptor) return;
    await invoke<void>("terminal_resize", {
      sessionId: descriptor.sessionId,
      cols,
      rows,
    });
  },

  appendLines: (lines) => {
    if (lines.length === 0) return;
    set((state) => {
      const next = state.recentLines.concat(lines);
      if (next.length > MAX_RECENT) {
        next.splice(0, next.length - MAX_RECENT);
      }
      return { recentLines: next };
    });
  },

  clear: () => set({ descriptor: null, recentLines: [], error: null }),
}));
