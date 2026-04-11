/**
 * Phase 9 US7 — frontend wrapper for the `notify_system` IPC command
 * (FR-070). The backend already validates and truncates the payload, so
 * this helper is intentionally thin. Errors are swallowed: a missing OS
 * notification must never block step status updates from rendering.
 */
import { invoke, SpecLensIpcError } from "@/lib/tauri";
import type { NotifyLevel, NotifyRequest, PreparedNotification } from "@/types/ipc";

export async function notifySystem(
  title: string,
  body: string,
  level: NotifyLevel = "info",
): Promise<PreparedNotification | null> {
  const request: NotifyRequest = { title, body, level };
  try {
    return await invoke<PreparedNotification>("notify_system", { request });
  } catch (err) {
    if (err instanceof SpecLensIpcError) {
      // Validation errors come from the dispatcher service — surface in
      // the dev console only; UI should not error out on a notification.
      console.warn("notify_system failed", err.code, err.message);
    } else {
      console.warn("notify_system failed", err);
    }
    return null;
  }
}
