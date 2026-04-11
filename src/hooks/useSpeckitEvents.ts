/**
 * Phase 9 US7 — `useSpeckitEvents` (T151 / FR-070).
 *
 * Subscribes to the backend `steps_state_changed` event stream and fires
 * a native OS notification whenever a step transitions to `done` or
 * `failed`, but ONLY when:
 *
 *   1. the user has the global `notificationsEnabled` toggle on, AND
 *   2. the SpecLens window is currently unfocused (the in-app step list
 *      already covers the focused case — duplicating into the OS toast
 *      would be noisy).
 *
 * The dispatch helper itself is in `lib/notify.ts`; this hook only owns
 * the diff logic + window focus check + i18n strings for the messages.
 */
import { useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";

import { subscribe } from "@/lib/tauri";
import { notifySystem } from "@/lib/notify";
import { useConfigStore, useStepsStore } from "@/stores";
import type { Step, StepStatus } from "@/types/ipc";

interface StepsChangedPayload {
  changedStepIds: string[];
  steps: Step[];
}

/** Statuses that should trigger an OS notification on transition. */
const NOTIFIABLE: ReadonlyArray<StepStatus> = ["done", "missing"];

/** Map a Spec-Kit step status to one of our three notification levels. */
function levelForStatus(status: StepStatus): "info" | "warn" | "error" {
  if (status === "done") return "info";
  if (status === "missing") return "error";
  return "info";
}

/** Window-focus probe that's safe in tests (jsdom defaults to focused). */
function isWindowFocused(): boolean {
  if (typeof document === "undefined") return true;
  return document.hasFocus();
}

export function useSpeckitEvents(): void {
  const { t } = useTranslation();
  const setSteps = useStepsStore((s) => s.setSteps);

  // Snapshot of the previous status per step id. Used to detect
  // transitions across event batches without depending on Zustand
  // history (which we don't track).
  const lastStatus = useRef<Map<string, StepStatus>>(new Map());

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let cancelled = false;

    void (async () => {
      unlisten = await subscribe<StepsChangedPayload>("steps_state_changed", (payload) => {
        if (cancelled || !payload?.steps) return;

        // Always refresh the store so the steps panel mirrors the
        // backend even when notifications are off.
        setSteps(payload.steps);

        const enabled = useConfigStore.getState().config?.notificationsEnabled ?? false;
        if (!enabled || isWindowFocused()) {
          // Still update the snapshot so a later transition is computed
          // against the latest known state, not stale data from before
          // the user disabled notifications.
          for (const step of payload.steps) {
            lastStatus.current.set(step.id, step.status);
          }
          return;
        }

        for (const step of payload.steps) {
          const previous = lastStatus.current.get(step.id);
          lastStatus.current.set(step.id, step.status);
          if (previous === step.status) continue;
          if (!NOTIFIABLE.includes(step.status)) continue;

          const titleKey =
            step.status === "done"
              ? "notifications.stepDoneTitle"
              : "notifications.stepFailedTitle";
          const bodyKey =
            step.status === "done"
              ? "notifications.stepDoneBody"
              : "notifications.stepFailedBody";

          void notifySystem(
            t(titleKey, { name: step.displayName }),
            t(bodyKey, { name: step.displayName }),
            levelForStatus(step.status),
          );
        }
      });
    })();

    return () => {
      cancelled = true;
      if (unlisten) unlisten();
    };
  }, [setSteps, t]);
}
