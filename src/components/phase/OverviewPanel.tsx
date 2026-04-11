import { useEffect } from "react";
import { useTranslation } from "react-i18next";

import { usePhaseStore } from "@/stores";
import type { Uuid } from "@/types/ipc";

interface OverviewPanelProps {
  projectId: Uuid;
  stepId: string;
}

/**
 * Overview sub-page: renders the lazy slices from `phase_overview`.
 * Terminal slice integration lands in US3.
 */
export function OverviewPanel({ projectId, stepId }: OverviewPanelProps) {
  const { t } = useTranslation();
  const overview = usePhaseStore((s) => s.overview);
  const loading = usePhaseStore((s) => s.loadingOverview);
  const loadOverview = usePhaseStore((s) => s.loadOverview);

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

  if (!overview || overview.slices.length === 0) {
    return (
      <p className="p-3 text-sm text-muted-foreground">
        {t("phase.noOverview", {
          defaultValue: "No overview content yet — run this step to populate its summary.",
        })}
      </p>
    );
  }

  return (
    <div className="space-y-4 p-3">
      {overview.slices.map((slice) => (
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
    </div>
  );
}
