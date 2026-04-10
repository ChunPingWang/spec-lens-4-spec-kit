# Phase 1 Data Model: SpecLens

**Branch**: `001-speclens-desktop` | **Date**: 2026-04-11 | **Plan**: [plan.md](./plan.md)

## Overview

This document defines the logical data model for SpecLens v1. All entities are
owned by the Rust backend and exposed to the React frontend through Tauri IPC
(see [contracts/ipc.md](./contracts/ipc.md)). Fields that are persisted are
marked **(persist: app-data)** for cross-project state or **(persist: project)**
for per-project state under `.speclens/`. Fields without a persistence tag are
derived or in-memory only.

The model is deliberately observational: no entity holds mutable execution
state that would imply Run/Retry/Reset controls (FR-044).

## Entity Relationship Overview

```text
AppConfig ─┬─ UserPreferences
           └─ RecentProject* ──┐
                               │
Project ◄──────────────────────┘
  │
  ├── EnvironmentStatus
  ├── AgentProfile
  ├── Step* ────── PhaseDocument*
  │                     │
  │                     └── DocHashRecord
  ├── TaskFile* ── TaskEntry*
  ├── ProjectState  (per-project persist)
  └── PtySession*
         │
         └── OutputLine* ── HighlightRule (shared, agent-agnostic)
```

(`*` = collection; diamonds omitted for readability.)

## Entities

### 1. AppConfig  (persist: app-data)

Global, process-wide settings shared across all windows.

| Field | Type | Notes |
|-------|------|-------|
| `schemaVersion` | `u32` | For forward-compat migrations. v1 = 1. |
| `language` | `"en" \| "zh-TW" \| "system"` | FR-087 / FR-088. `"system"` means auto-detect from OS locale. |
| `theme` | `"light" \| "dark" \| "system"` | Defaults to `"system"`. |
| `recentProjects` | `RecentProject[]` | FR-001 – FR-005. Capped at 20; LRU. |
| `terminalBufferMaxLines` | `u32` | Default 10_000 (FR-047). |
| `terminalDiskCapMiB` | `u32` | Default 512 (FR-048). User-adjustable. |
| `openWindows` | `WindowHandle[]` | FR-100 – FR-103. Re-opened on next launch. |
| `lastUpdatedAt` | `ISO-8601` | Written on every save. |

**Validation**:
- `language` must be one of the allowed values.
- `terminalBufferMaxLines` ∈ [1_000, 200_000].
- `terminalDiskCapMiB` ∈ [16, 8192].

**State transitions**: none; this is a flat, overwritten document.

---

### 2. RecentProject  (part of AppConfig)

| Field | Type | Notes |
|-------|------|-------|
| `id` | `Uuid` | Stable identifier independent of path. |
| `name` | `string` | Directory basename, user-editable. |
| `path` | `string` (absolute) | Must exist at open time; if missing, flagged `unreachable`. |
| `lastOpenedAt` | `ISO-8601` | Used for LRU ordering. |
| `pinned` | `bool` | Pinned items never age out. |

**Validation**: `path` must be an existing directory at time of opening
(FR-003). `unreachable` items stay in the list but are disabled in the UI.

---

### 3. WindowHandle  (part of AppConfig)

| Field | Type | Notes |
|-------|------|-------|
| `windowId` | `string` | Tauri window label. |
| `projectId` | `Uuid` | References `RecentProject.id`; may be null for welcome windows. |
| `bounds` | `{ x, y, width, height }` | Persisted to restore position. |

**Validation**: multiple `WindowHandle`s MAY share the same `projectId` only
if the user explicitly opened duplicate views (rare); default behavior is to
focus the existing window (FR-101).

---

### 4. Project  (in-memory; root context of a workspace window)

| Field | Type | Notes |
|-------|------|-------|
| `id` | `Uuid` | Matches `RecentProject.id`. |
| `name` | `string` | |
| `rootPath` | `string` | Canonicalized absolute path. |
| `speckitDetected` | `bool` | True if `.specify/` or `specs/` heuristics match. |
| `openedAt` | `ISO-8601` | Window-local. |
| `environment` | `EnvironmentStatus` | Latest snapshot. |
| `agent` | `AgentProfile` | |
| `steps` | `Step[]` | Ordered by canonical Spec-Kit sequence. |
| `taskFiles` | `TaskFile[]` | Scanned lazily. |
| `state` | `ProjectState` | Per-project persisted state (see below). |

