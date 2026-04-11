import type { PropsWithChildren } from "react";
import { useTranslation } from "react-i18next";

import { AgentBadge } from "@/components/agent/AgentBadge";
import type { AgentProfile } from "@/types/ipc";

interface ProjectHeaderInfo {
  name: string;
  path: string;
  lastOpenedAt?: string;
  agent?: AgentProfile;
}

interface TopBarProps {
  project?: ProjectHeaderInfo;
}

/**
 * Fixed top bar. When a project is attached, shows name / path / last
 * modified / agent badge (FR-005). Otherwise shows the app title only.
 */
export function TopBar({ project, children }: PropsWithChildren<TopBarProps>) {
  const { t, i18n } = useTranslation();

  const formattedLastOpened = project?.lastOpenedAt
    ? new Intl.DateTimeFormat(i18n.language, {
        dateStyle: "medium",
        timeStyle: "short",
      }).format(new Date(project.lastOpenedAt))
    : null;

  return (
    <header
      role="banner"
      className="flex h-12 shrink-0 items-center justify-between gap-4 border-b border-border bg-card px-4"
    >
      <div className="flex min-w-0 items-center gap-3">
        <span className="text-sm font-semibold tracking-tight">{t("app.name")}</span>
        {project ? (
          <div className="flex min-w-0 items-center gap-2 text-xs">
            <span
              className="truncate font-medium text-foreground"
              title={project.name}
              aria-label="project-name"
            >
              {project.name}
            </span>
            <span
              className="truncate text-muted-foreground"
              title={project.path}
              aria-label="project-path"
            >
              {project.path}
            </span>
            {formattedLastOpened && (
              <span className="shrink-0 text-muted-foreground" aria-label="last-opened-at">
                · {formattedLastOpened}
              </span>
            )}
          </div>
        ) : (
          <span className="text-xs text-muted-foreground">{t("app.tagline")}</span>
        )}
      </div>
      <div className="flex items-center gap-2">
        {project?.agent && <AgentBadge agent={project.agent} />}
        {children}
      </div>
    </header>
  );
}
