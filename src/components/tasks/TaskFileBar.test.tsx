/**
 * T109 — Vitest coverage for TaskFileBar asserting the 0 / 1 / many
 * rendering rules plus parent-directory disambiguation (FR-053).
 */
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import "@/i18n";

import { TaskFileBar } from "./TaskFileBar";
import type { ParsedTaskFile } from "@/stores/tasksStore";

function makeParsed(relativePath: string, total: number, done: number): ParsedTaskFile {
  return {
    file: {
      relativePath,
      format: "md",
      displayName: relativePath.split("/").pop() ?? relativePath,
      taskCount: total,
      completedCount: done,
      parsedAt: "2026-04-11T00:00:00Z",
    },
    entries: [],
  };
}

describe("TaskFileBar", () => {
  it("renders nothing but the add-file control when selection is empty", () => {
    render(
      <TaskFileBar
        selectedPaths={[]}
        activePath={null}
        parsed={{}}
        onSelectTab={() => {}}
        onCloseTab={() => {}}
        onAddFile={() => {}}
      />,
    );
    expect(screen.getByText(/no task files selected/i)).toBeInTheDocument();
    expect(screen.getByTestId("task-file-bar-add")).toBeInTheDocument();
    expect(screen.queryAllByRole("tab")).toHaveLength(0);
  });

  it("renders a single tab with the completion badge when one file is selected", () => {
    const parsed = { "specs/001/tasks.md": makeParsed("specs/001/tasks.md", 10, 4) };
    render(
      <TaskFileBar
        selectedPaths={["specs/001/tasks.md"]}
        activePath="specs/001/tasks.md"
        parsed={parsed}
        onSelectTab={() => {}}
        onCloseTab={() => {}}
        onAddFile={() => {}}
      />,
    );
    const tabs = screen.getAllByRole("tab");
    expect(tabs).toHaveLength(1);
    expect(tabs[0]).toHaveAttribute("aria-selected", "true");
    expect(screen.getByText("4/10")).toBeInTheDocument();
  });

  it("disambiguates clashing basenames by prepending the parent directory", () => {
    const parsed = {
      "specs/001-foo/tasks.md": makeParsed("specs/001-foo/tasks.md", 2, 1),
      "specs/002-bar/tasks.md": makeParsed("specs/002-bar/tasks.md", 3, 0),
    };
    render(
      <TaskFileBar
        selectedPaths={["specs/001-foo/tasks.md", "specs/002-bar/tasks.md"]}
        activePath="specs/001-foo/tasks.md"
        parsed={parsed}
        onSelectTab={() => {}}
        onCloseTab={() => {}}
        onAddFile={() => {}}
      />,
    );
    expect(screen.getByText("001-foo/tasks.md")).toBeInTheDocument();
    expect(screen.getByText("002-bar/tasks.md")).toBeInTheDocument();
  });

  it("invokes onSelectTab and onCloseTab callbacks", async () => {
    const onSelect = vi.fn();
    const onClose = vi.fn();
    const parsed = { "specs/001/tasks.md": makeParsed("specs/001/tasks.md", 2, 1) };
    render(
      <TaskFileBar
        selectedPaths={["specs/001/tasks.md"]}
        activePath="specs/001/tasks.md"
        parsed={parsed}
        onSelectTab={onSelect}
        onCloseTab={onClose}
        onAddFile={() => {}}
      />,
    );
    await userEvent.click(screen.getByTestId("task-tab-specs/001/tasks.md"));
    expect(onSelect).toHaveBeenCalledWith("specs/001/tasks.md");
    await userEvent.click(screen.getByLabelText(/Close specs\/001\/tasks\.md/));
    expect(onClose).toHaveBeenCalledWith("specs/001/tasks.md");
  });
});
