/**
 * T111 — Vitest coverage for TasksProgressPanel asserting the FR-055
 * color thresholds plus the combined + per-file bar rendering rule.
 */
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import "@/i18n";

import { TasksProgressPanel } from "./TasksProgressPanel";
import type { ParsedTaskFile } from "@/stores/tasksStore";

function makeParsed(
  relativePath: string,
  total: number,
  done: number,
  displayName?: string,
): ParsedTaskFile {
  return {
    file: {
      relativePath,
      format: "md",
      displayName: displayName ?? relativePath,
      taskCount: total,
      completedCount: done,
      parsedAt: "2026-04-11T00:00:00Z",
    },
    entries: [],
  };
}

describe("TasksProgressPanel", () => {
  it("renders nothing when no files are loaded", () => {
    const { container } = render(<TasksProgressPanel files={[]} activeFile={null} />);
    expect(container).toBeEmptyDOMElement();
  });

  it("uses amber under 50%", () => {
    const file = makeParsed("a.md", 10, 2);
    render(<TasksProgressPanel files={[file]} activeFile={file} />);
    const bar = screen.getByTestId("tasks-progress-combined");
    expect(bar).toHaveAttribute("data-percent", "20");
    expect(bar).toHaveAttribute("data-color", "bg-amber-500");
  });

  it("uses blue at 50–79%", () => {
    const file = makeParsed("a.md", 10, 6);
    render(<TasksProgressPanel files={[file]} activeFile={file} />);
    const bar = screen.getByTestId("tasks-progress-combined");
    expect(bar).toHaveAttribute("data-percent", "60");
    expect(bar).toHaveAttribute("data-color", "bg-blue-500");
  });

  it("uses green at 80% and above", () => {
    const file = makeParsed("a.md", 10, 9);
    render(<TasksProgressPanel files={[file]} activeFile={file} />);
    const bar = screen.getByTestId("tasks-progress-combined");
    expect(bar).toHaveAttribute("data-percent", "90");
    expect(bar).toHaveAttribute("data-color", "bg-green-500");
  });

  it("shows only a single combined bar when exactly one file is loaded", () => {
    const file = makeParsed("a.md", 4, 2);
    render(<TasksProgressPanel files={[file]} activeFile={file} />);
    expect(screen.getByTestId("tasks-progress-combined")).toBeInTheDocument();
    expect(screen.queryByTestId("tasks-progress-current")).not.toBeInTheDocument();
  });

  it("shows combined + per-current-file bars when more than one file is loaded", () => {
    const f1 = makeParsed("a.md", 10, 5);
    const f2 = makeParsed("b.md", 10, 10);
    render(<TasksProgressPanel files={[f1, f2]} activeFile={f2} />);
    const combined = screen.getByTestId("tasks-progress-combined");
    const current = screen.getByTestId("tasks-progress-current");
    // 15/20 = 75% → blue
    expect(combined).toHaveAttribute("data-percent", "75");
    expect(combined).toHaveAttribute("data-color", "bg-blue-500");
    // active = b.md → 10/10 = 100% → green
    expect(current).toHaveAttribute("data-percent", "100");
    expect(current).toHaveAttribute("data-color", "bg-green-500");
  });
});
