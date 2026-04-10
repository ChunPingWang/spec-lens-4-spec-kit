# Phase 1 Contract: Tauri IPC Commands & Events

**Branch**: `001-speclens-desktop` | **Date**: 2026-04-11 | **Plan**: [../plan.md](../plan.md)

## Purpose

Define the contract between the Rust backend (Tauri core) and the React
frontend. Every user-visible interaction in SpecLens v1 happens through one
of the commands or events below. All payloads use JSON-compatible shapes and
reference entities defined in [../data-model.md](../data-model.md).

Per the observe-only constraint (FR-044), **no command may start, retry, or
reset a Spec-Kit step**. Commands are read / list / watch / acknowledge only;
the PTY session is the single exception and it is strictly a passthrough for
whatever the user types.

## Conventions

- Commands are invoked via `tauri::command` handlers; the TypeScript side
  calls them through `@tauri-apps/api/core`'s `invoke<T>(name, args)`.
- Errors are returned as `{ "code": string, "message": string, "hint"?: string }`
  (matching `IpcError` in Rust via `thiserror`). Error messages MUST follow
  the Constitution IV "what happened / why / what to do next" shape.
- All time fields are ISO-8601 strings; all IDs are UUIDs unless otherwise
  stated.
- All commands that touch a project require a `projectId` or an open Tauri
  window that resolves to one; cross-window isolation is enforced in Rust.
- Events are emitted via `tauri::Window::emit` **scoped to the target window
  label**. The frontend listens with `listen<T>(eventName, handler)`.

## Command Catalog

### A. Project Management (FR-001 – FR-005)

#### `project_pick_directory`
- **Args**: `{}`
- **Returns**: `{ "path": string } | null` (null if user cancels)
- **Notes**: Opens the OS folder picker. Roams freely under FR-085 read-only
  exception. Does not open the project yet.

#### `project_open`
- **Args**: `{ "path": string, "windowId"?: string }`
- **Returns**: `{ "project": Project, "state": ProjectState }`
- **Errors**:
  - `E_PATH_NOT_FOUND` — path does not exist
  - `E_PATH_NOT_DIR` — path is a file, not a directory
  - `E_PATH_NOT_READABLE` — missing read permission
- **Side effects**:
  - Adds / updates a `RecentProject` in `AppConfig`.
  - Starts an FS watcher for `specs/`, `.specify/`, and task-file locations.
  - Lazily starts a `PtySession` when the user first attaches the terminal.
  - Emits `project_opened` on the target window.

#### `project_list_recent`
- **Args**: `{}`
- **Returns**: `RecentProject[]` (sorted by `lastOpenedAt` desc, pinned first)

#### `project_remove_recent`
- **Args**: `{ "projectId": string }`
- **Returns**: `RecentProject[]` (new list)

#### `project_pin_recent`
- **Args**: `{ "projectId": string, "pinned": boolean }`
- **Returns**: `RecentProject[]`

#### `project_close`
- **Args**: `{ "windowId": string }`
- **Returns**: `{}`
- **Side effects**: Closes the FS watcher and PTY session bound to that window,
  flushes spill logs, and persists `ProjectState`.

---

### B. Environment & Spec-Kit Detection (FR-010 – FR-013)

#### `env_check`
- **Args**: `{ "projectId": string, "force"?: boolean }`
- **Returns**: `EnvironmentStatus`
- **Notes**:
  - Runs asynchronously; result is also emitted on `env_status_changed`.
  - `force: true` bypasses the in-memory cache.

#### `env_get_install_guide`
- **Args**: `{ "platform"?: "macos" \| "windows" \| "linux" }`
- **Returns**: `{ "steps": InstallGuideStep[], "releaseNotesUrl": string }`
- **Notes**: Static payload bundled with the app; used by the
  `EnvCheckDialog`. `releaseNotesUrl` is the only outbound link v1 exposes
  and is opened by the OS, not fetched inside the app (FR-086).

---

### C. Steps & Phase Documents (FR-020 – FR-036)

