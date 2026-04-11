/**
 * Tiny inline glyph stand-in for the per-agent brand icons. v1 keeps the
 * desktop bundle hermetic by rendering a colored monogram (first 1-2
 * letters of the canonical brand name) instead of vendoring brand SVGs
 * — replaceable post-v1 once licensing is sorted.
 */
import type { AgentId } from "@/types/ipc";

interface AgentIconProps {
  id: AgentId;
  /** CSS pixel size of the icon (square). */
  size?: number;
  className?: string;
}

const MONOGRAM: Record<AgentId, string> = {
  "claude-code": "Cl",
  copilot: "GH",
  "gemini-cli": "Ge",
  cursor: "Cu",
  windsurf: "Wi",
  "amazon-q": "Aq",
  "codex-cli": "Cx",
  "qwen-code": "Qw",
  opencode: "oc",
  "kilo-code": "Ki",
  "auggie-cli": "Au",
  "roo-code": "Ro",
  generic: "··",
};

const TINT: Record<AgentId, string> = {
  "claude-code": "bg-orange-500/20 text-orange-700 dark:text-orange-300",
  copilot: "bg-zinc-500/20 text-zinc-700 dark:text-zinc-300",
  "gemini-cli": "bg-blue-500/20 text-blue-700 dark:text-blue-300",
  cursor: "bg-purple-500/20 text-purple-700 dark:text-purple-300",
  windsurf: "bg-teal-500/20 text-teal-700 dark:text-teal-300",
  "amazon-q": "bg-yellow-500/20 text-yellow-700 dark:text-yellow-300",
  "codex-cli": "bg-emerald-500/20 text-emerald-700 dark:text-emerald-300",
  "qwen-code": "bg-rose-500/20 text-rose-700 dark:text-rose-300",
  opencode: "bg-sky-500/20 text-sky-700 dark:text-sky-300",
  "kilo-code": "bg-indigo-500/20 text-indigo-700 dark:text-indigo-300",
  "auggie-cli": "bg-amber-500/20 text-amber-700 dark:text-amber-300",
  "roo-code": "bg-green-500/20 text-green-700 dark:text-green-300",
  generic: "bg-muted text-muted-foreground",
};

export function AgentIcon({ id, size = 16, className = "" }: AgentIconProps) {
  return (
    <span
      data-testid={`agent-icon-${id}`}
      aria-hidden="true"
      style={{ width: size, height: size, fontSize: Math.max(8, size - 8) }}
      className={`inline-flex items-center justify-center rounded-sm font-mono font-semibold leading-none ${TINT[id]} ${className}`}
    >
      {MONOGRAM[id]}
    </span>
  );
}