**Validation**:
- `rootPath` must be a directory the user has read access to.
- Writes (PTY `cwd`, `.speclens/` I/O) are locked to `rootPath` (FR-085).

---

### 5. EnvironmentStatus  (in-memory; refreshed on demand)

Represents FR-010 – FR-013 environment detection results.

| Field | Type | Notes |
|-------|------|-------|
| `speckitInstalled` | `bool` | |
| `speckitVersion` | `SemVer?` | Null if not installed. |
| `speckitPath` | `string?` | Resolved binary path. |
| `latestKnownVersion` | `SemVer?` | From release manifest; cached. |
| `updateAvailable` | `bool` | Derived: `latestKnownVersion > speckitVersion`. |
| `checkedAt` | `ISO-8601` | Timestamp of last probe. |
| `issues` | `EnvIssue[]` | Missing dep, PATH problem, etc. |

**State transitions** (badge color):
- `green` → `speckitInstalled && !updateAvailable && issues.isEmpty`
- `amber` → `speckitInstalled && (updateAvailable || issues.isWarn)`
- `red`   → `!speckitInstalled || issues.hasError`

---

### 6. AgentProfile  (in-memory; cached in project state)

Represents FR-060 / FR-061.

| Field | Type | Notes |
|-------|------|-------|
| `id` | `string` | One of 12 known IDs or `"generic"`. |
| `displayName` | `string` | Localized label. |
| `iconKey` | `string` | Maps to `lucide-react` or bundled svg. |
| `detectedFrom` | `"project" \| "path" \| "none"` | Source of detection. |
| `version` | `string?` | If the agent CLI can self-report. |
| `highlightRuleSet` | `"universal"` | v1 uses one rule set for all agents. |

**Validation**: if no agent is detected, `id = "generic"` and the UI shows an
"unrecognized agent" banner without blocking the workspace.

---

### 7. Step  (in-memory; derived by `phase_scanner`)

Represents a single Spec-Kit step (e.g., `/speckit.specify`).

| Field | Type | Notes |
|-------|------|-------|
| `id` | `string` | Stable slug (e.g., `"specify"`, `"plan"`, `"tasks"`). |
| `order` | `u32` | Canonical ordering in the steps list. |
| `displayName` | `string` | Localized. |
| `status` | `StepStatus` | See enum below. |
| `phaseDocuments` | `PhaseDocument[]` | Overview / Documents sub-pages read from this list. |
| `taskFilePaths` | `string[]` | Relative paths of task files for this step (tasks step only). |
| `lastObservedAt` | `ISO-8601?` | Set whenever output is routed to this step. |

**Enum `StepStatus`** (purely observational):

| Value | Meaning |
|-------|---------|
| `not_started` | No phase documents and no terminal activity observed. |
| `in_progress` | Terminal activity observed for this step but no completion signal. |
| `done`        | All expected phase documents exist and hashes are `generated`. |
| `modified`    | Phase documents exist but at least one has a `modified` hash. |
| `missing`     | Phase documents expected but at least one is absent. |
| `unknown`     | State cannot be determined (e.g., pre-Spec-Kit project). |

**State transitions**:

```text
not_started ─(pty output matches step)──► in_progress
in_progress ─(all docs present + hashes OK)──► done
done        ─(user edits doc)──► modified
done / modified ─(doc deleted)──► missing
any         ─(FS scan inconclusive)──► unknown
```

No transition ever triggers a user-invisible action; status is purely a
*report* of the file system and terminal observations.

---

### 8. PhaseDocument  (in-memory; derived by `phase_scanner`)

| Field | Type | Notes |
|-------|------|-------|
| `relativePath` | `string` | Relative to project root, POSIX style. |
| `displayName` | `string` | Usually basename. |
| `kind` | `"spec" \| "plan" \| "research" \| "data-model" \| "contract" \| "quickstart" \| "tasks" \| "other"` | Classification heuristic. |
| `hash` | `DocHashRecord?` | Null if not yet hashed. |
| `status` | `DocStatus` | Derived; see enum. |
| `size` | `u64` | Bytes. |
| `modifiedAt` | `ISO-8601` | From FS. |