#### `steps_list`
- **Args**: `{ "projectId": string }`
- **Returns**: `Step[]`

#### `steps_get`
- **Args**: `{ "projectId": string, "stepId": string }`
- **Returns**: `Step`
- **Errors**: `E_STEP_NOT_FOUND`

#### `phase_get_overview`
- **Args**: `{ "projectId": string, "stepId": string }`
- **Returns**: `{ "stepId": string, "slices": PhaseOverviewSlice[] }`
- **Notes**: Used by the Overview sub-page. Slices are lazy sections derived
  from the step's documents (e.g., "Summary", "Constitution Check").

#### `phase_get_documents`
- **Args**: `{ "projectId": string, "stepId": string }`
- **Returns**: `PhaseDocument[]`

#### `phase_read_document`
- **Args**: `{ "projectId": string, "relativePath": string, "maxBytes"?: number }`
- **Returns**: `{ "relativePath": string, "content": string, "truncated": boolean, "hash": DocHashRecord | null }`
- **Errors**:
  - `E_DOC_NOT_FOUND`
  - `E_DOC_TOO_LARGE` (falls back to `truncated: true` up to `maxBytes`)
- **Notes**: Read-only; never writes.

#### `phase_recompute_hashes`
- **Args**: `{ "projectId": string, "stepId"?: string }`
- **Returns**: `{ "updated": number }`
- **Side effects**: Emits `phase_documents_changed` when any record changes.

---

### D. Task Files & Task Entries (FR-050 – FR-057)

#### `tasks_scan_files`
- **Args**: `{ "projectId": string }`
- **Returns**: `TaskFile[]`
- **Notes**: Triggers a full re-scan; also runs automatically when the FS
  watcher sees a candidate file.

#### `tasks_parse_file`
- **Args**: `{ "projectId": string, "relativePath": string }`
- **Returns**: `{ "file": TaskFile, "entries": TaskEntry[] }`
- **Errors**:
  - `E_TASK_FILE_NOT_FOUND`
  - `E_TASK_PARSE_FAILED` — with `hint` pointing at the offending line.

#### `tasks_pick_file`
- **Args**: `{ "projectId": string }`
- **Returns**: `{ "relativePath": string } | null`
- **Notes**: Opens an OS file picker restricted to the project directory
  (FR-085). Returns null on cancel.

#### `tasks_state_set_selected`
- **Args**: `{ "projectId": string, "selectedPaths": string[], "activePath"?: string }`
- **Returns**: `ProjectState`
- **Validation**: `activePath` MUST be in `selectedPaths` if provided.
- **Side effects**: Persists `ProjectState`; emits `project_state_changed`.

---

### E. Terminal Bridge (FR-040 – FR-048)

#### `terminal_attach`
- **Args**: `{ "projectId": string, "windowId": string, "shell"?: string }`
- **Returns**: `{ "sessionId": string, "cwd": string }`
- **Errors**:
  - `E_PTY_SPAWN_FAILED` — includes OS error in `hint`.
- **Notes**: Lazy; only called when the user clicks the terminal pane for
  the first time in a window. `cwd` is always the project root (FR-085).

#### `terminal_detach`
- **Args**: `{ "sessionId": string }`
- **Returns**: `{}`
- **Side effects**: Flushes buffer to spill file; emits `pty_closed`.

#### `terminal_write`
- **Args**: `{ "sessionId": string, "data": string }`
- **Returns**: `{ "bytes": number }`
- **Notes**: Pure passthrough. The backend does not parse or filter the user's
  input — it only echoes through the PTY.

#### `terminal_resize`
- **Args**: `{ "sessionId": string, "cols": number, "rows": number }`
- **Returns**: `{}`

#### `terminal_get_slice`
- **Args**: `{ "sessionId": string, "fromSeq"?: number, "toSeq"?: number, "maxLines"?: number }`
- **Returns**: `{ "lines": OutputLine[], "firstSeq": number, "lastSeq": number, "hasMore": boolean }`
- **Notes**: Used for retrospective inspection (scroll-up, step drill-down).
  Efficient against both the in-memory ring buffer and the on-disk spill.

