/**
 * TaskDetail — expandable detail slice rendered under a TaskItem when
 * the user clicks to expand. Shows:
 *   - the raw line from the source file;
 *   - the breadcrumb section path so users can see which heading the
 *     task lives under;
 *   - the most recent 30 s of terminal output from the mirrored
 *     `recentLines` buffer as an approximation of "≈ 30 s before
 *     completion" (full `terminal_get_slice` wiring lands alongside
 *     step-to-output routing, T097).
 *
 * This component is intentionally presentation-only — TaskItem passes
 * the already-filtered terminal lines down so tests can render it in
 * isolation.
 */
import { useTranslation } from "react-i18next";

import type { OutputLine, TaskEntry } from "@/types/ipc";

interface TaskDetailProps {
  task: TaskEntry;
  terminalSlice: OutputLine[];
  errorSummary?: string;
}

export function TaskDetail({ task, terminalSlice, errorSummary }: TaskDetailProps) {
  const { t } = useTranslation();
  const breadcrumb = task.sectionPath.length > 0 ? task.sectionPath.join(" › ") : null;

  return (
    <div className="space-y-2 border-t border-border bg-muted/30 p-3 text-xs">
      <div>
        <div className="font-medium text-muted-foreground">
          {t("tasks.detail.description", { defaultValue: "Description" })}
        </div>
        <pre className="whitespace-pre-wrap break-words font-mono text-xs">{task.raw}</pre>
      </div>

      {breadcrumb && (
        <div>
          <span className="font-medium text-muted-foreground">
            {t("tasks.detail.section", { defaultValue: "Section" })}:
          </span>{" "}
          <span>{breadcrumb}</span>
        </div>
      )}

      <div>
        <div className="font-medium text-muted-foreground">
          {t("tasks.detail.filePath", { defaultValue: "File" })}
        </div>
        <code className="text-[11px]">
          {task.filePath}:{task.line}
        </code>
      </div>

      <div>
        <div className="font-medium text-muted-foreground">
          {t("tasks.detail.terminalSlice", {
            defaultValue: "≈ 30 s before completion",
          })}
        </div>
        {terminalSlice.length === 0 ? (
          <p className="text-muted-foreground">
            {t("tasks.detail.noSlice", {
              defaultValue: "No terminal output captured for this task yet.",
            })}
          </p>
        ) : (
          <pre className="max-h-40 overflow-auto whitespace-pre-wrap break-words font-mono text-[11px] leading-relaxed">
            {terminalSlice.map((l) => l.text).join("\n")}
          </pre>
        )}
      </div>

      {errorSummary && (
        <div className="rounded-sm border border-destructive/50 bg-destructive/10 p-2 text-destructive">
          <div className="font-medium">
            {t("tasks.detail.errorSummary", { defaultValue: "Error summary" })}
          </div>
          <p>{errorSummary}</p>
        </div>
      )}
    </div>
  );
}
