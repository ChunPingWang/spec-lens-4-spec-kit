/**
 * Store for the Phase detail pane (Overview / Documents / Tasks).
 * Owns the current step's document list plus the lazily-loaded document
 * content cache for the preview pane. See `specs/001-speclens-desktop/
 * spec.md §US2`.
 */
import { create } from "zustand";

import { invoke, SpecLensIpcError } from "@/lib/tauri";
import type { PhaseDocument, PhaseTab, Uuid } from "@/types/ipc";

export interface PhaseOverviewSlice {
  title: string;
  relativePath: string;
  content: string;
}

export interface PhaseOverviewResponse {
  stepId: string;
  slices: PhaseOverviewSlice[];
}

export interface PhaseDocumentReadResponse {
  relativePath: string;
  content: string;
  truncated: boolean;
  hash: { sha256: string; generatedAt: string; lastVerifiedAt: string } | null;
}

interface PhaseState {
  activeStepId: string | null;
  activeTab: PhaseTab;
  documents: PhaseDocument[];
  overview: PhaseOverviewResponse | null;
  documentCache: Record<string, PhaseDocumentReadResponse>;
  loadingDocuments: boolean;
  loadingOverview: boolean;
  error: SpecLensIpcError | null;

  setActiveStep: (stepId: string | null) => void;
  setActiveTab: (tab: PhaseTab) => void;
  loadDocuments: (projectId: Uuid, stepId: string) => Promise<void>;
  loadOverview: (projectId: Uuid, stepId: string) => Promise<void>;
  readDocument: (projectId: Uuid, relativePath: string) => Promise<PhaseDocumentReadResponse>;
  recomputeHashes: (projectId: Uuid, stepId?: string) => Promise<number>;
  clear: () => void;
}

export const usePhaseStore = create<PhaseState>((set, get) => ({
  activeStepId: null,
  activeTab: "overview",
  documents: [],
  overview: null,
  documentCache: {},
  loadingDocuments: false,
  loadingOverview: false,
  error: null,

  setActiveStep: (stepId) => set({ activeStepId: stepId }),
  setActiveTab: (tab) => set({ activeTab: tab }),

  loadDocuments: async (projectId, stepId) => {
    set({ loadingDocuments: true, error: null });
    try {
      const documents = await invoke<PhaseDocument[]>("phase_documents_list", {
        projectId,
        stepId,
      });
      set({ documents, loadingDocuments: false });
    } catch (err) {
      set({
        loadingDocuments: false,
        error: err instanceof SpecLensIpcError ? err : null,
      });
    }
  },

  loadOverview: async (projectId, stepId) => {
    set({ loadingOverview: true });
    try {
      const overview = await invoke<PhaseOverviewResponse>("phase_overview", {
        projectId,
        stepId,
      });
      set({ overview, loadingOverview: false });
    } catch (err) {
      set({
        loadingOverview: false,
        error: err instanceof SpecLensIpcError ? err : null,
      });
    }
  },

  readDocument: async (projectId, relativePath) => {
    const cached = get().documentCache[relativePath];
    if (cached) return cached;
    const response = await invoke<PhaseDocumentReadResponse>("phase_document_read", {
      projectId,
      relativePath,
    });
    set((state) => ({
      documentCache: { ...state.documentCache, [relativePath]: response },
    }));
    return response;
  },

  recomputeHashes: async (projectId, stepId) => {
    const { updated } = await invoke<{ updated: number }>("phase_recompute_hashes", {
      projectId,
      stepId,
    });
    // Invalidate cached reads; statuses might have flipped.
    set({ documentCache: {} });
    return updated;
  },

  clear: () =>
    set({
      activeStepId: null,
      activeTab: "overview",
      documents: [],
      overview: null,
      documentCache: {},
      error: null,
    }),
}));