#### `terminal_search`
- **Args**: `{ "sessionId": string, "query": string, "regex"?: boolean, "limit"?: number }`
- **Returns**: `{ "matches": OutputLineRef[] }`
- **Notes**: Scans buffer + spill; results reference `seq` and byte offsets.

#### `terminal_get_config`
- **Args**: `{}`
- **Returns**: `{ "bufferMaxLines": number, "diskCapMiB": number }`

#### `terminal_set_config`
- **Args**: `{ "bufferMaxLines"?: number, "diskCapMiB"?: number }`
- **Returns**: `{ "bufferMaxLines": number, "diskCapMiB": number }`
- **Validation**: ranges per `AppConfig` validation rules.

---

### F. Agent Detection (FR-060 / FR-061)

#### `agent_detect`
- **Args**: `{ "projectId": string }`
- **Returns**: `AgentProfile`
- **Notes**: Checks project root and `PATH`; falls back to `"generic"` without
  erroring.

#### `agent_list_known`
- **Args**: `{}`
- **Returns**: `AgentProfile[]` (the 12 official profiles + generic)

---

### G. App Config & Localization (FR-086 – FR-092, FR-100 – FR-103)

#### `config_get`
- **Args**: `{}`
- **Returns**: `AppConfig`

#### `config_set_language`
- **Args**: `{ "language": "en" \| "zh-TW" \| "system" }`
- **Returns**: `AppConfig`
- **Side effects**: Emits `app_config_changed` to all windows.

#### `config_set_theme`
- **Args**: `{ "theme": "light" \| "dark" \| "system" }`
- **Returns**: `AppConfig`

#### `config_set_terminal_limits`
- **Args**: `{ "bufferMaxLines"?: number, "diskCapMiB"?: number }`
- **Returns**: `AppConfig`

---

### H. Window & Session (FR-100 – FR-103)

#### `window_open_project`
- **Args**: `{ "projectId": string }`
- **Returns**: `{ "windowId": string }`
- **Notes**: If the project is already open in a window, focuses it instead
  of creating a duplicate.

#### `window_close`
- **Args**: `{ "windowId": string }`
- **Returns**: `{}`

#### `window_list`
- **Args**: `{}`
- **Returns**: `WindowHandle[]`

---

### I. Notifications (FR-070)

#### `notify_system`
- **Args**: `{ "title": string, "body": string, "level"?: "info" \| "warn" \| "error" }`
- **Returns**: `{}`
- **Notes**: Thin wrapper over `tauri-plugin-notification`; requested by the
  frontend when a step status changes while the window is not focused.

---

## Event Catalog

All events are per-window unless otherwise noted. Events never carry secrets
or raw file content beyond the minimum needed to refresh a view.

### 1. `project_opened`
- **Payload**: `{ "project": Project, "state": ProjectState }`
- **Fires**: Once, after `project_open` completes.

### 2. `project_state_changed`
- **Payload**: `ProjectState`
- **Fires**: Whenever the per-project state is persisted (tabs, active file,
  collapse preferences).

### 3. `env_status_changed`
- **Payload**: `EnvironmentStatus`
- **Fires**: After an `env_check` or background re-probe completes.

### 4. `steps_state_changed`
- **Payload**: `{ "changedStepIds": string[], "steps": Step[] }`
- **Fires**: When step statuses are recomputed (FS change, PTY routing, hash
  recompute).

### 5. `phase_documents_changed`
- **Payload**: `{ "stepId": string, "documents": PhaseDocument[] }`
- **Fires**: When the FS watcher or `phase_recompute_hashes` mutates any
  PhaseDocument for a step.

### 6. `task_files_changed`
- **Payload**: `{ "files": TaskFile[] }`
- **Fires**: When the task-file scanner detects additions/removals/edits.

### 7. `task_entries_changed`
- **Payload**: `{ "relativePath": string, "entries": TaskEntry[] }`
- **Fires**: When a parsed task file's entries change (saved edit, external tool).

