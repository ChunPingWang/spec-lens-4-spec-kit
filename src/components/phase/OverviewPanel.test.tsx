/**
 * T102 — Vitest coverage for the terminal slice integration in OverviewPanel.
 * Asserts that when the store has recentLines matching the active step, a
 * collapsible terminal slice section is rendered (FR-033).
 */
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import "@/i18n";

import { OverviewPanel } from "./OverviewPanel";
import { usePhaseStore } from "@/stores";
import { useTerminalStore } from "@/stores";
import type { OutputLine, PtySessionDescriptor } from "@/types/ipc";

vi.mock("@/lib/tauri", () => ({
  invoke: vi.fn().mockResolvedValue({ stepId: "step-specify", slices: [] }),
  SpecLensIpcError: class extends Error {
    code = "E_TEST";
  },
}));

const projectId = "00000000-0000-0000-0000-000000000001";
const stepId = "step-specify";

const descriptor: PtySessionDescriptor = {
  sessionId: "00000000-0000-0000-0000-0000000000aa",
  windowId: "win-1",
  projectId,
  cwd: "/tmp/project",
  shell: "/bin/zsh",
  startedAt: "2026-04-11T00:00:00Z",
};

function makeLine(seq: number, text: string, sid?: string): OutputLine {
  return {
    seq,
    ts: Date.now(),
    stream: "stdout",
    text,
    stepId: sid,
  };
}

describe("OverviewPanel — terminal slice (T102)", () => {
  beforeEach(() => {
    // Pre-set the store so loadOverview's effect doesn't trigger loading state
    usePhaseStore.setState({
      overview: { stepId, slices: [] },
      loadingOverview: false,
      loadOverview: vi.fn(),
    });
    useTerminalStore.setState({
      descriptor: null,
      recentLines: [],
    });
  });

  it("shows a 'no terminal output' message when there are no lines for the step", () => {
    render(<OverviewPanel projectId={projectId} stepId={stepId} />);
    expect(screen.getByText(/no terminal output/i)).toBeInTheDocument();
  });

  it("renders terminal lines filtered by the active step", () => {
    useTerminalStore.setState({
      descriptor,
      recentLines: [
        makeLine(1, "unrelated line", "step-other"),
        makeLine(2, "step output line 1", stepId),
        makeLine(3, "step output line 2", stepId),
      ],
    });
    render(<OverviewPanel projectId={projectId} stepId={stepId} />);
    expect(screen.getByText(/step output line 1/)).toBeInTheDocument();
    expect(screen.getByText(/step output line 2/)).toBeInTheDocument();
    expect(screen.queryByText(/unrelated line/)).not.toBeInTheDocument();
  });

  it("the terminal slice section is collapsible", async () => {
    const user = userEvent.setup();
    useTerminalStore.setState({
      descriptor,
      recentLines: [makeLine(1, "visible line", stepId)],
    });
    render(<OverviewPanel projectId={projectId} stepId={stepId} />);

    // The section should have a disclosure button
    const toggle = screen.getByRole("button", { name: /terminal/i });
    expect(toggle).toBeInTheDocument();

    // Lines visible initially (expanded by default)
    expect(screen.getByText(/visible line/)).toBeInTheDocument();

    // Collapse
    await user.click(toggle);
    expect(screen.queryByText(/visible line/)).not.toBeInTheDocument();
  });

  it("renders the terminal slice heading with line count", () => {
    useTerminalStore.setState({
      descriptor,
      recentLines: [
        makeLine(1, "line A", stepId),
        makeLine(2, "line B", stepId),
      ],
    });
    render(<OverviewPanel projectId={projectId} stepId={stepId} />);
    expect(screen.getByText(/terminal.*2/i)).toBeInTheDocument();
  });
});
