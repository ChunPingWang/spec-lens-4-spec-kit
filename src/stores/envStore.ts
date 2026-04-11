/**
 * Zustand store for the Spec-Kit environment check (FR-010 – FR-013).
 *
 * Holds the current `EnvironmentStatus` for the active project plus the
 * bundled install guide payload. The store is driven by two Tauri
 * commands: `env_check` (probe + cache) and `env_get_install_guide`
 * (platform-aware bundled steps). Consumers listen via selectors; the
 * Workspace page is gated behind `acknowledged` so a red result always
 * blocks until the user installs Spec-Kit or explicitly dismisses.
 */
import { create } from "zustand";

import { invoke, SpecLensIpcError } from "@/lib/tauri";
import type { EnvIssue, EnvironmentStatus, InstallGuide, Uuid } from "@/types/ipc";

/** Badge colour derived from an `EnvironmentStatus` mirroring the
 *  Rust `EnvironmentStatus::badge_color` logic so the two layers agree. */
export type EnvBadgeColor = "green" | "amber" | "red";

export function deriveBadgeColor(status: EnvironmentStatus): EnvBadgeColor {
  const hasError = status.issues.some((i) => i.severity === "error");
  const hasWarn = status.issues.some((i) => i.severity === "warn");
  if (!status.speckitInstalled || hasError) return "red";
  if (status.updateAvailable || hasWarn) return "amber";
  return "green";
}

interface EnvState {
  status: EnvironmentStatus | null;
  guide: InstallGuide | null;
  loading: boolean;
  guideLoading: boolean;
  error: SpecLensIpcError | null;
  /** User has dismissed the current result (amber/green). Reset on `check`. */
  acknowledged: boolean;

  check: (projectId: Uuid, force?: boolean) => Promise<EnvironmentStatus>;
  loadGuide: (platform?: string) => Promise<InstallGuide>;
  acknowledge: () => void;
  clear: () => void;
}

export const useEnvStore = create<EnvState>((set) => ({
  status: null,
  guide: null,
  loading: false,
  guideLoading: false,
  error: null,
  acknowledged: false,

  check: async (projectId, force = false) => {
    set({ loading: true, error: null, acknowledged: false });
    try {
      const status = await invoke<EnvironmentStatus>("env_check", {
        projectId,
        force,
      });
      set({ status, loading: false });
      return status;
    } catch (err) {
      set({
        loading: false,
        error: err instanceof SpecLensIpcError ? err : null,
      });
      throw err;
    }
  },

  loadGuide: async (platform) => {
    set({ guideLoading: true });
    try {
      const guide = await invoke<InstallGuide>("env_get_install_guide", {
        platform: platform ?? null,
      });
      set({ guide, guideLoading: false });
      return guide;
    } catch (err) {
      set({
        guideLoading: false,
        error: err instanceof SpecLensIpcError ? err : null,
      });
      throw err;
    }
  },

  acknowledge: () => set({ acknowledged: true }),

  clear: () =>
    set({
      status: null,
      guide: null,
      loading: false,
      guideLoading: false,
      error: null,
      acknowledged: false,
    }),
}));

/** Helper: return the first error-severity issue for quick summaries. */
export function firstErrorIssue(status: EnvironmentStatus): EnvIssue | null {
  return status.issues.find((i) => i.severity === "error") ?? null;
}