**Enum `DocStatus`**:

| Value | Meaning |
|-------|---------|
| `generated` | File exists and hash matches the recorded "generated" hash. |
| `modified`  | File exists and hash differs from the recorded "generated" hash. |
| `missing`   | File does not exist. |
| `unverified` | File exists but no baseline hash has been recorded yet. |

---

### 9. DocHashRecord  (persist: project — `.speclens/doc-hashes.json`)

| Field | Type | Notes |
|-------|------|-------|
| `path` | `string` | Project-relative POSIX path. Key of the JSON map. |
| `sha256` | `string` (hex) | SHA-256 of file bytes, lowercase hex. |
| `generatedAt` | `ISO-8601` | First time we saw the file after a Spec-Kit command. |
| `lastVerifiedAt` | `ISO-8601` | Last time hash was recomputed and matched. |

**Validation**:
- `sha256` must be 64 lowercase hex characters.
- Records for files that no longer exist are kept for 30 days, then GC'd.

---

### 10. TaskFile  (in-memory; derived by `task_file_scanner`)

| Field | Type | Notes |
|-------|------|-------|
| `relativePath` | `string` | e.g., `specs/001-foo/tasks.md`. |
| `format` | `"md" \| "json" \| "yaml" \| "txt"` | From extension, validated by parser. |
| `displayName` | `string` | Basename without extension. |
| `taskCount` | `u32` | Total entries. |
| `completedCount` | `u32` | Completed entries. |
| `parsedAt` | `ISO-8601` | |

---

### 11. TaskEntry  (in-memory; produced by `tasks_parser`)

| Field | Type | Notes |
|-------|------|-------|
| `id` | `string` | Stable within file: `"<fileHash>:<line>"`. |
| `title` | `string` | |
| `status` | `"todo" \| "in_progress" \| "done" \| "skipped"` | Mapped from checkbox / JSON field. |
| `sectionPath` | `string[]` | Heading breadcrumb for markdown. |
| `filePath` | `string` | Same as parent `TaskFile.relativePath`. |
| `line` | `u32` | 1-based line number. |
| `raw` | `string` | Original source line or JSON/YAML slice for "open in editor". |

**Validation**: `status` must normalize to one of the four enum values; unknown
checkbox characters become `"todo"` and emit a warning in structured logs.

---

### 12. ProjectState  (persist: project — `.speclens/state.json`)

Per-project state survives across sessions; one file per project root.

| Field | Type | Notes |
|-------|------|-------|
| `schemaVersion` | `u32` | v1 = 1. |
| `lastStepId` | `string?` | Step the user last viewed. |
| `lastPhaseTab` | `"overview" \| "documents" \| "tasks"?` | FR-030 – FR-036. |
| `selectedTaskFilePaths` | `string[]` | Task files the user pinned to tabs (FR-050 – FR-057). |
| `activeTaskFilePath` | `string?` | Currently focused tab. |
| `terminalCollapsed` | `bool` | Layout preference within this project. |
| `lastOpenedAt` | `ISO-8601` | |

**Validation**:
- `activeTaskFilePath` MUST be present in `selectedTaskFilePaths` if non-null.
- Paths MUST resolve under the project root.

---

### 13. PtySession  (in-memory; one per window)

Represents an attached terminal (FR-040 – FR-048).

| Field | Type | Notes |
|-------|------|-------|
| `sessionId` | `Uuid` | |
| `windowId` | `string` | |
| `projectId` | `Uuid` | |
| `cwd` | `string` | Equal to `Project.rootPath` (FR-085). |
| `shell` | `string` | Resolved user shell. |
| `startedAt` | `ISO-8601` | |
| `buffer` | `RingBuffer<OutputLine>` | In-memory rolling buffer. |
| `spillFile` | `string` | `.speclens/logs/<sessionId>.ndjson`. |
| `diskBytesUsed` | `u64` | Enforced against `AppConfig.terminalDiskCapMiB`. |
| `attachedStepId` | `string?` | Current step the output is being routed to. |

**Lifecycle**:

```text
creating → attached ──(window close)──► closing → closed
attached ─(shell exits)──► closed
```

