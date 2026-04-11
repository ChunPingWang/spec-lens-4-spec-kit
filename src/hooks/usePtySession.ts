/**
 * Subscribes the current Tauri window to `pty_output`, `pty_closed`, and
 * `buffer_spilled` events for the store's active session, forwarding each
 * batch of lines to the provided `onLines` callback and to the terminal
 * store's `recentLines` mirror. T101 / FR-041.
 */
import { useEffect } from "react";

import { subscribe } from "@/lib/tauri";
import { useTerminalStore } from "@/stores";
import type { OutputLine, Uuid } from "@/types/ipc";

interface PtyOutputPayload {
  sessionId: Uuid;
  lines: OutputLine[];
}

interface PtyClosedPayload {
  sessionId: Uuid;
}

interface BufferSpilledPayload {
  sessionId: Uuid;
  evictedCount: number;
}

interface UsePtySessionOptions {
  onLines?: (lines: OutputLine[]) => void;
  onClosed?: () => void;
  onSpilled?: (count: number) => void;
}

export function usePtySession(options: UsePtySessionOptions = {}): void {
  const descriptor = useTerminalStore((s) => s.descriptor);
  const appendLines = useTerminalStore((s) => s.appendLines);
  const { onLines, onClosed, onSpilled } = options;

  useEffect(() => {
    if (!descriptor) return;
    let unlistenOutput: (() => void) | null = null;
    let unlistenClosed: (() => void) | null = null;
    let unlistenSpill: (() => void) | null = null;
    let cancelled = false;

    void (async () => {
      unlistenOutput = await subscribe<PtyOutputPayload>("pty_output", (payload) => {
        if (cancelled || payload.sessionId !== descriptor.sessionId) return;
        appendLines(payload.lines);
        onLines?.(payload.lines);
      });
      unlistenClosed = await subscribe<PtyClosedPayload>("pty_closed", (payload) => {
        if (cancelled || payload.sessionId !== descriptor.sessionId) return;
        onClosed?.();
      });
      unlistenSpill = await subscribe<BufferSpilledPayload>("buffer_spilled", (payload) => {
        if (cancelled || payload.sessionId !== descriptor.sessionId) return;
        onSpilled?.(payload.evictedCount);
      });
    })();

    return () => {
      cancelled = true;
      unlistenOutput?.();
      unlistenClosed?.();
      unlistenSpill?.();
    };
  }, [descriptor, appendLines, onLines, onClosed, onSpilled]);
}
