/**
 * TaskFilter — All / Completed / Incomplete chips plus a search input.
 * Per FR-056 the state here is scoped per active tab; the parent owns
 * the state so it can be keyed by the current file path.
 */
import { useTranslation } from "react-i18next";

export type TaskFilterMode = "all" | "completed" | "incomplete";

interface TaskFilterProps {
  mode: TaskFilterMode;
  query: string;
  onModeChange: (mode: TaskFilterMode) => void;
  onQueryChange: (query: string) => void;
}

const MODES: TaskFilterMode[] = ["all", "incomplete", "completed"];

export function TaskFilter({ mode, query, onModeChange, onQueryChange }: TaskFilterProps) {
  const { t } = useTranslation();
  return (
    <div className="flex items-center gap-2 border-b border-border px-2 py-1">
      <div role="tablist" aria-label={t("tasks.filterLabel", { defaultValue: "Filter tasks" })}>
        {MODES.map((m) => (
          <button
            key={m}
            type="button"
            role="tab"
            aria-selected={mode === m}
            onClick={() => onModeChange(m)}
            data-testid={`task-filter-${m}`}
            className={`rounded-sm px-2 py-1 text-xs ${
              mode === m
                ? "bg-primary text-primary-foreground"
                : "text-muted-foreground hover:text-foreground"
            }`}
          >
            {t(`tasks.filter.${m}`, { defaultValue: m })}
          </button>
        ))}
      </div>
      <input
        type="search"
        value={query}
        onChange={(e) => onQueryChange(e.target.value)}
        placeholder={t("tasks.searchPlaceholder", { defaultValue: "Search tasks…" })}
        aria-label={t("tasks.searchLabel", { defaultValue: "Search tasks" })}
        className="ml-auto w-48 rounded-sm border border-border bg-background px-2 py-1 text-xs"
      />
    </div>
  );
}
