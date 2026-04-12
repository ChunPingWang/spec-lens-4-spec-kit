import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { usePhaseStore, useTerminalStore } from "@/stores";
import type { Uuid } from "@/types/ipc";

interface OverviewPanelProps {
  projectId: Uuid;
  stepId: string;
}

/**
 * Overview sub-page: renders the lazy slices from `phase_overview` and a
 * collapsible terminal output section filtered to the active step (FR-033).
 */
export function OverviewPanel({ projectId, stepId }: OverviewPanelProps) {
  const { t } = useTranslation();
  const overview = usePhaseStore((s) => s.overview);
  const loading = usePhaseStore((s) => s.loadingOverview);
  const loadOverview = usePhaseStore((s) => s.loadOverview);
  const recentLines = useTerminalStore((s) => s.recentLines);
  const [terminalExpanded, setTerminalExpanded] = useState(true);

  const stepLines = useMemo(
    () => recentLines.filter((l) => l.stepId === stepId),
    [recentLines, stepId],
  );

  useEffect(() => {
    void loadOverview(projectId, stepId);
  }, [projectId, stepId, loadOverview]);

  if (loading) {
    return (
      <p className="p-3 text-sm text-muted-foreground">
        {t("phase.loadingOverview", { defaultValue: "Loading overview…" })}
      </p>
    );
  }

  const hasOverviewSlices = overview && overview.slices.length > 0;

  return (
    <div className="space-y-4 p-3">
      {!hasOverviewSlices && (
        <p className="text-sm text-muted-foreground">
          {t("phase.noOverview", {
            defaultValue: "No overview content yet — run this step to populate its summary.",
          })}
        </p>
      )}
      {hasOverviewSlices && overview.slices.map((slice) => (
        <section key={slice.relativePath} className="rounded-md border border-border p-3">
          <header className="mb-2 flex items-center justify-between">
            <h3 className="text-sm font-semibold">{slice.title}</h3>
            <span className="truncate text-xs text-muted-foreground" title={slice.relativePath}>
              {slice.relativePath}
            </span>
          </header>
          <pre className="whitespace-pre-wrap break-words font-mono text-xs leading-relaxed text-muted-foreground">
            {slice.content}
          </pre>
        </section>
      ))}

      {/* Terminal output slice for this step (FR-033 / T102) */}
      <section className="rounded-md border border-border p-3">
        <button
          type="button"
          aria-label={t("phase.terminalSliceToggle", { defaultValue: "Terminal output" })}
          onClick={() => setTerminalExpanded((v) => !v)}
          className="flex w-full items-center justify-between text-left"
        >
          <h3 className="text-sm font-semibold">
            {t("phase.terminalSliceHeading", {
              defaultValue: "Terminal ({{count}})",
              count: stepLines.length,
            })}
          </h3>
          <span className="text-xs text-muted-foreground">{terminalExpanded ? "▼" : "▶"}</span>
        </button>
        {terminalExpanded && (
          <div className="mt-2">
            {stepLines.length === 0 ? (
              <p className="text-xs text-muted-foreground">
                {t("phase.noTerminalOutput", {
                  defaultValue: "No terminal output for this step yet.",
                })}
              </p>
            ) : (
              <pre className="max-h-60 overflow-auto whitespace-pre-wrap break-words font-mono text-[11px] leading-relaxed text-muted-foreground">
                {stepLines.map((l) => l.text).join("\n")}
              </pre>
            )}
          </div>
        )}
      </section>
    </div>
  );
}
