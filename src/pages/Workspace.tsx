import { useEffect } from "react";
import { useTranslation } from "react-i18next";

import { invoke } from "@/lib/tauri";
import { useProjectStore, useStepsStore } from "@/stores";
import type { Step } from "@/types/ipc";

/**
 * Workspace page — three-pane layout (Steps / Phase / Terminal).
 * Phase 3 US1 wires the Steps sidebar + placeholder detail panes so the
 * user can confirm a project opened correctly. Phase/Terminal panes gain
 * real content in US2–US4.
 */
export function Workspace() {
  const { t } = useTranslation();
  const project = useProjectStore((s) => s.current);
  const steps = useStepsStore((s) => s.steps);
  const selectedStepId = useStepsStore((s) => s.selectedStepId);
  const setSteps = useStepsStore((s) => s.setSteps);
  const selectStep = useStepsStore((s) => s.selectStep);
  const completionPercent = useStepsStore((s) => s.completionPercent);

  // Load steps for the active project. The backend reads from the
  // open-projects table seeded by `project_open`.
  useEffect(() => {
    if (!project) return;
    let cancelled = false;
    void (async () => {
      try {
        const loaded = await invoke<Step[]>("steps_list", { projectId: project.id });
        if (!cancelled) setSteps(loaded);
      } catch {
        if (!cancelled) setSteps(project.steps ?? []);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [project, setSteps]);

  if (!project) return null;

  const percent = completionPercent();

  return (
    <section
      aria-label="Workspace"
      className="grid flex-1 grid-cols-[260px_1fr_320px] gap-2 p-2"
    >
      <aside
        aria-label={t("workspace.stepsHeading") ?? undefined}
        className="flex flex-col gap-3 rounded-md border border-border bg-card p-3"
      >
        <div>
          <h2 className="text-sm font-semibold">{t("workspace.stepsHeading")}</h2>
          <p className="truncate text-xs text-muted-foreground" title={project.rootPath}>
            {project.name}
          </p>
        </div>

        <div>
          <div className="mb-1 flex items-center justify-between text-xs text-muted-foreground">
            <span>{t("workspace.progressLabel")}</span>
            <span aria-label={`${percent}%`}>{percent}%</span>
          </div>
          <div
            role="progressbar"
            aria-valuemin={0}
            aria-valuemax={100}
            aria-valuenow={percent}
            className="h-1.5 w-full overflow-hidden rounded-full bg-muted"
          >
            <div
              className="h-full bg-primary transition-[width]"
              style={{ width: `${percent}%` }}
            />
          </div>
        </div>

        <ul className="space-y-1 text-sm">
          {steps.map((s) => {
            const active = s.id === selectedStepId;
            return (
              <li key={s.id}>
                <button
                  type="button"
                  onClick={() => selectStep(s.id)}
                  className={`flex w-full items-center justify-between rounded-sm px-2 py-1 text-left ${
                    active ? "bg-accent text-accent-foreground" : "hover:bg-accent/60"
                  }`}
                  aria-current={active ? "step" : undefined}
                >
                  <span className="truncate">{s.displayName}</span>
                  <span className="ml-2 text-xs text-muted-foreground">
                    {t(`steps.status.${s.status}`)}
                  </span>
                </button>
              </li>
            );
          })}
        </ul>
      </aside>

      <div className="flex flex-col rounded-md border border-border bg-card p-3">
        <h2 className="text-sm font-semibold">{project.rootPath}</h2>
        <p className="mt-2 text-sm text-muted-foreground">{t("workspace.noStep")}</p>
      </div>

      <aside className="flex flex-col rounded-md border border-border bg-card p-3">
        <h2 className="text-sm font-semibold">{t("workspace.terminalHeading")}</h2>
      </aside>
    </section>
  );
}
