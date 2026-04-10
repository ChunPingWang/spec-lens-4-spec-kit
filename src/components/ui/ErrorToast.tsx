import { useEffect } from "react";
import { useTranslation } from "react-i18next";

import { useProjectStore } from "@/stores";

/**
 * Global toast for the most recent `SpecLensIpcError`. Constitution IV
 * requires errors to describe "what happened / why / what to do next"
 * — we render message as the primary line and hint as the secondary.
 */
export function ErrorToast() {
  const { t } = useTranslation();
  const error = useProjectStore((s) => s.error);
  const clearError = useProjectStore((s) => s.clearError);

  useEffect(() => {
    if (!error) return;
    const timeout = setTimeout(() => clearError(), 8000);
    return () => clearTimeout(timeout);
  }, [error, clearError]);

  if (!error) return null;

  return (
    <div
      role="alert"
      aria-live="assertive"
      className="pointer-events-auto fixed bottom-4 right-4 z-50 max-w-sm rounded-md border border-destructive/40 bg-card p-3 shadow-lg"
    >
      <div className="flex items-start justify-between gap-3">
        <div>
          <p className="text-sm font-semibold text-destructive">
            {t("errors.genericTitle")}
          </p>
          <p className="mt-1 text-sm text-foreground">{error.message}</p>
          {error.hint ? (
            <p className="mt-1 text-xs text-muted-foreground">{error.hint}</p>
          ) : null}
          <p className="mt-1 text-[10px] uppercase tracking-wider text-muted-foreground">
            {error.code}
          </p>
        </div>
        <button
          type="button"
          onClick={clearError}
          className="rounded-sm px-2 py-1 text-xs text-muted-foreground hover:bg-muted focus-visible:outline focus-visible:outline-2 focus-visible:outline-ring"
        >
          {t("errors.dismiss")}
        </button>
      </div>
    </div>
  );
}
