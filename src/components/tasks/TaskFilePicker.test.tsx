/**
 * T110 — Vitest coverage for `TaskFilePicker`. Locks in FR-050:
 *   - no candidates are pre-checked on mount;
 *   - the confirm button is disabled until ≥ 1 selection exists;
 *   - clicking a checkbox enables confirm and forwards the selected
 *     paths to `onConfirm`.
 */
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import "@/i18n";

import { TaskFilePicker } from "./TaskFilePicker";
import { useTasksStore } from "@/stores";
import type { TaskFileCandidate } from "@/types/ipc";

const projectId = "00000000-0000-0000-0000-000000000001";

const candidates: TaskFileCandidate[] = [
  { relativePath: "specs/001-foo/tasks.md", displayName: "tasks.md", format: "md" },
  { relativePath: "specs/002-bar/tasks.json", displayName: "tasks.json", format: "json" },
];

function resetStore() {
  useTasksStore.setState({
    candidates,
    parsed: {},
    selectedPaths: [],
    activePath: null,
    loadingScan: false,
    loadingParse: {},
    error: null,
  });
  // Stub the async scan so the mount effect resolves without hitting IPC.
  useTasksStore.setState({
    scan: async () => candidates,
  } as unknown as Partial<ReturnType<typeof useTasksStore.getState>>);
}

describe("TaskFilePicker", () => {
  beforeEach(() => {
    resetStore();
  });

  it("starts with nothing pre-checked and the confirm button disabled", () => {
    render(<TaskFilePicker projectId={projectId} onConfirm={() => {}} />);
    const checkboxes = screen.getAllByRole("checkbox");
    expect(checkboxes).toHaveLength(candidates.length);
    for (const cb of checkboxes) {
      expect(cb).not.toBeChecked();
    }
    expect(screen.getByTestId("task-picker-confirm")).toBeDisabled();
  });

  it("enables confirm once a candidate is checked and forwards the selection", async () => {
    const onConfirm = vi.fn();
    render(<TaskFilePicker projectId={projectId} onConfirm={onConfirm} />);
    const first = screen.getAllByRole("checkbox")[0]!;
    await userEvent.click(first);
    const confirm = screen.getByTestId("task-picker-confirm");
    expect(confirm).not.toBeDisabled();
    await userEvent.click(confirm);
    expect(onConfirm).toHaveBeenCalledWith(["specs/001-foo/tasks.md"]);
  });

  it("lets the user add a manual project-relative path", async () => {
    const onConfirm = vi.fn();
    render(<TaskFilePicker projectId={projectId} onConfirm={onConfirm} />);
    const input = screen.getByLabelText(/manual path/i);
    await userEvent.type(input, "specs/999-manual/tasks.md");
    await userEvent.click(screen.getByRole("button", { name: /add path/i }));
    await userEvent.click(screen.getByTestId("task-picker-confirm"));
    expect(onConfirm).toHaveBeenCalledWith(["specs/999-manual/tasks.md"]);
  });
});
