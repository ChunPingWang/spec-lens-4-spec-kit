import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { SpecLensIpcError } from "@/lib/tauri";
import { useProjectStore } from "@/stores";

/**
 * Welcome page — shown when no project is attached to this window.
 * Hosts the "Open project" action plus the recent-projects list.
 * Matches US1 acceptance in `specs/001-speclens-desktop/spec.md §US1`.
 */
export function Welcome() {
  const { t } = useTranslation();
  const recent = useProjectStore((s) => s.recent);
  const openProject = useProjectStore((s) => s.open);
  const loading = useProjectStore((s) => s.loading);
  const [picking, setPicking] = useState(false);

  async function pickAndOpen(): Promise<void> {
    setPicking(true);
    try {
      const chosen = await openDialog({ directory: true, multiple: false });
      if (typeof chosen === "string" && chosen.length > 0) {
        await openProject(chosen);
      }
    } catch (err) {
      // Error state is persisted in the store by `open`; any dialog-only
      // error (user cancelled etc.) is silently ignored.
      if (err instanceof SpecLensIpcError) {
        // already captured in store
      }
    } finally {
      setPicking(false);
    }
  }

  async function openFromRecent(path: string): Promise<void> {
    try {
      await openProject(path);
    } catch {
      // handled via store.error
    }
  }

  const busy = loading || picking;

  return (
    <section
      aria-labelledby="welcome-heading"
      className="flex flex-1 flex-col items-center justify-center gap-6 px-6 py-10 text-center"
    >
      <div>
        <h1 id="welcome-heading" className="text-2xl font-semibold tracking-tight">
          {t("welcome.title")}
        </h1>
        <p className="mt-2 text-sm text-muted-foreground">{t("welcome.subtitle")}</p>
      </div>

      <button
        type="button"
        onClick={() => {
          void pickAndOpen();
        }}
        disabled={busy}
        className="rounded-md border border-input bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-60"
      >
        {t("welcome.openButton")}
      </button>

      <div className="w-full max-w-md text-left">
        <h2 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
          {t("welcome.recentHeading")}
        </h2>
        {recent.length === 0 ? (
          <p className="mt-2 text-sm text-muted-foreground">{t("welcome.noRecent")}</p>
        ) : (
          <ul className="mt-2 space-y-1">
            {recent.map((p) => (
              <li key={p.id}>
                <button
                  type="button"
                  onClick={() => {
                    void openFromRecent(p.path);
                  }}
                  disabled={busy}
                  className="flex w-full items-center justify-between rounded-sm px-2 py-1 text-left text-sm hover:bg-accent disabled:cursor-not-allowed disabled:opacity-60"
                >
                  <span className="truncate">{p.name}</span>
                  <span className="ml-2 truncate text-xs text-muted-foreground">{p.path}</span>
                </button>
              </li>
            ))}
          </ul>
        )}
      </div>
    </section>
  );
}
