/**
 * Thin typed wrapper around `@tauri-apps/api/core` `invoke` and the
 * event system. Command handlers in `src-tauri/src/commands/*` expose
 * typed responses; the helpers below convert rejected promises to our
 * canonical `IpcError` shape so UI code can always rely on
 * `{ code, message, hint? }`.
 */
import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn, type Event as TauriEvent } from "@tauri-apps/api/event";

import type { IpcError } from "@/types/ipc";
import { isIpcError } from "@/types/ipc";

export class SpecLensIpcError extends Error implements IpcError {
  code: string;
  hint?: string;

  constructor(source: IpcError) {
    super(source.message);
    this.code = source.code;
    this.hint = source.hint;
    this.name = "SpecLensIpcError";
  }
}

/**
 * Invoke a Tauri command and normalise rejection values to
 * `SpecLensIpcError`. Prefer this over `invoke` directly.
 */
export async function invoke<T>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return (await tauriInvoke(cmd, args)) as T;
  } catch (err) {
    if (isIpcError(err)) {
      throw new SpecLensIpcError(err);
    }
    throw new SpecLensIpcError({
      code: "E_INTERNAL",
      message: err instanceof Error ? err.message : String(err),
    });
  }
}

/** Subscribe to a Tauri event and return an unlisten function. */
export async function subscribe<T>(
  event: string,
  handler: (payload: T) => void,
): Promise<UnlistenFn> {
  return listen<T>(event, (ev: TauriEvent<T>) => handler(ev.payload));
}
