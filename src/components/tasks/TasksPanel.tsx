/**
 * TasksPanel — composes the US4 flow:
 *
 *   1. If the user has never selected task files for this project,
 *      show the `TaskFilePicker` (FR-050 — no pre-selection).
 *   2. Otherwise show the `TaskFileBar`, a combined + per-file
 *      `TasksProgressPanel`, a per-tab `TaskFilter`, and a scrollable
 *      list of `TaskItem` rows.
 *   3. The "+ add file" control re-shows the picker in "add mode" so
 *      existing selections are preserved (FR-054).
 *
 * Filter / search state is kept per tab (per FR-056) in a local map
 * keyed by relative path.
 */
import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { TaskDetail } from "./TaskDetail";
import { TaskFileBar } from "./TaskFileBar";
import { TaskFilePicker } from "./TaskFilePicker";
import { TaskFilter, type TaskFilterMode } from "./TaskFilter";
import { TaskItem } from "./TaskItem";
import { TasksProgressPanel } from "./TasksProgressPanel";
import { useProjectStore, useTasksStore, useTerminalStore } from "@/stores";
import type { OutputLine, TaskEntry, Uuid } from "@/types/ipc";

interface TasksPanelProps {
  projectId: Uuid;
}

interface TabFilterState {
  mode: TaskFilterMode;
  query: string;
}

const DEFAULT_FILTER: TabFilterState = { mode: "all", query: "" };

function sortIncompleteFirst(entries: TaskEntry[]): TaskEntry[] {
  return [...entries].sort((a, b) => {
    const aDone = a.status === "done" ? 1 : 0;
    const bDone = b.status === "done" ? 1 : 0;
    if (aDone !== bDone) return aDone - bDone;
    return a.line - b.line;
  });
}

function applyFilter(entries: TaskEntry[], state: TabFilterState): TaskEntry[] {
  const q = state.query.trim().toLowerCase();
  return entries.filter((e) => {
    if (state.mode === "completed" && e.status !== "done") return false;
    if (state.mode === "incomplete" && e.status === "done") return false;
    if (q && !e.title.toLowerCase().includes(q)) return false;
    return true;
  });
}

/** Return the `recentLines` entries whose timestamp falls within 30 s
 * of the most recent line. Used as a placeholder for the future
 * `terminal_get_slice`-backed "≈ 30 s before completion" view. */
function recentSlice(lines: OutputLine[]): OutputLine[] {
  if (lines.length === 0) return [];
  const last = lines[lines.length - 1];
  if (!last) return [];
  const cutoff = last.ts - 30_000;
  return lines.filter((l) => l.ts >= cutoff);
}

