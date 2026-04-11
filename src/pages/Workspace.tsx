import { useEffect } from "react";
import { useTranslation } from "react-i18next";

import { EnvCheckDialog } from "@/components/env/EnvCheckDialog";
import { UpdateBanner } from "@/components/env/UpdateBanner";
import { PhaseTabs } from "@/components/phase/PhaseTabs";
import { TerminalPanel } from "@/components/terminal/TerminalPanel";
import { invoke } from "@/lib/tauri";
import { deriveBadgeColor, useEnvStore } from "@/stores/envStore";
import { useProjectStore, useStepsStore } from "@/stores";
import type { Step } from "@/types/ipc";

/**
 * Workspace page — three-pane layout (Steps / Phase / Terminal) gated
 * behind a blocking Spec-Kit environment check per FR-010. Amber
 * results show a persistent `UpdateBanner` instead of blocking.
 */
export function Workspace() {
  const { t } = useTranslation();
  const project = useProjectStore((s) => s.current);
  const steps = useStepsStore((s) => s.steps);
  const selectedStepId = useStepsStore((s) => s.selectedStepId);
  const setSteps = useStepsStore((s) => s.setSteps);
  const selectStep = useStepsStore((s) => s.selectStep);
  const completionPercent = useStepsStore((s) => s.completionPercent);

  const envStatus = useEnvStore((s) => s.status);
  const envLoading = useEnvStore((s) => s.loading);
  const envAcknowledged = useEnvStore((s) => s.acknowledged);
  const checkEnv = useEnvStore((s) => s.check);
  const clearEnv = useEnvStore((s) => s.clear);

  // Kick off the Spec-Kit env check the moment a project is attached
  // (FR-010). We never auto-acknowledge — the dialog owns that choice.
  useEffect(() => {
    if (!project) {
      clearEnv();
      return;
    }
    let cancelled = false;
    void (async () => {
      try {
        await checkEnv(project.id);
      } catch {
        if (cancelled) return;
        // Error surfaced through envStore.error → global toast.
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [project, checkEnv, clearEnv]);

  // Load steps for the active project and restore the persisted
  // `lastStepId` selection (T062 / FR-062).
  useEffect(() => {
    if (!project) return;
    let cancelled = false;
    void (async () => {
      try {
        const loaded = await invoke<Step[]>("steps_list", { projectId: project.id });
        if (cancelled) return;
        setSteps(loaded);
        const restored =
          project.state.lastStepId && loaded.some((s) => s.id === project.state.lastStepId)
            ? project.state.lastStepId
            : loaded[0]?.id ?? null;
        selectStep(restored);
      } catch {
        if (cancelled) return;
        const fallback = project.steps ?? [];
        setSteps(fallback);
        selectStep(project.state.lastStepId ?? fallback[0]?.id ?? null);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [project, setSteps, selectStep]);

  async function handleSelectStep(id: string): Promise<void> {
    selectStep(id);
    if (!project) return;
    try {
      await invoke("project_set_last_step", { projectId: project.id, stepId: id });
    } catch {
      // Non-fatal: selection still works in-memory for this session.
    }
  }

  const badge = envStatus ? deriveBadgeColor(envStatus) : null;
  // Green never blocks — acknowledge it as soon as it arrives so that
  // the amber banner logic stays symmetric and the dialog only ever
  // renders for red/amber states.
  useEffect(() => {
    if (envStatus && badge === "green" && !envAcknowledged) {
      useEnvStore.getState().acknowledge();
    }
  }, [envStatus, badge, envAcknowledged]);

  if (!project) return null;

  const percent = completionPercent();
  // Red always blocks; amber blocks until the user acknowledges.
  const mustBlock =
    envStatus !== null && !envAcknowledged && (badge === "red" || badge === "amber");
  const showAmberBanner =
    envStatus !== null && envAcknowledged && badge === "amber";

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      {showAmberBanner && envStatus ? (
        <UpdateBanner projectId={project.id} status={envStatus} />
      ) : null}

      {mustBlock && envStatus ? (
        <EnvCheckDialog projectId={project.id} status={envStatus} />
      ) : null}

      {envLoading && !envStatus ? (
        <div className="flex h-12 items-center justify-center border-b border-border bg-muted/30 text-xs text-muted-foreground">
          {t("env.checking")}
        </div>
      ) : null}

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
                    onClick={() => {
                      void handleSelectStep(s.id);
                    }}
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

        <div className="flex flex-col overflow-hidden rounded-md border border-border bg-card">
          {selectedStepId ? (
            <PhaseTabs projectId={project.id} stepId={selectedStepId} />
          ) : (
            <p className="p-4 text-sm text-muted-foreground">{t("workspace.noStep")}</p>
          )}
        </div>

        <aside className="flex flex-col overflow-hidden rounded-md border border-border bg-card">
          <TerminalPanel projectId={project.id} />
        </aside>
      </section>
    </div>
  );
}
