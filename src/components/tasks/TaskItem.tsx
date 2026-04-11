/**
 * TaskItem — a single task row. Collapsed by default; expanding calls
 * `onToggleExpand` so the parent can render a `<TaskDetail />` slice.
 * Status icons reflect `TaskStatus` and are mirrored into `aria-label`.
 */
import { useTranslation } from "react-i18next";

import type { TaskEntry, TaskStatus } from "@/types/ipc";

interface TaskItemProps {
  task: TaskEntry;
  expanded: boolean;
  onToggleExpand: () => void;
}

const STATUS_ICON: Record<TaskStatus, string> = {
  todo: "○",
  in_progress: "◐",
  done: "●",
  skipped: "—",
};

export function TaskItem({ task, expanded, onToggleExpand }: TaskItemProps) {
  const { t } = useTranslation();
  const statusLabel = t(`tasks.status.${task.status}`, { defaultValue: task.status });

  return (
    <li className="flex items-center gap-2" data-testid={`task-item-${task.id}`}>
      <span
        aria-label={statusLabel}
        title={statusLabel}
        className="w-4 select-none text-center text-sm"
      >
        {STATUS_ICON[task.status]}
      </span>
      <button
        type="button"
        onClick={onToggleExpand}
        aria-expanded={expanded}
        className="flex-1 truncate text-left text-xs hover:underline"
      >
        <span
          className={task.status === "done" ? "text-muted-foreground line-through" : undefined}
        >
          {task.title}
        </span>
      </button>
      <span className="text-[10px] text-muted-foreground">{expanded ? "▾" : "▸"}</span>
    </li>
  );
}
