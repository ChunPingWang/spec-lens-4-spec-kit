import { useTranslation } from "react-i18next";

import type { DocStatus, PhaseDocument } from "@/types/ipc";

interface DocumentItemProps {
  document: PhaseDocument;
  active: boolean;
  onSelect: (relativePath: string) => void;
}

function badgeClass(status: DocStatus): string {
  switch (status) {
    case "generated":
      return "bg-emerald-500/15 text-emerald-500";
    case "modified":
      return "bg-amber-500/15 text-amber-500";
    case "missing":
      return "bg-rose-500/15 text-rose-500";
    case "unverified":
      return "bg-sky-500/15 text-sky-500";
    default:
      return "bg-muted text-muted-foreground";
  }
}

/**
 * A single document row in the Documents sub-page. Colour-codes status
 * via the `badgeClass` helper and surfaces `relativePath` on hover so the
 * user can reveal it in their editor.
 */
export function DocumentItem({ document, active, onSelect }: DocumentItemProps) {
  const { t } = useTranslation();
  const label = t(`phase.docStatus.${document.status}`, { defaultValue: document.status });

  return (
    <button
      type="button"
      onClick={() => onSelect(document.relativePath)}
      disabled={document.status === "missing"}
      className={`flex w-full items-center justify-between gap-2 rounded-sm px-2 py-1 text-left text-sm ${
        active ? "bg-accent text-accent-foreground" : "hover:bg-accent/60"
      } disabled:cursor-not-allowed disabled:opacity-60`}
      aria-current={active ? "true" : undefined}
      title={document.relativePath}
    >
      <span className="truncate">{document.displayName}</span>
      <span
        className={`shrink-0 rounded-full px-2 py-0.5 text-xs font-medium ${badgeClass(document.status)}`}
        aria-label={`status-${document.status}`}
      >
        {label}
      </span>
    </button>
  );
}