### 8. `pty_output`
- **Payload**: `{ "sessionId": string, "lines": OutputLine[] }`
- **Fires**: Batched every ~16 ms for smooth rendering at 60 fps (FR-081).
- **Backpressure**: If the frontend cannot keep up, the backend coalesces
  batches into larger ones and drops the `ansiSpans` hint; full lines are
  never dropped from the buffer.

### 9. `pty_closed`
- **Payload**: `{ "sessionId": string, "reason": "user" \| "shell_exit" \| "error", "exitCode"?: number }`

### 10. `agent_detected`
- **Payload**: `AgentProfile`
- **Fires**: On initial detection and whenever the user re-runs `agent_detect`.

### 11. `app_config_changed`
- **Payload**: `AppConfig`
- **Scope**: Broadcast to **all** windows (one of the few global events).

### 12. `window_list_changed`
- **Payload**: `WindowHandle[]`
- **Scope**: Broadcast to all windows.

### 13. `buffer_spilled`
- **Payload**: `{ "sessionId": string, "bytesOnDisk": number, "linesEvicted": number }`
- **Fires**: When the rolling buffer spills lines to disk or evicts an old
  session due to the project cap (FR-047 / FR-048). Used for a small
  status-bar indicator.

### 14. `ipc_error` (diagnostic)
- **Payload**: `{ "command": string, "code": string, "message": string }`
- **Fires**: Anytime a command returns an error, mirrored for the global
  error reporter. Not used for control flow; the invoker still receives
  the error directly.

## Error Catalog

| Code | Category | Typical Trigger |
|------|----------|-----------------|
| `E_PATH_NOT_FOUND` | project | User picks a removed directory |
| `E_PATH_NOT_DIR` | project | User picks a file |
| `E_PATH_NOT_READABLE` | project | Permission error |
| `E_STEP_NOT_FOUND` | steps | Stale step id from cached UI |
| `E_DOC_NOT_FOUND` | phase | File deleted between scan and read |
| `E_DOC_TOO_LARGE` | phase | File exceeds `maxBytes`; returned with partial content |
| `E_TASK_FILE_NOT_FOUND` | tasks | Parse requested on missing file |
| `E_TASK_PARSE_FAILED` | tasks | Malformed JSON/YAML or unsupported schema |
| `E_PTY_SPAWN_FAILED` | terminal | OS refuses PTY allocation |
| `E_PTY_SESSION_NOT_FOUND` | terminal | Stale `sessionId` |
| `E_PTY_WRITE_FAILED` | terminal | Underlying PTY write errored |
| `E_CONFIG_INVALID` | config | Out-of-range buffer / cap |
| `E_WINDOW_NOT_FOUND` | window | Stale window id |
| `E_INTERNAL` | generic | Fallback for unexpected errors; logged with `tracing` |

## Contract Test Coverage

Per Constitution III (TDD), every command and event above will have a
failing contract test authored before its implementation. The test list
below is the minimum:

- `project_open` with missing / non-dir / ok inputs
- `project_list_recent` LRU and pinned ordering
- `env_check` with and without Spec-Kit installed (via fixture PATH)
- `steps_list` against a fixture project with 0, 1, and N phase documents
- `phase_read_document` truncation boundary
- `phase_recompute_hashes` updates `DocHashRecord` deterministically
- `tasks_parse_file` for `.md` / `.json` / `.yaml` / `.txt` fixtures
- `tasks_state_set_selected` rejects `activePath` not in `selectedPaths`
- `terminal_attach` locks `cwd` to project root (FR-085)
- `terminal_write` passthrough echo (round-trip via `pty_output`)
- `terminal_set_config` range validation
- `agent_detect` with each of the 12 known agents and `"generic"` fallback
- `config_set_language` emits `app_config_changed` to all windows
- `window_open_project` focuses existing window when already open
- Every event payload is serde-round-trip stable (snapshot tests)

These tests will be enumerated in `tasks.md` during `/speckit.tasks`.
