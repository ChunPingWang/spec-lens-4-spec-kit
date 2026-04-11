/**
 * Task File Picker — presents the `tasks_scan_files` result as a checklist
 * so the user can explicitly pin one or more task files to tabs. FR-050
 * requires NO items to be pre-checked, and the confirm button stays
 * disabled until at least one file is selected.
 */
import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { useTasksStore } from "@/stores";
import type { TaskFileCandidate, Uuid } from "@/types/ipc";

interface TaskFilePickerProps {
  projectId: Uuid;
  onConfirm: (selected: string[]) => void | Promise<void>;
  onCancel?: () => void;
}

export function TaskFilePicker({ projectId, onConfirm, onCancel }: TaskFilePickerProps) {
  const { t } = useTranslation();
  const candidates = useTasksStore((s) => s.candidates);
  const loadingScan = useTasksStore((s) => s.loadingScan);
  const scan = useTasksStore((s) => s.scan);
  const [checked, setChecked] = useState<Set<string>>(new Set());
  const [manualPath, setManualPath] = useState<string>("");

  useEffect(() => {
    void scan(projectId);
  }, [projectId, scan]);

  function toggle(path: string) {
    setChecked((prev) => {
      const next = new Set(prev);
      if (next.has(path)) next.delete(path);
      else next.add(path);
      return next;
    });
  }

  function addManual() {
    const trimmed = manualPath.trim();
    if (!trimmed) return;
    setChecked((prev) => new Set(prev).add(trimmed));
    setManualPath("");
  }

  const sorted = useMemo(() => [...candidates], [candidates]);
  const canConfirm = checked.size > 0;

  return (
    <section
      aria-label={t("tasks.pickerTitle", { defaultValue: "Pick task files" })}
      className="flex h-full flex-col gap-3 p-3"
    >
      <header>
        <h3 className="text-sm font-semibold">
          {t("tasks.pickerTitle", { defaultValue: "Pick task files" })}
        </h3>
        <p className="text-xs text-muted-foreground">
          {t("tasks.pickerHint", {
            defaultValue:
              "Nothing is pre-selected — pick at least one file to open as a tab.",
          })}
        </p>
      </header>

      <div className="min-h-[120px] flex-1 overflow-auto rounded-md border border-border">
        {loadingScan ? (
          <p className="p-3 text-xs text-muted-foreground">
            {t("tasks.scanning", { defaultValue: "Scanning project…" })}
          </p>
        ) : sorted.length === 0 ? (
          <p className="p-3 text-xs text-muted-foreground">
            {t("tasks.noCandidates", {
              defaultValue:
                "No task files detected automatically. Paste a project-relative path below.",
            })}
          </p>
        ) : (
          <ul className="divide-y divide-border">
            {sorted.map((c: TaskFileCandidate) => {
              const id = `task-file-${c.relativePath}`;
              return (
                <li key={c.relativePath} className="flex items-center gap-2 p-2 text-sm">
                  <input
                    id={id}
                    type="checkbox"
                    checked={checked.has(c.relativePath)}
                    onChange={() => toggle(c.relativePath)}
                    aria-label={c.relativePath}
                  />
                  <label htmlFor={id} className="flex-1 cursor-pointer">
                    <span className="font-medium">{c.displayName}</span>
                    <span className="ml-2 text-xs text-muted-foreground">
                      {c.relativePath}
                    </span>
                  </label>
                  <span className="rounded-sm bg-muted px-1.5 py-0.5 text-[10px] uppercase text-muted-foreground">
                    {c.format}
                  </span>
                </li>
              );
            })}
          </ul>
        )}
      </div>

      <div className="flex items-center gap-2">
        <input
          type="text"
          placeholder={t("tasks.manualPathPlaceholder", {
            defaultValue: "Project-relative path (e.g. specs/001/tasks.md)",
          })}
          value={manualPath}
          onChange={(e) => setManualPath(e.target.value)}
          className="flex-1 rounded-sm border border-border bg-background px-2 py-1 text-xs"
          aria-label={t("tasks.manualPathLabel", { defaultValue: "Manual path" })}
        />
        <button
          type="button"
          onClick={addManual}
          disabled={!manualPath.trim()}
          className="rounded-sm border border-border px-2 py-1 text-xs hover:bg-accent disabled:cursor-not-allowed disabled:opacity-50"
        >
          {t("tasks.addManual", { defaultValue: "Add path" })}
        </button>
      </div>

      <div className="flex items-center justify-end gap-2">
        {onCancel && (
          <button
            type="button"
            onClick={onCancel}
            className="rounded-sm border border-border px-3 py-1 text-xs hover:bg-accent"
          >
            {t("tasks.cancel", { defaultValue: "Cancel" })}
          </button>
        )}
        <button
          type="button"
          onClick={() => {
            void onConfirm(Array.from(checked));
          }}
          disabled={!canConfirm}
          data-testid="task-picker-confirm"
          className="rounded-sm bg-primary px-3 py-1 text-xs text-primary-foreground disabled:cursor-not-allowed disabled:opacity-50"
        >
          {t("tasks.confirm", { defaultValue: "Open selected" })}
        </button>
      </div>
    </section>
  );
}
