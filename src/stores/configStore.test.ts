/**
 * T146 — Vitest coverage for `configStore` notification toggle
 * persistence (US7 / FR-070).
 *
 * Asserts:
 *   - `setNotificationsEnabled(false)` invokes `config_set_notifications_enabled`
 *     with `{ enabled: false }` and stores the returned `AppConfig`;
 *   - `setNotificationsEnabled(true)` round-trips the value back through
 *     the IPC layer (no client-side optimism — backend stays the source
 *     of truth);
 *   - `load` populates the cache from `config_get`.
 */
import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useConfigStore } from "./configStore";
import type { AppConfig } from "@/types/ipc";

const baseConfig: AppConfig = {
  schemaVersion: 1,
  language: "system",
  theme: "system",
  recentProjects: [],
  terminalBufferMaxLines: 10_000,
  terminalDiskCapMiB: 512,
  openWindows: [],
  notificationsEnabled: true,
  lastUpdatedAt: "2026-04-11T00:00:00Z",
};

const mockedInvoke = vi.mocked(tauriInvoke);

function resetStore(): void {
  useConfigStore.setState({ config: null, loading: false, error: null });
}

describe("configStore", () => {
  beforeEach(() => {
    resetStore();
    mockedInvoke.mockReset();
  });

  it("hydrates the cache via config_get on load()", async () => {
    mockedInvoke.mockImplementationOnce(async (cmd) => {
      expect(cmd).toBe("config_get");
      return baseConfig;
    });

    await useConfigStore.getState().load();

    expect(useConfigStore.getState().config).toEqual(baseConfig);
    expect(useConfigStore.getState().loading).toBe(false);
    expect(useConfigStore.getState().error).toBeNull();
  });

  it("persists the notifications toggle off through config_set_notifications_enabled", async () => {
    const flipped: AppConfig = { ...baseConfig, notificationsEnabled: false };
    mockedInvoke.mockImplementationOnce(async (cmd, args) => {
      expect(cmd).toBe("config_set_notifications_enabled");
      expect(args).toEqual({ enabled: false });
      return flipped;
    });

    await useConfigStore.getState().setNotificationsEnabled(false);

    const state = useConfigStore.getState();
    expect(state.config).toEqual(flipped);
    expect(state.config?.notificationsEnabled).toBe(false);
  });

  it("round-trips the notifications toggle back on through the same command", async () => {
    useConfigStore.setState({ config: { ...baseConfig, notificationsEnabled: false } });

    mockedInvoke.mockImplementationOnce(async (cmd, args) => {
      expect(cmd).toBe("config_set_notifications_enabled");
      expect(args).toEqual({ enabled: true });
      return { ...baseConfig, notificationsEnabled: true };
    });

    await useConfigStore.getState().setNotificationsEnabled(true);

    expect(useConfigStore.getState().config?.notificationsEnabled).toBe(true);
  });

  it("captures the error message when config_get rejects", async () => {
    mockedInvoke.mockImplementationOnce(async () => {
      throw new Error("disk unreachable");
    });

    await useConfigStore.getState().load();

    expect(useConfigStore.getState().config).toBeNull();
    expect(useConfigStore.getState().error).toContain("disk unreachable");
    expect(useConfigStore.getState().loading).toBe(false);
  });
});
