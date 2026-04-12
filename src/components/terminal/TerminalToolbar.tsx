/**
 * T099 — Toolbar for the terminal pane. Provides search (FR-042), severity
 * filters (FR-042), copy / export / clear-view (FR-043), and auto-scroll
 * lock. Stateless — all callbacks are owned by the parent TerminalPanel.
 */
import { useCallback, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

import type { HighlightSeverity } from "@/types/ipc";

export interface TerminalToolbarProps {
  connected: boolean;
  autoScroll: boolean;
  onAutoScrollChange: (value: boolean) => void;
  onSearch: (query: string, isRegex: boolean) => void;
  onClearSearch: () => void;
  onClearView: () => void;
  onCopy: () => void;
  onExport: (format: "log" | "txt") => void;
  severityFilter: HighlightSeverity | null;
  onSeverityFilterChange: (severity: HighlightSeverity | null) => void;
  searchResultCount: number;
}

const SEVERITIES: HighlightSeverity[] = ["info", "warn", "error"];

export function TerminalToolbar({
  connected,
  autoScroll,
  onAutoScrollChange,
  onSearch,
  onClearSearch,
  onClearView,
  onCopy,
  onExport,
  severityFilter,
  onSeverityFilterChange,
  searchResultCount,
}: TerminalToolbarProps) {
  const { t } = useTranslation();
  const [regexMode, setRegexMode] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  const handleSubmit = useCallback(
    (e: React.FormEvent) => {
      e.preventDefault();
      const query = inputRef.current?.value.trim() ?? "";
      if (query) {
        onSearch(query, regexMode);
      }
    },
    [onSearch, regexMode],
  );

  const handleSeverityClick = useCallback(
    (severity: HighlightSeverity) => {
      onSeverityFilterChange(severityFilter === severity ? null : severity);
    },
    [onSeverityFilterChange, severityFilter],
  );

  return (
    <div className="flex flex-wrap items-center gap-2 border-b border-border px-2 py-1">
      {/* Search */}
      <form onSubmit={handleSubmit} className="flex items-center gap-1">
        <input
          ref={inputRef}
          type="search"
          disabled={!connected}
          placeholder={t("terminal.searchPlaceholder", { defaultValue: "Search…" })}
          aria-label={t("terminal.searchLabel", { defaultValue: "Search terminal" })}
          className="h-6 w-36 rounded-sm border border-border bg-background px-1 text-xs placeholder:text-muted-foreground disabled:cursor-not-allowed disabled:opacity-50"
        />
        <button
          type="button"
          aria-label={t("terminal.regexToggle", { defaultValue: "Regex" })}
          aria-pressed={regexMode}
          onClick={() => setRegexMode((v) => !v)}
          className={`h-6 rounded-sm border px-1.5 text-xs font-mono ${
            regexMode
              ? "border-primary bg-primary/10 text-primary"
              : "border-border text-muted-foreground hover:bg-accent"
          }`}
        >
          .*
        </button>
        {searchResultCount > 0 && (
          <>
            <span className="text-xs text-muted-foreground">{searchResultCount}</span>
            <button
              type="button"
              aria-label={t("terminal.clearSearch", { defaultValue: "Clear search" })}
              onClick={onClearSearch}
              className="h-6 rounded-sm border border-border px-1.5 text-xs text-muted-foreground hover:bg-accent"
            >
              &times;
            </button>
          </>
        )}
      </form>

      {/* Severity filters */}
      <div className="flex items-center gap-0.5" role="group" aria-label={t("terminal.severityFilterGroup", { defaultValue: "Severity filters" })}>
        {SEVERITIES.map((s) => (
          <button
            key={s}
            type="button"
            aria-label={s}
            aria-pressed={severityFilter === s}
            onClick={() => handleSeverityClick(s)}
            className={`h-6 rounded-sm border px-1.5 text-xs capitalize ${
              severityFilter === s
                ? "border-primary bg-primary/10 text-primary"
                : "border-border text-muted-foreground hover:bg-accent"
            }`}
          >
            {s}
          </button>
        ))}
      </div>

      {/* Spacer */}
      <div className="flex-1" />

      {/* Actions */}
      <div className="flex items-center gap-1">
        <button
          type="button"
          aria-label={t("terminal.copy", { defaultValue: "Copy" })}
          disabled={!connected}
          onClick={onCopy}
          className="h-6 rounded-sm border border-border px-1.5 text-xs text-muted-foreground hover:bg-accent disabled:cursor-not-allowed disabled:opacity-50"
        >
          {t("terminal.copy", { defaultValue: "Copy" })}
        </button>
        <button
          type="button"
          aria-label={t("terminal.exportLog", { defaultValue: "Export .log" })}
          onClick={() => onExport("log")}
          className="h-6 rounded-sm border border-border px-1.5 text-xs text-muted-foreground hover:bg-accent"
        >
          .log
        </button>
        <button
          type="button"
          aria-label={t("terminal.exportTxt", { defaultValue: "Export .txt" })}
          onClick={() => onExport("txt")}
          className="h-6 rounded-sm border border-border px-1.5 text-xs text-muted-foreground hover:bg-accent"
        >
          .txt
        </button>
        <button
          type="button"
          aria-label={t("terminal.clearView", { defaultValue: "Clear view" })}
          onClick={onClearView}
          className="h-6 rounded-sm border border-border px-1.5 text-xs text-muted-foreground hover:bg-accent"
        >
          {t("terminal.clearView", { defaultValue: "Clear view" })}
        </button>
        <label className="flex items-center gap-1 text-xs text-muted-foreground">
          <input
            type="checkbox"
            checked={autoScroll}
            onChange={(e) => onAutoScrollChange(e.target.checked)}
            aria-label={t("terminal.autoScrollLabel", { defaultValue: "Auto-scroll" })}
          />
          {t("terminal.autoScrollLabel", { defaultValue: "Auto-scroll" })}
        </label>
      </div>
    </div>
  );
}
