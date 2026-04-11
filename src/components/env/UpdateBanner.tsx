/**
 * Non-blocking banner shown at the top of the Workspace when the user
 * has dismissed an amber env-check result (update available or warn).
 * Keeps the "update nudge" visible without trapping the user (FR-013).
 */
import { useTranslation } from "react-i18next";

import { useEnvStore } from "@/stores/envStore";
import type { EnvironmentStatus, Uuid } from "@/types/ipc";

interface UpdateBannerProps {
  projectId: Uuid;
  status: EnvironmentStatus;
}

export function UpdateBanner({ projectId, status }: UpdateBannerProps) {
  const { t } = useTranslation();
  const check = useEnvStore((s) => s.check);
  const loading = useEnvStore((s) => s.loading);

  const installed = status.speckitVersion ?? "?";
  const latest = status.latestKnownVersion ?? "?";

  async function handleRecheck(): Promise<void> {
    try {
      await check(projectId, true);
    } catch {
      // Surfaced via envStore.error / global toast.
    }
  }

  return (
    <div
      role="status"
      data-testid="env-update-banner"
      className="flex items-center justify-between gap-3 border-b border-amber-500/40 bg-amber-500/10 px-4 py-2 text-xs text-amber-900 dark:text-amber-100"
    >
      <p>
        {status.updateAvailable
          ? t("env.updateAvailable", { installed, latest })
          : t("env.warnBanner")}
      </p>
      <button
        type="button"
        onClick={() => {
          void handleRecheck();
        }}
        disabled={loading}
        className="rounded-sm border border-amber-500/50 px-2 py-0.5 text-amber-900 hover:bg-amber-500/20 disabled:opacity-50 dark:text-amber-100"
      >
        {loading ? t("env.rechecking") : t("env.recheck")}
      </button>
    </div>
  );
}
