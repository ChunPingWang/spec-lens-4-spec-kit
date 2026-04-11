/**
 * Compact badge for the active project's AI agent (FR-060 / FR-061).
 *
 * Renders the [`AgentIcon`] + canonical brand name (untranslated per
 * FR-087) plus a localized "auto-detected" / "configured" / "generic"
 * subscript label derived from `detectedFrom`. Used by the TopBar; the
 * label clarifies *how* SpecLens identified the agent so users know
 * whether to trust the result.
 */
import { useTranslation } from "react-i18next";

import { AgentIcon } from "@/components/agent/AgentIcon";
import type { AgentProfile } from "@/types/ipc";

interface AgentBadgeProps {
  agent: AgentProfile;
}

export function AgentBadge({ agent }: AgentBadgeProps) {
  const { t } = useTranslation();

  // Localized "how we detected this" suffix:
  //   project → configured (.specify/init-options.json)
  //   path    → auto-detected (filesystem fingerprint)
  //   none    → generic mode
  const sourceKey =
    agent.detectedFrom === "project"
      ? "agent.source.configured"
      : agent.detectedFrom === "path"
        ? "agent.source.autoDetected"
        : "agent.source.generic";

  return (
    <span
      data-testid="agent-badge"
      data-agent-id={agent.id}
      data-detected-from={agent.detectedFrom}
      title={t(sourceKey) ?? undefined}
      className="inline-flex items-center gap-1.5 rounded-full border border-border bg-muted/40 px-2 py-0.5 text-xs"
    >
      <AgentIcon id={agent.id} size={14} />
      <span className="font-medium text-foreground">{agent.displayName}</span>
      <span className="text-[10px] uppercase tracking-wider text-muted-foreground">
        {t(sourceKey)}
      </span>
    </span>
  );
}