export function TasksPanel({ projectId }: TasksPanelProps) {
  const { t } = useTranslation();
  const project = useProjectStore((s) => s.current);
  const selectedPaths = useTasksStore((s) => s.selectedPaths);
  const activePath = useTasksStore((s) => s.activePath);
  const parsed = useTasksStore((s) => s.parsed);
  const hydrateFromProjectState = useTasksStore((s) => s.hydrateFromProjectState);
  const setSelected = useTasksStore((s) => s.setSelected);
  const closeTab = useTasksStore((s) => s.closeTab);
  const parse = useTasksStore((s) => s.parse);
  const recentLines = useTerminalStore((s) => s.recentLines);

  const [addingFile, setAddingFile] = useState(false);
  const [expandedTaskId, setExpandedTaskId] = useState<string | null>(null);
  const [filterByPath, setFilterByPath] = useState<Record<string, TabFilterState>>({});

  // Hydrate persisted tab state from the currently-open project.
  useEffect(() => {
    if (project?.state) {
      hydrateFromProjectState(project.state);
    }
  }, [project, hydrateFromProjectState]);

  // Eagerly parse any selected files that haven't been parsed yet (e.g.
  // after hydrating from persisted state).
  useEffect(() => {
    for (const path of selectedPaths) {
      if (!parsed[path]) {
        void parse(projectId, path).catch(() => undefined);
      }
    }
  }, [selectedPaths, parsed, parse, projectId]);

  const activeFile = activePath ? (parsed[activePath] ?? null) : null;
  const currentFilter = activePath
    ? (filterByPath[activePath] ?? DEFAULT_FILTER)
    : DEFAULT_FILTER;

  const visibleEntries = useMemo(() => {
    if (!activeFile) return [] as TaskEntry[];
    return applyFilter(sortIncompleteFirst(activeFile.entries), currentFilter);
  }, [activeFile, currentFilter]);

  const parsedFiles = useMemo(
    () => selectedPaths.map((p) => parsed[p]).filter((p): p is NonNullable<typeof p> => !!p),
    [selectedPaths, parsed],
  );

  const terminalSlice = useMemo(() => recentSlice(recentLines), [recentLines]);

  const showingPicker = selectedPaths.length === 0 || addingFile;

  async function handlePickerConfirm(newPaths: string[]) {
    const merged = Array.from(new Set([...selectedPaths, ...newPaths]));
    const nextActive = activePath ?? newPaths[0] ?? merged[0] ?? null;
    await setSelected(projectId, merged, nextActive);
    setAddingFile(false);
  }

  function handleFilterChange(mode: TaskFilterMode) {
    if (!activePath) return;
    setFilterByPath((prev) => ({
      ...prev,
      [activePath]: { ...(prev[activePath] ?? DEFAULT_FILTER), mode },
    }));
  }

  function handleQueryChange(query: string) {
    if (!activePath) return;
    setFilterByPath((prev) => ({
      ...prev,
      [activePath]: { ...(prev[activePath] ?? DEFAULT_FILTER), query },
    }));
  }

  async function handleSelectTab(path: string) {
    await setSelected(projectId, selectedPaths, path);
    setExpandedTaskId(null);
  }

  async function handleCloseTab(path: string) {
    await closeTab(projectId, path);
  }

  if (showingPicker) {
    return (
      <div className="flex h-full flex-col">
        <TaskFileBar
          selectedPaths={selectedPaths}
          activePath={activePath}
          parsed={parsed}
          onSelectTab={(p) => void handleSelectTab(p)}
          onCloseTab={(p) => void handleCloseTab(p)}
          onAddFile={() => setAddingFile(true)}
        />
        <TaskFilePicker
          projectId={projectId}
          onConfirm={handlePickerConfirm}
          onCancel={addingFile ? () => setAddingFile(false) : undefined}
        />
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col">
      <TaskFileBar
        selectedPaths={selectedPaths}
        activePath={activePath}
        parsed={parsed}
        onSelectTab={(p) => void handleSelectTab(p)}
        onCloseTab={(p) => void handleCloseTab(p)}
        onAddFile={() => setAddingFile(true)}
      />
      <TasksProgressPanel files={parsedFiles} activeFile={activeFile} />
      <TaskFilter
        mode={currentFilter.mode}
        query={currentFilter.query}
        onModeChange={handleFilterChange}
        onQueryChange={handleQueryChange}
      />
      <div className="flex-1 overflow-auto">
        {activeFile ? (
          visibleEntries.length === 0 ? (
            <p className="p-3 text-xs text-muted-foreground">
              {t("tasks.empty", { defaultValue: "No tasks match the current filter." })}
            </p>
          ) : (
            <ul className="divide-y divide-border">
              {visibleEntries.map((task) => {
                const expanded = expandedTaskId === task.id;
                return (
                  <div key={task.id} className="px-3 py-2">
                    <TaskItem
                      task={task}
                      expanded={expanded}
                      onToggleExpand={() =>
                        setExpandedTaskId((prev) => (prev === task.id ? null : task.id))
                      }
                    />
                    {expanded && <TaskDetail task={task} terminalSlice={terminalSlice} />}
                  </div>
                );
              })}
            </ul>
          )
        ) : (
          <p className="p-3 text-xs text-muted-foreground">
            {t("tasks.noActive", { defaultValue: "Select a task file tab above." })}
          </p>
        )}
      </div>
    </div>
  );
}
