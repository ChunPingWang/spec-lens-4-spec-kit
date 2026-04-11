/**
 * Blocking dialog for the Spec-Kit environment check (FR-010 – FR-013).
 *
 * Visual states, driven by `deriveBadgeColor`:
 *
 * - **red**   — Spec-Kit binary missing or probe errored. Shows the
 *               bundled install guide and a "Re-check" button. Dialog is
 *               modal and cannot be dismissed.
 * - **amber** — Installed but outdated OR offline/partial failure.
 *               Shows the version mismatch / warning with hints plus a
 *               "Continue anyway" button (FR-013).
 * - **green** — Spec-Kit installed and up to date. Auto-dismisses via
 *               the parent `Workspace` page calling `acknowledge()`.
 */
import { useEffect, useMemo } from "react";
import { useTranslation } from "react-i18next";

import { InstallGuideView } from "@/components/env/InstallGuideView";
import { deriveBadgeColor, useEnvStore } from "@/stores/envStore";
import type { EnvironmentStatus, Uuid } from "@/types/ipc";

interface EnvCheckDialogProps {
  projectId: Uuid;
  status: EnvironmentStatus;
}

const BADGE_STYLE: Record<"green" | "amber" | "red", string> = {
  green: "bg-green-500/15 text-green-700 dark:text-green-300 border-green-500/40",
  amber: "bg-amber-500/15 text-amber-700 dark:text-amber-300 border-amber-500/40",
  red: "bg-red-500/15 text-red-700 dark:text-red-300 border-red-500/40",
};

export function EnvCheckDialog({ projectId, status }: EnvCheckDialogProps) {
  const { t } = useTranslation();
  const badge = useMemo(() => deriveBadgeColor(status), [status]);
  const loading = useEnvStore((s) => s.loading);
  const guide = useEnvStore((s) => s.guide);
  const guideLoading = useEnvStore((s) => s.guideLoading);
  const check = useEnvStore((s) => s.check);
  const loadGuide = useEnvStore((s) => s.loadGuide);
  const acknowledge = useEnvStore((s) => s.acknowledge);

  // Lazily fetch the install guide the first time a red state appears.
  useEffect(() => {
    if (badge === "red" && !guide && !guideLoading) {
      void loadGuide();
    }
  }, [badge, guide, guideLoading, loadGuide]);

  const titleKey =
    badge === "red"
      ? "env.title.red"
      : badge === "amber"
        ? "env.title.amber"
        : "env.title.green";

  const errorIssue = status.issues.find((i) => i.severity === "error");
  const warnIssues = status.issues.filter((i) => i.severity === "warn");

  async function handleRecheck(): Promise<void> {
    try {
      await check(projectId, true);
    } catch {
      // Error surfaced via envStore.error → global ErrorToast.
    }
  }

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="env-check-title"
      data-testid="env-check-dialog"
      data-badge={badge}
      className="fixed inset-0 z-40 flex items-center justify-center bg-background/80 p-4 backdrop-blur-sm"
    >
      <div className="flex max-h-[90vh] w-full max-w-xl flex-col gap-4 overflow-hidden rounded-lg border border-border bg-card p-5 shadow-xl">
        <header className="flex items-start justify-between gap-3">
          <div>
            <span
              data-testid="env-badge"
              data-color={badge}
              className={`inline-flex items-center rounded-full border px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wider ${BADGE_STYLE[badge]}`}
            >
              {t(`env.badge.${badge}`)}
            </span>
            <h2
              id="env-check-title"
              className="mt-2 text-lg font-semibold text-foreground"
            >
              {t(titleKey)}
            </h2>
          </div>
          <div className="text-right text-xs text-muted-foreground">
            {status.speckitVersion ? (
              <p>
                <span className="text-muted-foreground">
                  {t("env.installedVersion")}:{" "}
                </span>
                <span className="font-mono text-foreground">
                  {status.speckitVersion}
                </span>
              </p>
            ) : null}
            {status.latestKnownVersion ? (
              <p>
                <span className="text-muted-foreground">
                  {t("env.latestVersion")}:{" "}
                </span>
                <span className="font-mono text-foreground">
                  {status.latestKnownVersion}
                </span>
              </p>
            ) : null}
          </div>
        </header>

        <div className="flex-1 overflow-y-auto">
          {errorIssue ? (
            <p className="mb-3 text-sm text-foreground">
              {errorIssue.message}
              {errorIssue.hint ? (
                <span className="mt-1 block text-xs text-muted-foreground">
                  {errorIssue.hint}
                </span>
              ) : null}
            </p>
          ) : null}

          {warnIssues.length > 0 ? (
            <ul className="mb-3 space-y-2 text-sm">
              {warnIssues.map((issue, idx) => (
                <li
                  key={`${issue.kind}-${idx}`}
                  className="rounded-md border border-amber-500/30 bg-amber-500/10 p-2 text-amber-900 dark:text-amber-100"
                >
                  <p>{issue.message}</p>
                  {issue.hint ? (
                    <p className="mt-1 text-xs opacity-80">{issue.hint}</p>
                  ) : null}
                </li>
              ))}
            </ul>
          ) : null}

          {badge === "green" && status.issues.length === 0 ? (
            <p className="text-sm text-muted-foreground">{t("env.allClear")}</p>
          ) : null}

          {badge === "red" ? (
            guide ? (
              <InstallGuideView guide={guide} />
            ) : guideLoading ? (
              <p className="text-sm text-muted-foreground">
                {t("env.loadingGuide")}
              </p>
            ) : null
          ) : null}
        </div>

        <footer className="flex items-center justify-end gap-2">
          <button
            type="button"
            onClick={() => {
              void handleRecheck();
            }}
            disabled={loading}
            className="rounded-sm border border-border px-3 py-1.5 text-sm hover:bg-muted disabled:opacity-50"
          >
            {loading ? t("env.rechecking") : t("env.recheck")}
          </button>
          {badge !== "red" ? (
            <button
              type="button"
              onClick={acknowledge}
              data-testid="env-continue-button"
              className="rounded-sm bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90"
            >
              {badge === "green" ? t("env.continue") : t("env.continueAnyway")}
            </button>
          ) : null}
        </footer>
      </div>
    </div>
  );
}
