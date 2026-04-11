/**
 * Always-visible Task File Bar (FR-053 / FR-054). Renders one tab per
 * selected task file with:
 *   - filename, disambiguated by parent directory when two tabs share a
 *     basename;
 *   - a completion badge `{done}/{total}`;
 *   - a close button that calls `closeTab`;
 *   - a trailing "+ add file" control that lets the user append more
 *     files via the picker (FR-054).
 *
 * The bar MUST render even when no files are selected so that the user
 * can always access the "+ add file" control (FR-053).
 */
import { useMemo } from "react";
import { useTranslation } from "react-i18next";

import type { ParsedTaskFile } from "@/stores/tasksStore";

interface TaskFileBarProps {
  selectedPaths: string[];
  activePath: string | null;
  parsed: Record<string, ParsedTaskFile>;
  onSelectTab: (path: string) => void;
  onCloseTab: (path: string) => void;
  onAddFile: () => void;
}

function basename(path: string): string {
  const parts = path.split("/");
  return parts[parts.length - 1] ?? path;
}

function parentDir(path: string): string {
  const parts = path.split("/");
  return parts.length >= 2 ? (parts[parts.length - 2] ?? "") : "";
}

/** Decide the user-visible label for each tab, disambiguating clashing
 * basenames by prepending the parent directory. */
function computeLabels(paths: string[]): Record<string, string> {
  const counts = new Map<string, number>();
  for (const p of paths) {
    const b = basename(p);
    counts.set(b, (counts.get(b) ?? 0) + 1);
  }
  const labels: Record<string, string> = {};
  for (const p of paths) {
    const b = basename(p);
    if ((counts.get(b) ?? 0) > 1) {
      const parent = parentDir(p);
      labels[p] = parent ? `${parent}/${b}` : b;
    } else {
      labels[p] = b;
    }
  }
  return labels;
}

export function TaskFileBar({
  selectedPaths,
  activePath,
  parsed,
  onSelectTab,
  onCloseTab,
  onAddFile,
}: TaskFileBarProps) {
  const { t } = useTranslation();
  const labels = useMemo(() => computeLabels(selectedPaths), [selectedPaths]);

  return (
    <div
      role="tablist"
      aria-label={t("tasks.fileBarLabel", { defaultValue: "Task files" })}
      className="flex items-center gap-1 overflow-x-auto border-b border-border bg-background/50 px-2 py-1"
      data-testid="task-file-bar"
    >
      {selectedPaths.length === 0 ? (
        <span className="px-2 text-xs text-muted-foreground">
          {t("tasks.noTabs", { defaultValue: "No task files selected." })}
        </span>
      ) : (
        selectedPaths.map((path) => {
          const parsedFile = parsed[path];
          const total = parsedFile?.file.taskCount ?? 0;
          const done = parsedFile?.file.completedCount ?? 0;
          const active = path === activePath;
          return (
            <div
              key={path}
              role="tab"
              aria-selected={active}
              className={`flex items-center gap-1 rounded-sm border px-2 py-1 text-xs ${
                active
                  ? "border-primary bg-muted text-foreground"
                  : "border-border text-muted-foreground hover:text-foreground"
              }`}
            >
              <button
                type="button"
                onClick={() => onSelectTab(path)}
                title={path}
                className="max-w-[180px] truncate text-left"
                data-testid={`task-tab-${path}`}
              >
                {labels[path]}
              </button>
              <span
                className="rounded-sm bg-background px-1 text-[10px] text-muted-foreground"
                aria-label={t("tasks.completionBadge", {
                  defaultValue: "{{done}} of {{total}} complete",
                  done,
                  total,
                })}
              >
                {done}/{total}
              </span>
              <button
                type="button"
                onClick={(e) => {
                  e.stopPropagation();
                  onCloseTab(path);
                }}
                aria-label={t("tasks.closeTab", {
                  defaultValue: "Close {{path}}",
                  path,
                })}
                className="rounded-sm px-1 text-muted-foreground hover:bg-accent hover:text-foreground"
              >
                ×
              </button>
            </div>
          );
        })
      )}

      <button
        type="button"
        onClick={onAddFile}
        data-testid="task-file-bar-add"
        className="ml-auto rounded-sm border border-dashed border-border px-2 py-1 text-xs text-muted-foreground hover:bg-accent hover:text-foreground"
      >
        {t("tasks.addFile", { defaultValue: "+ add file" })}
      </button>
    </div>
  );
}
