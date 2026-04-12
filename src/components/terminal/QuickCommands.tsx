/**
 * T100 — Optional quick-command buttons for common Spec-Kit CLI invocations.
 * Gated behind the `allowTerminalForwarding` setting (FR-044); renders
 * nothing when disabled. This is the ONLY path through which SpecLens may
 * cause command execution — the main workspace never exposes Run/Retry/Reset.
 */
import { useCallback } from "react";
import { useTranslation } from "react-i18next";

export interface QuickCommandsProps {
  allowTerminalForwarding: boolean;
  connected: boolean;
  onSend: (command: string) => void;
}

interface QuickCmd {
  labelKey: string;
  defaultLabel: string;
  command: string;
}

const COMMANDS: QuickCmd[] = [
  { labelKey: "terminal.cmdStatus", defaultLabel: "spec-kit status", command: "spec-kit status\n" },
  { labelKey: "terminal.cmdRun", defaultLabel: "spec-kit run", command: "spec-kit run\n" },
  { labelKey: "terminal.cmdValidate", defaultLabel: "spec-kit validate", command: "spec-kit validate\n" },
];

export function QuickCommands({ allowTerminalForwarding, connected, onSend }: QuickCommandsProps) {
  const { t } = useTranslation();

  const handleClick = useCallback(
    (cmd: string) => {
      onSend(cmd);
    },
    [onSend],
  );

  if (!allowTerminalForwarding) return null;

  return (
    <div
      role="group"
      aria-label={t("terminal.quickCommandsGroup", { defaultValue: "Quick commands" })}
      className="flex items-center gap-1 border-b border-border px-2 py-1"
    >
      {COMMANDS.map((c) => (
        <button
          key={c.labelKey}
          type="button"
          aria-label={t(c.labelKey, { defaultValue: c.defaultLabel })}
          disabled={!connected}
          onClick={() => handleClick(c.command)}
          className="h-6 rounded-sm border border-border px-2 text-xs text-muted-foreground hover:bg-accent disabled:cursor-not-allowed disabled:opacity-50"
        >
          {t(c.labelKey, { defaultValue: c.defaultLabel })}
        </button>
      ))}
    </div>
  );
}
