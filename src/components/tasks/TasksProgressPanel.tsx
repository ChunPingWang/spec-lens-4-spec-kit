/**
 * TasksProgressPanel — per FR-055:
 *   - a single bar when exactly one file is loaded;
 *   - a combined bar + per-current-file bar when more than one file is
 *     loaded;
 *   - color bands: amber 0–49%, blue 50–79%, green 80–100%.
 */
import { useTranslation } from "react-i18next";

import type { ParsedTaskFile } from "@/stores/tasksStore";

interface TasksProgressPanelProps {
  files: ParsedTaskFile[];
  activeFile: ParsedTaskFile | null;
}

function colorForPercent(pct: number): string {
  if (pct >= 80) return "bg-green-500";
  if (pct >= 50) return "bg-blue-500";
  return "bg-amber-500";
}

function percent(done: number, total: number): number {
  if (total === 0) return 0;
  return Math.round((done * 100) / total);
}

interface BarProps {
  label: string;
  done: number;
  total: number;
  testId: string;
}

function Bar({ label, done, total, testId }: BarProps) {
  const pct = percent(done, total);
  const color = colorForPercent(pct);
  return (
    <div className="space-y-1" data-testid={testId} data-percent={pct} data-color={color}>
      <div className="flex items-center justify-between text-[11px] text-muted-foreground">
        <span>{label}</span>
        <span>
          {done}/{total} ({pct}%)
        </span>
      </div>
      <div
        role="progressbar"
        aria-valuenow={pct}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-label={label}
        className="h-2 w-full overflow-hidden rounded-sm bg-muted"
      >
        <div className={`h-full ${color}`} style={{ width: `${pct}%` }} />
      </div>
    </div>
  );
}

export function TasksProgressPanel({ files, activeFile }: TasksProgressPanelProps) {
  const { t } = useTranslation();
  if (files.length === 0) return null;

  const combinedDone = files.reduce((acc, f) => acc + f.file.completedCount, 0);
  const combinedTotal = files.reduce((acc, f) => acc + f.file.taskCount, 0);
  const single = files.length === 1;

  return (
    <div className="space-y-2 border-b border-border p-2" data-testid="tasks-progress-panel">
      <Bar
        label={
          single
            ? t("tasks.progress.single", { defaultValue: "Progress" })
            : t("tasks.progress.combined", { defaultValue: "Combined progress" })
        }
        done={combinedDone}
        total={combinedTotal}
        testId="tasks-progress-combined"
      />
      {!single && activeFile && (
        <Bar
          label={t("tasks.progress.current", {
            defaultValue: "Current file: {{name}}",
            name: activeFile.file.displayName,
          })}
          done={activeFile.file.completedCount}
          total={activeFile.file.taskCount}
          testId="tasks-progress-current"
        />
      )}
    </div>
  );
}