A closed session flushes remaining buffer to the spill file and removes
itself from the session table. The spill file is retained until the
per-project disk cap forces eviction (LRU by `sessionId` age).

---

### 14. OutputLine  (in-memory; element of `PtySession.buffer` and spill file)

| Field | Type | Notes |
|-------|------|-------|
| `seq` | `u64` | Monotonic within a session. |
| `ts` | `u64` | Millisecond Unix timestamp. |
| `stream` | `"stdout" \| "stderr"` | |
| `text` | `string` | Raw bytes decoded as UTF-8 (lossy). |
| `ansiSpans` | `AnsiSpan[]?` | Pre-parsed for fast rendering. |
| `highlight` | `HighlightMatch?` | First matching rule, if any. |
| `stepId` | `string?` | Step this line was routed to. |

**Validation**: `ts` monotonicity within a session is enforced; a regression
forces the line to adopt the previous `ts` plus 1 and emits a warning log.

---

### 15. HighlightRule  (static; loaded from bundled JSON)

Agent-agnostic rules applied to all OutputLines (FR-040 – FR-046).

| Field | Type | Notes |
|-------|------|-------|
| `id` | `string` | |
| `severity` | `"info" \| "success" \| "warn" \| "error"` | Drives color. |
| `pattern` | `regex` | Rust regex syntax; anchored where appropriate. |
| `description` | `string` | Shown in the rule-editor tooltip (read-only in v1). |
| `capture` | `{ stepId?: string }` | If present, a match routes the line to this step. |

**Validation**: regex MUST compile at load time; a broken rule aborts load with
an actionable error (Constitution IV: "what happened / why / what to do next").

---

### 16. EnvIssue  (in-memory; member of EnvironmentStatus)

| Field | Type | Notes |
|-------|------|-------|
| `kind` | `"missing_binary" \| "version_mismatch" \| "path_not_writable" \| "other"` | |
| `severity` | `"warn" \| "error"` | |
| `message` | `string` | Localized, actionable. |
| `hint` | `string?` | Suggested remediation text (e.g., install command). |

---

## Persistence Layout Summary

```text
<OS app-data>/SpecLens/
└── app-config.json                 # AppConfig + RecentProject[] + WindowHandle[]

<project-root>/.speclens/
├── state.json                      # ProjectState
├── doc-hashes.json                 # DocHashRecord map
└── logs/
    └── <sessionId>.ndjson          # Spilled OutputLine history per PtySession
```

## Derived / Transient Data (not persisted)

- `Project.environment`, `Project.steps`, `Project.taskFiles`,
  `Project.agent` are rebuilt every time a project opens.
- `PtySession.buffer` is never persisted as a whole; only the spilled tail is
  on disk.
- `HighlightRule`s are compiled from a bundled static resource at startup.

## Change-Data Flows

| Source event | Updates | Notified via IPC event |
|--------------|---------|------------------------|
| Folder opened | `Project`, `EnvironmentStatus`, `AgentProfile`, `Step[]` | `project_opened` |
| FS watcher fires under `specs/` | `PhaseDocument`, `DocHashRecord`, `Step.status` | `phase_documents_changed` |
| FS watcher fires under task files | `TaskFile`, `TaskEntry[]` | `task_files_changed` |
| PTY bytes arrive | `PtySession.buffer`, possibly `Step.status` | `pty_output`, `steps_state_changed` |
| User edits settings | `AppConfig` | `app_config_changed` |
| User picks task file tab | `ProjectState` | `project_state_changed` |

All events are scoped per window so one workspace never redraws because
another workspace received data.

## Invariants

1. Every `Step.id` is unique within a project and stable across sessions.
2. Every `TaskEntry.id` is unique within its `TaskFile` and deterministic for
   a given file content (used for test snapshots).
3. `PtySession.cwd == Project.rootPath` always holds (FR-085).
4. `DocHashRecord.sha256` is the canonical witness of "generated" state;
   modifying the record outside the backend is undefined behavior.
5. `AppConfig.terminalDiskCapMiB` ≥ total size of all `.speclens/logs/*.ndjson`
   across all open projects; the oldest session is evicted first when the cap
   is exceeded.
6. No entity stores user code, secrets, or network-retrieved content that
   could leave the device (FR-086 / SC-010).
