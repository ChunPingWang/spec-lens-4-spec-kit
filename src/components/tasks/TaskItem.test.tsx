/**
 * T112 — Vitest coverage for TaskItem + TaskDetail asserting expansion
 * shows description, section breadcrumb, the "≈ 30 s before completion"
 * terminal slice, and the error summary block.
 */
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import "@/i18n";

import { TaskDetail } from "./TaskDetail";
import { TaskItem } from "./TaskItem";
import type { OutputLine, TaskEntry } from "@/types/ipc";

function makeTask(overrides: Partial<TaskEntry> = {}): TaskEntry {
  return {
    id: "abc123:7",
    title: "Install dependencies",
    status: "todo",
    sectionPath: ["Phase 1: Setup"],
    filePath: "specs/001/tasks.md",
    line: 7,
    raw: "- [ ] T001 Install dependencies",
    ...overrides,
  };
}

function makeLine(seq: number, text: string, ts: number): OutputLine {
  return { seq, ts, stream: "stdout", text };
}

describe("TaskItem", () => {
  it("toggles expansion when clicked", async () => {
    const toggle = vi.fn();
    render(<TaskItem task={makeTask()} expanded={false} onToggleExpand={toggle} />);
    const row = screen.getByRole("button", { name: /install dependencies/i });
    expect(row).toHaveAttribute("aria-expanded", "false");
    await userEvent.click(row);
    expect(toggle).toHaveBeenCalledTimes(1);
  });

  it("shows the status label as the icon's aria-label", () => {
    render(
      <TaskItem task={makeTask({ status: "done" })} expanded={false} onToggleExpand={() => {}} />,
    );
    expect(screen.getByLabelText(/done/i)).toBeInTheDocument();
  });
});

describe("TaskDetail", () => {
  it("renders the raw description, breadcrumb, and file path on expand", () => {
    const task = makeTask({ sectionPath: ["Phase 1", "Setup"] });
    render(<TaskDetail task={task} terminalSlice={[]} />);
    expect(screen.getByText(/install dependencies/i)).toBeInTheDocument();
    expect(screen.getByText(/Phase 1 › Setup/)).toBeInTheDocument();
    expect(screen.getByText(/specs\/001\/tasks\.md:7/)).toBeInTheDocument();
    expect(screen.getByText(/No terminal output captured/i)).toBeInTheDocument();
  });

  it("renders the ≈ 30 s terminal slice when lines are provided", () => {
    const lines = [
      makeLine(1, "running step", 1_000),
      makeLine(2, "step finished", 2_000),
    ];
    render(<TaskDetail task={makeTask()} terminalSlice={lines} />);
    expect(screen.getByText(/≈ 30 s before completion/i)).toBeInTheDocument();
    expect(screen.getByText(/running step/)).toBeInTheDocument();
    expect(screen.getByText(/step finished/)).toBeInTheDocument();
  });

  it("renders the error summary block when provided", () => {
    render(
      <TaskDetail
        task={makeTask()}
        terminalSlice={[]}
        errorSummary="Command exited with code 1"
      />,
    );
    expect(screen.getByText(/error summary/i)).toBeInTheDocument();
    expect(screen.getByText(/Command exited with code 1/)).toBeInTheDocument();
  });
});
