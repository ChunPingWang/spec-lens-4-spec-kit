/**
 * Store for the Spec-Kit steps panel. Tracks the current step list and
 * the user's selected step. Full wiring (IPC + live events) lands with
 * Phase 3 US1 acceptance tests.
 */
import { create } from "zustand";

import type { Step } from "@/types/ipc";

interface StepsState {
  steps: Step[];
  selectedStepId: string | null;
  setSteps: (steps: Step[]) => void;
  selectStep: (id: string | null) => void;
  completionPercent: () => number;
}

export const useStepsStore = create<StepsState>((set, get) => ({
  steps: [],
  selectedStepId: null,
  setSteps: (steps) => {
    set({ steps });
    // Preserve selection if it still exists; otherwise reset.
    const selected = get().selectedStepId;
    if (selected && !steps.some((s) => s.id === selected)) {
      set({ selectedStepId: null });
    }
  },
  selectStep: (id) => set({ selectedStepId: id }),
  completionPercent: () => {
    const { steps } = get();
    if (steps.length === 0) return 0;
    const done = steps.filter((s) => s.status === "done").length;
    return Math.round((done / steps.length) * 100);
  },
}));
