/**
 * Presentational install guide for the Spec-Kit CLI. Rendered inside
 * `EnvCheckDialog` when the probe returns red (missing binary) so the
 * user has an offline, copy-pasteable path forward (FR-011 / FR-086).
 */
import { useTranslation } from "react-i18next";

import type { InstallGuide } from "@/types/ipc";

interface InstallGuideViewProps {
  guide: InstallGuide;
}

export function InstallGuideView({ guide }: InstallGuideViewProps) {
  const { t } = useTranslation();
  return (
    <section
      data-testid="env-install-guide"
      aria-label={t("env.installGuideLabel") ?? undefined}
      className="flex flex-col gap-3"
    >
      <header className="flex items-center justify-between text-xs">
        <span className="font-semibold uppercase tracking-wide text-muted-foreground">
          {t("env.platform", { platform: guide.platform })}
        </span>
        <a
          href={guide.releaseNotesUrl}
          target="_blank"
          rel="noreferrer"
          className="text-primary underline-offset-2 hover:underline"
        >
          {t("env.releaseNotes")}
        </a>
      </header>
      <ol className="space-y-3 text-sm">
        {guide.steps.map((step, idx) => (
          <li
            key={`${step.title}-${idx}`}
            data-testid={`env-install-step-${idx}`}
            className="rounded-md border border-border bg-muted/30 p-3"
          >
            <p className="font-medium text-foreground">
              {idx + 1}. {step.title}
            </p>
            <p className="mt-1 text-xs text-muted-foreground">{step.description}</p>
            {step.command ? (
              <pre className="mt-2 overflow-x-auto rounded-sm bg-background px-2 py-1 font-mono text-xs">
                <code>{step.command}</code>
              </pre>
            ) : null}
          </li>
        ))}
      </ol>
    </section>
  );
}
