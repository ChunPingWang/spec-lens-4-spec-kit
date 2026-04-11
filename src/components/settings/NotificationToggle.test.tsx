/**
 * T150 — Vitest coverage for `NotificationToggle`. Verifies the switch
 * mirrors `configStore.config.notificationsEnabled`, hydrates from the
 * backend on first mount, and dispatches the toggle through
 * `setNotificationsEnabled` on click (FR-070).
 */
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import "@/i18n";

import { NotificationToggle } from "./NotificationToggle";
import { useConfigStore } from "@/stores/configStore";
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

function stubStore(overrides: Partial<ReturnType<typeof useConfigStore.getState>> = {}) {
  useConfigStore.setState({
    config: baseConfig,
    loading: false,
    error: null,
    load: vi.fn(async () => undefined),
    setLanguage: vi.fn(async () => undefined),
    setTheme: vi.fn(async () => undefined),
    setNotificationsEnabled: vi.fn(async (enabled: boolean) => {
      useConfigStore.setState({
        config: { ...baseConfig, notificationsEnabled: enabled },
      });
    }),
    ...overrides,
  } as unknown as Partial<ReturnType<typeof useConfigStore.getState>>);
}

describe("NotificationToggle", () => {
  beforeEach(() => {
    stubStore();
  });

  it("renders an enabled switch when notificationsEnabled is true", () => {
    render(<NotificationToggle />);
    const wrap = screen.getByTestId("notification-toggle");
    expect(wrap).toHaveAttribute("data-enabled", "true");
    const sw = screen.getByRole("switch");
    expect(sw).toHaveAttribute("aria-checked", "true");
  });

  it("renders a disabled switch when notificationsEnabled is false", () => {
    useConfigStore.setState({
      config: { ...baseConfig, notificationsEnabled: false },
    });
    render(<NotificationToggle />);
    const wrap = screen.getByTestId("notification-toggle");
    expect(wrap).toHaveAttribute("data-enabled", "false");
    expect(screen.getByRole("switch")).toHaveAttribute("aria-checked", "false");
  });

  it("invokes setNotificationsEnabled with the inverted value on click", async () => {
    render(<NotificationToggle />);
    await userEvent.click(screen.getByRole("switch"));
    await waitFor(() => {
      expect(useConfigStore.getState().setNotificationsEnabled).toHaveBeenCalledWith(false);
    });
    expect(screen.getByRole("switch")).toHaveAttribute("aria-checked", "false");
  });

  it("calls load() when the store has no cached config", () => {
    const load = vi.fn(async () => undefined);
    stubStore({ config: null, load });
    render(<NotificationToggle />);
    expect(load).toHaveBeenCalled();
  });
});
