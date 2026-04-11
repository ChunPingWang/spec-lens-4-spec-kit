/**
 * T090 — Vitest coverage for `TerminalPanel`. We cannot easily instantiate
 * xterm in jsdom, so the tests exercise the sr-only `<ul>` mirror the
 * component renders as a fallback: timestamps, severity attrs, auto-scroll
 * checkbox, and the connect-button guard when no project is open.
 */
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import "@/i18n";

import { TerminalPanel } from "./TerminalPanel";
import { useTerminalStore } from "@/stores";
import type { OutputLine, PtySessionDescriptor } from "@/types/ipc";

vi.mock("@/hooks/usePtySession", () => ({
  usePtySession: () => {},
}));

// Stub xterm + fit addon so the lazy dynamic import resolves synchronously
// without touching jsdom's DOM (xterm's `open()` throws in jsdom). The
// component gracefully falls back to the sr-only `<ul>` mirror, which is
// exactly what these tests assert against.
vi.mock("@xterm/xterm", () => ({
  Terminal: class {
    loadAddon() {}
    open() {}
    write() {}
    dispose() {}
  },
}));
vi.mock("@xterm/addon-fit", () => ({
  FitAddon: class {
    fit() {}
  },
}));

const projectId = "00000000-0000-0000-0000-000000000001";

const descriptor: PtySessionDescriptor = {
  sessionId: "00000000-0000-0000-0000-0000000000aa",
  windowId: "win-1",
  projectId,
  cwd: "/tmp/project",
  shell: "/bin/zsh",
  startedAt: "2026-04-11T00:00:00Z",
};

function makeLine(seq: number, text: string, severity?: "error" | "warn"): OutputLine {
  // 2026-04-11T01:02:03.456Z → 01:02:03.456 (display uses local time, but
  // both components share the same Date so the assertion still matches).
  const ts = Date.UTC(2026, 3, 11, 1, 2, 3, 456);
  return {
    seq,
    ts,
    stream: "stdout",
    text,
    highlight: severity ? { ruleId: `${severity}-rule`, severity } : undefined,
  };
}

function resetTerminalStore() {
  useTerminalStore.setState({
    descriptor: null,
    connecting: false,
    error: null,
    recentLines: [],
  });
}

describe("TerminalPanel", () => {
  beforeEach(() => {
    resetTerminalStore();
  });

  it("disables the connect button when no project is open", () => {
    render(<TerminalPanel projectId={null} />);
    const connect = screen.getByRole("button", { name: /connect/i });
    expect(connect).toBeDisabled();
  });

  it("shows the idle status text before a descriptor exists", () => {
    render(<TerminalPanel projectId={projectId} />);
    expect(screen.getByText(/not connected/i)).toBeInTheDocument();
  });

  it("mirrors recent lines with timestamps and severity markers", () => {
    useTerminalStore.setState({
      descriptor,
      recentLines: [
        makeLine(1, "hello world"),
        makeLine(2, "ERROR: boom", "error"),
      ],
    });
    render(<TerminalPanel projectId={projectId} />);

    const mirror = screen.getByTestId("terminal-line-mirror");
    const items = Array.from(mirror.querySelectorAll("li"));
    expect(items).toHaveLength(2);
    expect(items[0]?.getAttribute("data-seq")).toBe("1");
    expect(items[1]?.getAttribute("data-severity")).toBe("error");

    // Every mirrored line exposes its formatted HH:MM:SS.mmm stamp so the
    // overview slice and screen readers can render timestamps consistently.
    const stamps = Array.from(mirror.querySelectorAll('[data-testid="terminal-line-ts"]'));
    expect(stamps).toHaveLength(2);
    expect(stamps[0]?.textContent).toMatch(/^\d{2}:\d{2}:\d{2}\.\d{3}$/);
  });

  it("toggles the auto-scroll checkbox", async () => {
    const user = userEvent.setup();
    render(<TerminalPanel projectId={projectId} />);
    const toggle = screen.getByRole("checkbox", { name: /auto-scroll/i });
    expect(toggle).toBeChecked();
    await user.click(toggle);
    expect(toggle).not.toBeChecked();
  });

  it("swaps to the disconnect button once a descriptor is set", () => {
    useTerminalStore.setState({ descriptor });
    render(<TerminalPanel projectId={projectId} />);
    expect(screen.getByRole("button", { name: /disconnect/i })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /^connect$/i })).not.toBeInTheDocument();
  });
});
