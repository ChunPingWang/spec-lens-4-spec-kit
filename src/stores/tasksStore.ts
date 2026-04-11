/**
 * Store for the Tasks sub-page (FR-050 – FR-057). Owns the candidate list
 * returned by `tasks_scan_files`, the parsed entries keyed by relative
 * path, and the pinned tab state mirroring `ProjectState`.
 *
 * The store does NOT auto-select any tab — per FR-050 the first entry
 * must be an explicit user action.
 */
import { create } from "zustand";

import { invoke, SpecLensIpcError } from "@/lib/tauri";
import type {
  ProjectState,
  TaskEntry,
  TaskFile,
  TaskFileCandidate,
  TasksParseResponse,
  Uuid,
} from "@/types/ipc";

export interface ParsedTaskFile {
  file: TaskFile;
  entries: TaskEntry[];
}

interface TasksState {
  candidates: TaskFileCandidate[];
  parsed: Record<string, ParsedTaskFile>;
  selectedPaths: string[];
  activePath: string | null;
  loadingScan: boolean;
  loadingParse: Record<string, boolean>;
  error: SpecLensIpcError | null;

  scan: (projectId: Uuid) => Promise<TaskFileCandidate[]>;
  parse: (projectId: Uuid, relativePath: string) => Promise<ParsedTaskFile>;
  setSelected: (
    projectId: Uuid,
    selectedPaths: string[],
    activePath: string | null,
  ) => Promise<void>;
  closeTab: (projectId: Uuid, relativePath: string) => Promise<void>;
  hydrateFromProjectState: (state: ProjectState) => void;
  clear: () => void;
}

export const useTasksStore = create<TasksState>((set, get) => ({
  candidates: [],
  parsed: {},
  selectedPaths: [],
  activePath: null,
  loadingScan: false,
  loadingParse: {},
  error: null,

  scan: async (projectId) => {
    set({ loadingScan: true, error: null });
    try {
      const candidates = await invoke<TaskFileCandidate[]>("tasks_scan_files", {
        projectId,
      });
      set({ candidates, loadingScan: false });
      return candidates;
    } catch (err) {
      set({
        loadingScan: false,
        error: err instanceof SpecLensIpcError ? err : null,
      });
      throw err;
    }
  },

  parse: async (projectId, relativePath) => {
    set((state) => ({
      loadingParse: { ...state.loadingParse, [relativePath]: true },
    }));
    try {
      const res = await invoke<TasksParseResponse>("tasks_parse_file", {
        projectId,
        relativePath,
      });
      const parsed: ParsedTaskFile = { file: res.file, entries: res.entries };
      set((state) => ({
        parsed: { ...state.parsed, [relativePath]: parsed },
        loadingParse: { ...state.loadingParse, [relativePath]: false },
      }));
      return parsed;
    } catch (err) {
      set((state) => ({
        loadingParse: { ...state.loadingParse, [relativePath]: false },
        error: err instanceof SpecLensIpcError ? err : state.error,
      }));
      throw err;
    }
  },

  setSelected: async (projectId, selectedPaths, activePath) => {
    const validated =
      activePath && selectedPaths.includes(activePath) ? activePath : null;
    try {
      await invoke<ProjectState>("tasks_state_set_selected", {
        projectId,
        selectedPaths,
        activePath: validated,
      });
      set({ selectedPaths, activePath: validated });
      // Eagerly parse any newly-selected file that hasn't been parsed yet.
      const missing = selectedPaths.filter((p) => !get().parsed[p]);
      await Promise.all(
        missing.map((p) =>
          get()
            .parse(projectId, p)
            .catch(() => undefined),
        ),
      );
    } catch (err) {
      set({ error: err instanceof SpecLensIpcError ? err : null });
      throw err;
    }
  },

  closeTab: async (projectId, relativePath) => {
    const { selectedPaths, activePath, setSelected } = get();
    const next = selectedPaths.filter((p) => p !== relativePath);
    const nextActive =
      activePath === relativePath ? (next[0] ?? null) : activePath;
    await setSelected(projectId, next, nextActive);
  },

  hydrateFromProjectState: (state) => {
    set({
      selectedPaths: state.selectedTaskFilePaths ?? [],
      activePath: state.activeTaskFilePath ?? null,
    });
  },

  clear: () =>
    set({
      candidates: [],
      parsed: {},
      selectedPaths: [],
      activePath: null,
      loadingScan: false,
      loadingParse: {},
      error: null,
    }),
}));
