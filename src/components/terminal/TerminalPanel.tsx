/**
 * Bottom-right terminal pane for the workspace. Wraps `@xterm/xterm` with
 * the fit addon for sizing and delegates PTY IO to `useTerminalStore` /
 * `usePtySession`. See spec §US3 + T098.
 *
 * For testability and SSR-friendliness the xterm instance is loaded lazily
 * inside a `useEffect`; the component falls back to a plain `<ul>` mirror
 * of `recentLines` when xterm is unavailable (tests and the initial paint).
 */
import { useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

import { usePtySession } from "@/hooks/usePtySession";
import type { SpecLensIpcError } from "@/lib/tauri";
import { useTerminalStore } from "@/stores";
import type { OutputLine, Uuid } from "@/types/ipc";

interface TerminalPanelProps {
  projectId: Uuid | null;
}

type XtermHandles = {
  write: (chunk: string) => void;
  dispose: () => void;
  fit: () => void;
};

export function TerminalPanel({ projectId }: TerminalPanelProps) {
  const { t } = useTranslation();
  const descriptor = useTerminalStore((s) => s.descriptor);
  const connecting = useTerminalStore((s) => s.connecting);
  const error = useTerminalStore((s) => s.error);
  const recentLines = useTerminalStore((s) => s.recentLines);
  const attach = useTerminalStore((s) => s.attach);
  const detach = useTerminalStore((s) => s.detach);

  const mountRef = useRef<HTMLDivElement | null>(null);
  const handlesRef = useRef<XtermHandles | null>(null);
  const [autoScrollLocked, setAutoScrollLocked] = useState<boolean>(true);

  // Subscribe to PTY events. Writes new lines into xterm if available and
  // always mirrors into the store's recentLines.
  usePtySession({
    onLines: (lines: OutputLine[]) => {
      const handles = handlesRef.current;
      if (!handles) return;
      for (const line of lines) {
        handles.write(`${formatLineForXterm(line)}\r\n`);
      }
    },
  });

  // Lazy-load xterm once the descriptor is set and we're in a browser.
  useEffect(() => {
    if (!descriptor) {
      handlesRef.current?.dispose();
      handlesRef.current = null;
      return;
    }
    let disposed = false;
    let handles: XtermHandles | null = null;
    void (async () => {
      try {
        const [{ Terminal }, { FitAddon }] = await Promise.all([
          import("@xterm/xterm"),
          import("@xterm/addon-fit"),
        ]);
        if (disposed || !mountRef.current) return;
        const term = new Terminal({
          convertEol: true,
          fontFamily: "ui-monospace, SFMono-Regular, Menlo, Monaco, monospace",
          fontSize: 12,
          theme: { background: "#0b0b0e" },
        });
        const fit = new FitAddon();
        term.loadAddon(fit);
        term.open(mountRef.current);
        try {
          fit.fit();
        } catch {
          // fit may fail on hidden containers; safe to ignore.
        }
        handles = {
          write: (chunk) => term.write(chunk),
          dispose: () => term.dispose(),
          fit: () => {
            try {
              fit.fit();
            } catch {
              // noop
            }
          },
        };
        handlesRef.current = handles;
        // Replay the current recentLines so the xterm mount catches up.
        for (const line of recentLines) {
          term.write(`${formatLineForXterm(line)}\r\n`);
        }
      } catch {
        // xterm failed to load (tests, offline) — silently fall back to
        // the `<ul>` mirror rendered below.
      }
    })();
    return () => {
      disposed = true;
      handles?.dispose();
      if (handlesRef.current === handles) {
        handlesRef.current = null;
      }
    };
    // Intentionally omit recentLines from deps — we only want to replay on
    // initial mount, not on every store change.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [descriptor]);

  async function handleConnect(): Promise<void> {
    if (!projectId) return;
    try {
      await attach(projectId);
    } catch {
      // Error is surfaced via the store.
    }
  }

  const status = useMemo<string>(() => {
    if (error) return formatError(error);
    if (connecting) return t("terminal.connecting", { defaultValue: "Connecting…" });
    if (descriptor) {
      return t("terminal.connected", {
        defaultValue: `Connected • ${descriptor.shell}`,
        shell: descriptor.shell,
      });
    }
    return t("terminal.idle", { defaultValue: "Not connected" });
  }, [connecting, descriptor, error, t]);

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center justify-between gap-2 border-b border-border px-2 py-1">
        <h2 className="text-sm font-semibold">
          {t("workspace.terminalHeading", { defaultValue: "Terminal" })}
        </h2>
        <div className="flex items-center gap-2 text-xs">
          <label className="flex items-center gap-1 text-muted-foreground">
            <input
              type="checkbox"
              checked={autoScrollLocked}
              onChange={(e) => setAutoScrollLocked(e.target.checked)}
              aria-label={t("terminal.autoScrollLabel", { defaultValue: "Auto-scroll" })}
            />
            {t("terminal.autoScrollLabel", { defaultValue: "Auto-scroll" })}
          </label>
          {descriptor ? (
            <button
              type="button"
              onClick={() => {
                void detach();
              }}
              className="rounded-sm border border-border px-2 py-0.5 hover:bg-accent"
            >
              {t("terminal.disconnect", { defaultValue: "Disconnect" })}
            </button>
          ) : (
            <button
              type="button"
              onClick={() => {
                void handleConnect();
              }}
              disabled={!projectId || connecting}
              className="rounded-sm border border-border px-2 py-0.5 hover:bg-accent disabled:cursor-not-allowed disabled:opacity-50"
            >
              {t("terminal.connect", { defaultValue: "Connect" })}
            </button>
          )}
        </div>
      </div>
      <p className="px-2 py-1 text-xs text-muted-foreground" aria-live="polite">
        {status}
      </p>
      <div
        ref={mountRef}
        data-testid="terminal-xterm-mount"
        className="flex-1 overflow-hidden bg-[#0b0b0e]"
      />
      {/* Plain-text mirror — used by screen readers and tests. */}
      <ul
        data-testid="terminal-line-mirror"
        aria-label={t("terminal.mirrorLabel", { defaultValue: "Terminal output mirror" })}
        className="sr-only"
      >
        {recentLines.map((line) => (
          <li key={line.seq} data-seq={line.seq} data-severity={line.highlight?.severity}>
            <span data-testid="terminal-line-ts">{formatTimestamp(line.ts)}</span>
            <span>{line.text}</span>
          </li>
        ))}
      </ul>
    </div>
  );
}

function formatLineForXterm(line: OutputLine): string {
  const ts = formatTimestamp(line.ts);
  return `[${ts}] ${line.text}`;
}

function formatTimestamp(ms: number): string {
  if (!ms) return "00:00:00.000";
  const date = new Date(ms);
  const hh = String(date.getHours()).padStart(2, "0");
  const mm = String(date.getMinutes()).padStart(2, "0");
  const ss = String(date.getSeconds()).padStart(2, "0");
  const msPart = String(date.getMilliseconds()).padStart(3, "0");
  return `${hh}:${mm}:${ss}.${msPart}`;
}

function formatError(err: SpecLensIpcError): string {
  return err.hint ? `${err.message} — ${err.hint}` : err.message;
}
