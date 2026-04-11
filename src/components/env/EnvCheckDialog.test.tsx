/**
 * T128 — Vitest coverage for `EnvCheckDialog` asserting the three
 * render branches (red / amber / green) and the install-guide fetch
 * fired in the red path. Mirrors the US5 integration tests:
 *   - red   → install guide visible, no "Continue" button;
 *   - amber → warning text + "Continue anyway";
 *   - acknowledge hides the dialog via envStore state.
 */
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import "@/i18n";

import { EnvCheckDialog } from "./EnvCheckDialog";
import { useEnvStore } from "@/stores/envStore";
import type { EnvironmentStatus, InstallGuide } from "@/types/ipc";

const projectId = "00000000-0000-0000-0000-000000000001";

function redStatus(): EnvironmentStatus {
  return {
    speckitInstalled: false,
    updateAvailable: false,
    checkedAt: "2026-04-11T00:00:00Z",
    issues: [
      {
        kind: "missing_binary",
        severity: "error",
        message: "spec-kit binary not found on PATH",
        hint: "Install Spec-Kit CLI and re-run the check.",
      },
    ],
  };
}

function amberStatus(): EnvironmentStatus {
  return {
    speckitInstalled: true,
    speckitVersion: "0.5.0",
    latestKnownVersion: "0.6.0",
    updateAvailable: true,
    checkedAt: "2026-04-11T00:00:00Z",
    issues: [
      {
        kind: "version_mismatch",
        severity: "warn",
        message: "Installed Spec-Kit 0.5.0 is older than latest known 0.6.0",
        hint: "Run the in-app updater from the environment banner.",
      },
    ],
  };
}

function greenStatus(): EnvironmentStatus {
  return {
    speckitInstalled: true,
    speckitVersion: "0.6.0",
    latestKnownVersion: "0.6.0",
    updateAvailable: false,
    checkedAt: "2026-04-11T00:00:00Z",
    issues: [],
  };
}

function stubStore(overrides: Partial<ReturnType<typeof useEnvStore.getState>> = {}) {
  useEnvStore.setState({
    status: null,
    guide: null,
    loading: false,
    guideLoading: false,
    error: null,
    acknowledged: false,
    check: vi.fn(async () => redStatus()),
    loadGuide: vi.fn(async () => {
      const guide: InstallGuide = {
        platform: "macos",
        releaseNotesUrl: "https://github.com/github/spec-kit/releases",
        steps: [
          {
            title: "Install with Homebrew",
            description: "Homebrew is the supported distribution on macOS 12+.",
            command: "brew install github/tap/spec-kit",
          },
        ],
      };
      useEnvStore.setState({ guide });
      return guide;
    }),
    ...overrides,
  } as unknown as Partial<ReturnType<typeof useEnvStore.getState>>);
}

describe("EnvCheckDialog", () => {
  beforeEach(() => {
    stubStore();
  });

  it("renders the red badge and fetches the install guide when binary is missing", async () => {
    render(<EnvCheckDialog projectId={projectId} status={redStatus()} />);
    const dialog = screen.getByTestId("env-check-dialog");
    expect(dialog).toHaveAttribute("data-badge", "red");
    expect(screen.getByTestId("env-badge")).toHaveAttribute("data-color", "red");
    expect(
      screen.getByText(/spec-kit binary not found on path/i),
    ).toBeInTheDocument();
    // No "continue" button in the red state.
    expect(screen.queryByTestId("env-continue-button")).toBeNull();
    // Install guide is fetched and rendered.
    await waitFor(() => {
      expect(screen.getByTestId("env-install-guide")).toBeInTheDocument();
    });
    expect(useEnvStore.getState().loadGuide).toHaveBeenCalled();
  });

  it("renders the amber badge with a Continue anyway button for outdated installs", async () => {
    render(<EnvCheckDialog projectId={projectId} status={amberStatus()} />);
    const dialog = screen.getByTestId("env-check-dialog");
    expect(dialog).toHaveAttribute("data-badge", "amber");
    expect(screen.getByTestId("env-badge")).toHaveAttribute("data-color", "amber");
    expect(
      screen.getByText(/installed spec-kit 0\.5\.0 is older than latest known 0\.6\.0/i),
    ).toBeInTheDocument();
    const cont = screen.getByTestId("env-continue-button");
    expect(cont).toBeInTheDocument();
    await userEvent.click(cont);
    expect(useEnvStore.getState().acknowledged).toBe(true);
  });

  it("renders the green badge with a plain Continue button when everything is ready", () => {
    render(<EnvCheckDialog projectId={projectId} status={greenStatus()} />);
    const dialog = screen.getByTestId("env-check-dialog");
    expect(dialog).toHaveAttribute("data-badge", "green");
    expect(screen.getByTestId("env-badge")).toHaveAttribute("data-color", "green");
    expect(screen.getByTestId("env-continue-button")).toBeInTheDocument();
  });

  it("triggers a forced re-check when the Re-check button is pressed", async () => {
    const check = vi.fn(async () => redStatus());
    useEnvStore.setState({ check } as unknown as Partial<
      ReturnType<typeof useEnvStore.getState>
    >);
    render(<EnvCheckDialog projectId={projectId} status={redStatus()} />);
    await userEvent.click(screen.getByRole("button", { name: /re-check/i }));
    expect(check).toHaveBeenCalledWith(projectId, true);
  });
});
