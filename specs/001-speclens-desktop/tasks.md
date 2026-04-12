# Tasks: SpecLens — Cross-Platform Visual Management for GitHub Spec-Kit

**Branch**: `001-speclens-desktop` | **Date**: 2026-04-11
**Input**: [spec.md](./spec.md), [plan.md](./plan.md), [research.md](./research.md), [data-model.md](./data-model.md), [contracts/ipc.md](./contracts/ipc.md), [quickstart.md](./quickstart.md)

**Tests**: Included. Constitution III (`.specify/memory/constitution.md` §III) designates TDD as NON-NEGOTIABLE — every behavior-bearing task below is preceded by a failing test task.

**Organization**: Grouped by user story (US1–US7) so each story can be implemented, tested, and shipped independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel with other `[P]` tasks in the same phase (different files, no dependencies).
- **[Story]**: Maps the task to a user story (US1…US7). Setup, Foundational, and Polish phases have no story label.
- Every task includes an exact file path.

## Path Conventions

- Rust backend lives under `src-tauri/src/…` (Cargo crate `speclens`).
- React frontend lives under `src/…` (Vite + TypeScript).
- Rust benches live under `src-tauri/benches/…` (criterion).
- E2E tests live under `tests/e2e/…` (Playwright + tauri-driver).
- Per-feature fixtures live under `tests/fixtures/…` and are consumed by both Rust and E2E layers.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Bring up the Tauri v2 monorepo so every subsequent task has a working build, lint, and test loop.

- [ ] T001 Create the Tauri v2 monorepo skeleton at the repository root with `src/`, `src-tauri/`, `tests/`, `public/` directories per `plan.md` Project Structure
- [ ] T002 [P] Initialize `package.json` with React 18, Vite, Tailwind, shadcn/ui, @xterm/xterm v5, @tanstack/react-virtual, Zustand, i18next, react-i18next, marked, highlight.js, lucide-react, Vitest, React Testing Library, Playwright, and TypeScript 5.4 dev deps; commit as `package.json` at repo root
- [ ] T003 [P] Initialize `src-tauri/Cargo.toml` with Tauri v2 (`tauri`, `tauri-plugin-fs`, `tauri-plugin-shell`, `tauri-plugin-notification`, `tauri-plugin-store`), `tokio`, `serde`, `serde_json`, `serde_yaml`, `reqwest`, `portable-pty`, `notify`, `sha2`, `hex`, `semver`, `chrono`, `tracing`, `tracing-subscriber`, `thiserror`, `anyhow`, plus dev deps `pretty_assertions`, `insta`, `criterion`
- [ ] T004 [P] Create `src-tauri/tauri.conf.json` with macOS / Windows / Linux bundle identifiers, window defaults, and CSP matching `plan.md`
- [ ] T005 [P] Create `vite.config.ts`, `tsconfig.json`, `tailwind.config.ts`, `postcss.config.js`, and `.eslintrc.cjs` with strict TypeScript, React, and a11y plugins
- [ ] T006 [P] Create `src-tauri/rustfmt.toml` and `src-tauri/clippy.toml` enforcing `-D warnings` per Constitution I
- [ ] T007 [P] Create `.github/workflows/ci.yml` running `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`, `cargo bench -- --test`, `pnpm lint`, `pnpm typecheck`, `pnpm test`, `pnpm test:e2e` on macOS / Windows / Ubuntu matrix
- [ ] T008 [P] Create `.github/workflows/build.yml` triggering on version tags, running `pnpm tauri build` on the three OSes with artifact upload
- [ ] T009 [P] Create `src-tauri/src/main.rs` and `src-tauri/src/lib.rs` bootstrap that registers an empty Tauri builder; runs `pnpm tauri dev` to a blank window
- [ ] T010 [P] Create `src/main.tsx`, `src/App.tsx`, and `src/index.css` rendering a placeholder "SpecLens" page styled with Tailwind tokens
- [ ] T011 [P] Create `tests/fixtures/README.md` and the four sample fixtures used throughout the task list: `tests/fixtures/speckit-project-ok/`, `tests/fixtures/speckit-project-empty/`, `tests/fixtures/speckit-project-broken/`, `tests/fixtures/task-files/` (md, json, yaml, txt samples)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared types, error handling, persistence, IPC scaffolding, i18n, and design tokens that every user story depends on.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

### Shared Rust foundation

- [ ] T012 Create `src-tauri/src/error.rs` defining `IpcError` via `thiserror` with the error codes from `contracts/ipc.md §Error Catalog` and a `Result<T>` alias
- [ ] T013 [P] Create `src-tauri/src/models/mod.rs` and re-export empty stubs for every entity in `data-model.md` (AppConfig, RecentProject, WindowHandle, Project, EnvironmentStatus, AgentProfile, Step, PhaseDocument, DocHashRecord, TaskFile, TaskEntry, ProjectState, PtySession, OutputLine, HighlightRule, EnvIssue)
- [ ] T014 [P] Write `src-tauri/src/models/app_config.rs` implementing `AppConfig`, `RecentProject`, `WindowHandle` with serde, validation ranges, and SemVer `schemaVersion = 1`
- [ ] T015 [P] Write `src-tauri/src/models/project.rs`, `step.rs`, `phase_document.rs`, `doc_hash.rs` with serde round-trip and invariants from `data-model.md §Invariants`
- [ ] T016 [P] Write `src-tauri/src/models/task_file.rs` and `task_entry.rs` with the status enum normalization specified in `data-model.md §11`
- [ ] T017 [P] Write `src-tauri/src/models/pty.rs` defining `PtySession`, `OutputLine`, `AnsiSpan`, and `HighlightMatch`
- [ ] T018 [P] Write `src-tauri/src/models/agent.rs` defining `AgentProfile` with the 12-agent enum + generic
- [ ] T019 [P] Write `src-tauri/src/models/env.rs` defining `EnvironmentStatus`, `EnvIssue`, and the green/amber/red derivation helper

### Shared persistence layer

- [ ] T020 Write failing Rust test `src-tauri/src/services/app_data_store.rs` `#[cfg(test)] mod tests` asserting round-trip read/write of `AppConfig` via `tauri-plugin-store` under a temp dir (env override `SPECLENS_APP_DATA_DIR`)
- [ ] T021 Implement `src-tauri/src/services/app_data_store.rs` to make T020 pass, including LRU ordering for `recentProjects` and `schemaVersion` migration stub
- [ ] T022 Write failing Rust test `src-tauri/src/services/state_store.rs` asserting round-trip of `ProjectState` under a per-project `.speclens/state.json`
- [ ] T023 Implement `src-tauri/src/services/state_store.rs` to make T022 pass, including default fallbacks when `.speclens/` is missing or unreadable (FR-092)
- [ ] T024 [P] Write failing Rust test `src-tauri/src/services/fs_watcher.rs` asserting a debounced event is emitted when a file in a fixture project is touched
- [ ] T025 Implement `src-tauri/src/services/fs_watcher.rs` wrapping `notify` + `debouncer-full` to make T024 pass

### IPC scaffolding

- [ ] T026 Create `src-tauri/src/commands/mod.rs` and empty modules `project.rs`, `speckit.rs`, `terminal.rs`, `phase.rs`, `tasks.rs`, `agent.rs`, `config.rs`, each exposing an empty `pub fn register(app: &mut tauri::App)` helper
- [ ] T027 Wire `src-tauri/src/lib.rs` to register all command modules and install the Tauri store plugin, FS watcher singleton, and tracing subscriber with env filter `SPECLENS_LOG`

### Shared frontend foundation

- [ ] T028 [P] Create `src/lib/tauri.ts` that wraps `invoke<T>` and `listen<T>` with typed command / event names from `contracts/ipc.md`
- [ ] T029 [P] Create `src/types/ipc.ts` with TypeScript type mirrors of every Rust serde model in `data-model.md`
- [ ] T030 [P] Create `src/i18n/index.ts` initializing i18next with `en` and `zh-TW` resources, OS locale detection, and a Zustand-friendly language setter (FR-087 / FR-088)
- [ ] T031 [P] Seed `src/i18n/en.json` and `src/i18n/zh-TW.json` with the minimal keys needed by the Welcome page and the error toast system
- [ ] T032 [P] Create `src/stores/configStore.ts` (Zustand) mirroring `AppConfig` and subscribing to the `app_config_changed` IPC event
- [ ] T033 [P] Create `src/stores/projectStore.ts`, `src/stores/stepsStore.ts`, `src/stores/phaseStore.ts`, `src/stores/tasksStore.ts`, `src/stores/terminalStore.ts`, `src/stores/agentStore.ts`, each exporting empty slice factories
- [ ] T034 [P] Create `src/components/layout/AppShell.tsx`, `TopBar.tsx`, `ResizablePanel.tsx` using shadcn/ui primitives; enforce keyboard navigation and WCAG 2.1 AA contrast tokens
- [ ] T035 [P] Create `src/components/ui/ErrorToast.tsx` rendering the Constitution IV error format "what happened / why / what to do next" from an `IpcError` payload
- [ ] T036 [P] Create `src/lib/highlightRules.ts` loading the bundled default rule set (a static JSON at `src/lib/default-highlight-rules.json`) exported for both frontend preview and Rust loader

### Shared test plumbing

- [ ] T037 [P] Create `vitest.config.ts` and `src/test/setup.ts` initializing `@testing-library/jest-dom` and a mock Tauri `invoke` / `listen` bridge
- [ ] T038 [P] Create `playwright.config.ts` with a `tauri-driver` project targeting the dev binary and a fixture path override for the three sample projects
- [ ] T039 [P] Create `src-tauri/tests/common/mod.rs` exposing helpers: `fixture_project(name) -> TempDir`, `with_app_data_dir(|dir| …)`, `touch(path)`
- [ ] T040 [P] Create `src-tauri/benches/bench_main.rs` criterion harness with empty group stubs for `terminal_buffer_slice`, `tasks_parser`, `highlight_rules`, `doc_hash_recompute`

**Checkpoint**: Foundation ready. User story phases can now begin in parallel by different developers.

---

## Phase 3: User Story 1 — Open Project & See Progress (Priority: P1) 🎯 MVP

**Goal**: Pick a folder, open it as a workspace, detect the agent, list the Spec-Kit steps with status dots, and show a top progress bar; remember recent projects; restore last step + tab on reopen.

**Independent Test**: From the Welcome page, pick `tests/fixtures/speckit-project-ok`; the workspace renders within 3 s showing the project name, agent badge, 6-state step list, and an overall progress bar; closing and reopening restores the last selected step from `.speclens/state.json` and surfaces the project in the recent-projects list.

### Tests for User Story 1 ⚠️ (write first, ensure FAILING)

- [ ] T041 [P] [US1] Contract test `src-tauri/tests/cmd_project_pick_directory.rs` asserting `project_pick_directory` returns `{ path: string } | null` and validates path existence
- [ ] T042 [P] [US1] Contract test `src-tauri/tests/cmd_project_open.rs` covering `E_PATH_NOT_FOUND`, `E_PATH_NOT_DIR`, `E_PATH_NOT_READABLE`, and the happy path returning `{ project, state }` against `tests/fixtures/speckit-project-ok`
- [ ] T043 [P] [US1] Contract test `src-tauri/tests/cmd_project_recent.rs` covering `project_list_recent` LRU order, `project_pin_recent` pinning, and `project_remove_recent`
- [ ] T044 [P] [US1] Contract test `src-tauri/tests/cmd_project_close.rs` asserting `.speclens/state.json` is flushed and the FS watcher is released on close
- [ ] T045 [P] [US1] Contract test `src-tauri/tests/cmd_steps_list.rs` asserting the canonical Spec-Kit step list is returned for a fixture project with `pending`, `in_progress`, and `done` steps
- [ ] T046 [P] [US1] Rust unit test `src-tauri/src/services/phase_scanner.rs::tests` asserting `Step.status` transitions from `not_started → in_progress → done → modified → missing` per `data-model.md §7`
- [ ] T047 [P] [US1] Vitest `src/pages/Welcome.test.tsx` asserting the recent-projects list renders with empty state, with items, and handles click-to-open
- [ ] T048 [P] [US1] Vitest `src/components/steps/StepsList.test.tsx` asserting rendering of the six step states with distinct status dot classes
- [ ] T049 [P] [US1] Vitest `src/components/steps/ProgressBar.test.tsx` asserting `X / Y steps completed` label and percentage calculation
- [ ] T050 [P] [US1] Playwright `tests/e2e/open-project.spec.ts` covering: Welcome → pick fixture → workspace renders with steps + progress within 3 s → close → reopen restores last step

### Implementation for User Story 1

- [ ] T051 [US1] Implement `src-tauri/src/services/phase_scanner.rs` to read `.specify/` and `specs/` layouts and produce an ordered `Vec<Step>` with status derived purely from FS observations
- [ ] T052 [US1] Implement `src-tauri/src/commands/project.rs::project_pick_directory` using `tauri-plugin-dialog`
- [ ] T053 [US1] Implement `src-tauri/src/commands/project.rs::project_open` — canonicalize path, validate, register FS watcher scope, upsert `RecentProject`, load `ProjectState`, emit `project_opened`
- [ ] T054 [US1] Implement `project_list_recent`, `project_remove_recent`, `project_pin_recent` in `src-tauri/src/commands/project.rs` backed by `app_data_store`
- [ ] T055 [US1] Implement `project_close` in `src-tauri/src/commands/project.rs` flushing `ProjectState` and tearing down FS + PTY resources for the window
- [ ] T056 [US1] Implement `src-tauri/src/commands/speckit.rs::steps_list` and `steps_get` calling `phase_scanner` and returning serde models
- [ ] T057 [P] [US1] Create `src/pages/Welcome.tsx` rendering the recent-projects list from `configStore`, a "Pick a project" button, and drag-and-drop handling routed through `project_pick_directory`
- [ ] T058 [P] [US1] Create `src/pages/Workspace.tsx` as the per-window shell that dispatches `project_open` on mount and listens to `project_opened`
- [ ] T059 [P] [US1] Create `src/components/steps/StepsList.tsx`, `StepItem.tsx`, `ProgressBar.tsx` consuming `stepsStore` with `@tanstack/react-virtual`
- [ ] T060 [P] [US1] Create `src/hooks/useSpeckitEvents.ts` subscribing to `steps_state_changed`, `project_opened`, `project_state_changed` and pushing updates into the relevant Zustand stores
- [ ] T061 [US1] Wire the `TopBar` to show project name, path, last-modified time, and placeholder agent badge (real data lands in US6) per FR-005
- [ ] T062 [US1] Persist last selected step to `ProjectState.lastStepId` via `tasks_state_set_selected`'s sibling helper in `state_store`; restore on project open

**Checkpoint**: At this point US1 is independently demo-able — a user can open a project and see accurate progress, and recent projects / last step persist across relaunches.

---

## Phase 4: User Story 2 — Drill Into a Phase (Priority: P1)

**Goal**: Each step exposes Overview / Documents / (Tasks for the `tasks` step only); Documents shows `generated | modified | missing | unverified` badges; clicking a doc opens an inline preview.

**Independent Test**: Open `tests/fixtures/speckit-project-ok`, click the `plan` step; Overview shows step metadata and a terminal-slice placeholder; Documents lists `plan.md`, `research.md`, `data-model.md`, `contracts/ipc.md`, `quickstart.md` with correct badges; clicking `plan.md` shows the rendered Markdown. Click the `tasks` step; a third Tasks sub-page appears (panel content lives in US4).

### Tests for User Story 2 ⚠️ (write first, ensure FAILING)

- [ ] T063 [P] [US2] Contract test `src-tauri/tests/cmd_phase_get_overview.rs` asserting slice extraction for a fixture `plan.md`
- [ ] T064 [P] [US2] Contract test `src-tauri/tests/cmd_phase_get_documents.rs` covering the four `DocStatus` values
- [ ] T065 [P] [US2] Contract test `src-tauri/tests/cmd_phase_read_document.rs` covering happy path and `E_DOC_TOO_LARGE` truncation boundary
- [ ] T066 [P] [US2] Contract test `src-tauri/tests/cmd_phase_recompute_hashes.rs` asserting that editing a fixture doc flips its status from `generated` to `modified`
- [ ] T067 [P] [US2] Rust unit test `src-tauri/src/services/doc_hash_store.rs::tests` asserting SHA-256 hex length 64, atomic write, and 30-day GC for stale records
- [ ] T068 [P] [US2] Vitest `src/components/phase/PhaseTabs.test.tsx` asserting the Tasks sub-page appears only for the `tasks` step (FR-030)
- [ ] T069 [P] [US2] Vitest `src/components/phase/DocumentItem.test.tsx` asserting each badge state renders with the correct a11y label and tooltip text
- [ ] T070 [P] [US2] Vitest `src/components/phase/DocumentPreview.test.tsx` asserting Markdown, JSON pretty-print, and syntax-highlighted code branches
- [ ] T071 [P] [US2] Playwright `tests/e2e/phase-drilldown.spec.ts` covering click-through from StepsList to DocumentPreview

### Implementation for User Story 2

- [ ] T072 [US2] Implement `src-tauri/src/services/doc_hash_store.rs` reading/writing `.speclens/doc-hashes.json` with SHA-256 + `chrono` timestamps
- [ ] T073 [US2] Extend `src-tauri/src/services/phase_scanner.rs` to compute `PhaseDocument[]` per step with the four `DocStatus` values and hook the FS watcher to emit `phase_documents_changed`
- [ ] T074 [US2] Implement `src-tauri/src/commands/phase.rs::phase_get_overview` producing `PhaseOverviewSlice[]` from document headings
- [ ] T075 [US2] Implement `phase_get_documents` and `phase_read_document` (with `maxBytes` truncation) in `src-tauri/src/commands/phase.rs`
- [ ] T076 [US2] Implement `phase_recompute_hashes` in `src-tauri/src/commands/phase.rs` emitting `phase_documents_changed` on delta
- [ ] T077 [P] [US2] Create `src/components/phase/PhaseTabs.tsx` and `PhaseTabPanel.tsx` with locked/unlocked tab behavior per FR-032
- [ ] T078 [P] [US2] Create `src/components/phase/OverviewPanel.tsx` rendering step metadata and a collapsible terminal-slice placeholder (filled in US3)
- [ ] T079 [P] [US2] Create `src/components/phase/DocumentList.tsx` and `DocumentItem.tsx` with badges (`generated`, `modified`, `missing`, `unverified`) sourced from `phaseStore`
- [ ] T080 [P] [US2] Create `src/components/phase/DocumentPreview.tsx` using `marked` + `highlight.js` with lazy loading
- [ ] T081 [US2] Wire `src/hooks/useSpeckitEvents.ts` to refresh `phaseStore` on `phase_documents_changed` and update step status on `steps_state_changed`

**Checkpoint**: US1 + US2 deliver the dashboard + drill-down experience end-to-end (minus live terminal and tasks, which follow).

---

## Phase 5: User Story 3 — Live Terminal with Universal Highlights (Priority: P1)

**Goal**: A bottom terminal panel streams PTY output with ANSI color, per-line millisecond timestamps, universal highlight rules, filters/search, and bounded memory via rolling buffer + disk spill.

**Independent Test**: Attach the terminal in the opened fixture project; run `echo hello`; the line appears in ≤ 100 ms (FR-081) with a timestamp; a `✓` line gets the success highlight; filtering hides non-matching lines without erasing history; exceeding the buffer cap spills to `.speclens/logs/<session>.ndjson`.

### Tests for User Story 3 ⚠️ (write first, ensure FAILING)

- [ ] T082 [P] [US3] Rust unit test `src-tauri/src/services/output_buffer.rs::tests` asserting ring-buffer eviction and spill-file append for `OutputLine[]`
- [ ] T083 [P] [US3] Rust unit test `src-tauri/src/services/highlight_rules.rs::tests` using `insta` snapshots for each FR-045 rule (success, error, running, warning, JSON, file-path)
- [ ] T084 [P] [US3] Contract test `src-tauri/tests/cmd_terminal_attach.rs` asserting `cwd` is locked to the project root (FR-085) and `E_PTY_SPAWN_FAILED` hint propagation
- [ ] T085 [P] [US3] Contract test `src-tauri/tests/cmd_terminal_write.rs` asserting echo round-trip via `pty_output` event
- [ ] T086 [P] [US3] Contract test `src-tauri/tests/cmd_terminal_get_slice.rs` asserting transparent read across in-memory and on-disk tiers for a session that overflowed the buffer (FR-047)
- [ ] T087 [P] [US3] Contract test `src-tauri/tests/cmd_terminal_search.rs` asserting regex and plain search across both tiers
- [ ] T088 [P] [US3] Contract test `src-tauri/tests/cmd_terminal_set_config.rs` asserting `bufferMaxLines ∈ [1_000, 200_000]` and `diskCapMiB ∈ [16, 8192]` validation (FR-048)
- [ ] T089 [P] [US3] Criterion bench `src-tauri/benches/bench_main.rs::terminal_buffer_slice` asserting ≤ 1 ms per 1 000-line slice retrieval
- [ ] T090 [P] [US3] Vitest `src/components/terminal/TerminalPanel.test.tsx` asserting xterm mount, auto-scroll lock, and line-timestamp rendering
- [ ] T091 [P] [US3] Playwright `tests/e2e/terminal-stream.spec.ts` asserting `echo` round-trip latency under 100 ms at p95

### Implementation for User Story 3

- [ ] T092 [US3] Implement `src-tauri/src/services/output_buffer.rs` with an in-memory `VecDeque<OutputLine>` capped at `bufferMaxLines` and an append-only `.speclens/logs/<sessionId>.ndjson` spill writer
- [ ] T093 [US3] Implement `src-tauri/src/services/highlight_rules.rs` loading a bundled JSON rule set, compiling regexes at load time, and applying the first matching rule per line
- [ ] T094 [US3] Implement `src-tauri/src/services/terminal_bridge.rs` spawning `portable-pty`, reading stdout/stderr into `OutputLine`, batching emits every ~16 ms per FR-081, and enforcing the disk cap + `buffer_spilled` event
- [ ] T095 [US3] Implement `src-tauri/src/commands/terminal.rs::terminal_attach`, `terminal_detach`, `terminal_write`, `terminal_resize` passing through to `terminal_bridge`
- [ ] T096 [US3] Implement `terminal_get_slice`, `terminal_search`, `terminal_get_config`, `terminal_set_config` in `src-tauri/src/commands/terminal.rs`
- [ ] T097 [US3] Wire `terminal_bridge` to route PTY output lines to the currently attached step based on regex rules and publish `steps_state_changed` when a line transitions a step
- [ ] T098 [P] [US3] Create `src/components/terminal/TerminalPanel.tsx` wrapping `@xterm/xterm` with addon-fit, addon-search, addon-web-links, addon-unicode11 and consuming `terminalStore`
- [ ] T099 [P] [US3] Create `src/components/terminal/TerminalToolbar.tsx` (filters, regex search, clear view, export `.log`/`.txt`, copy, auto-scroll lock) per FR-042 / FR-043
- [ ] T100 [P] [US3] Create `src/components/terminal/QuickCommands.tsx` behind the optional `allowTerminalForwarding` setting (FR-044); disabled by default
- [ ] T101 [P] [US3] Create `src/hooks/usePtySession.ts` subscribing to `pty_output`, `pty_closed`, `buffer_spilled` and feeding the xterm instance
- [ ] T102 [US3] Integrate the terminal slice into `OverviewPanel.tsx` (from US2) using `terminal_get_slice` bounded by the step's run window (FR-033)

**Checkpoint**: US1 + US2 + US3 deliver the core dashboard, drill-down, and live terminal.

---

## Phase 6: User Story 4 — Task File Picker + Checklist (Priority: P1)

**Goal**: First Tasks-tab entry always shows the picker, never auto-loads. Selected files appear as persistent tabs in the Task File Bar (even with one file), support per-tab filter/search, combined + per-file progress bars with color thresholds, and expandable rows with description / links / 30 s terminal slice / error summary.

**Independent Test**: Open `tests/fixtures/speckit-project-ok` → Tasks tab → picker shows scan results with nothing pre-checked → pick `tasks.md` → Task File Bar shows one tab with a completion badge; "+ add file" adds a second tab; filter chips behave per-tab; expand a row to see detail and a ≈30 s terminal slice.

### Tests for User Story 4 ⚠️ (write first, ensure FAILING)

- [ ] T103 [P] [US4] Rust unit test `src-tauri/src/services/tasks_parser.rs::tests` using `insta` snapshots for `.md`, `.json`, `.yaml`, `.txt` fixtures in `tests/fixtures/task-files/`
- [ ] T104 [P] [US4] Rust unit test `src-tauri/src/services/task_file_scanner.rs::tests` asserting scan returns candidates without auto-selecting any (FR-050)
- [ ] T105 [P] [US4] Contract test `src-tauri/tests/cmd_tasks_scan_files.rs` against a fixture with 0, 1, and many task files
- [ ] T106 [P] [US4] Contract test `src-tauri/tests/cmd_tasks_parse_file.rs` covering `E_TASK_PARSE_FAILED` with actionable `hint`
- [ ] T107 [P] [US4] Contract test `src-tauri/tests/cmd_tasks_state_set_selected.rs` asserting rejection when `activePath` not in `selectedPaths`
- [ ] T108 [P] [US4] Criterion bench `src-tauri/benches/bench_main.rs::tasks_parser` targeting ≤ 5 ms per 1 000-line task file
- [ ] T109 [P] [US4] Vitest `src/components/tasks/TaskFileBar.test.tsx` asserting the 0 / 1 / many tab rendering rules (FR-053)
- [ ] T110 [P] [US4] Vitest `src/components/tasks/TaskFilePicker.test.tsx` asserting no items are pre-checked and the confirm button is disabled until ≥ 1 selection (FR-050)
- [ ] T111 [P] [US4] Vitest `src/components/tasks/TasksProgressPanel.test.tsx` asserting the amber / blue / green color thresholds and combined + per-file bars (FR-055)
- [ ] T112 [P] [US4] Vitest `src/components/tasks/TaskItem.test.tsx` asserting expansion shows description, related docs, "≈ 30 s before completion" slice, and error summary
- [ ] T113 [P] [US4] Playwright `tests/e2e/tasks-first-entry.spec.ts` covering: open project → tasks tab → picker → confirm → Task File Bar → add file → filter per tab

### Implementation for User Story 4

- [ ] T114 [US4] Implement `src-tauri/src/services/tasks_parser.rs` with four parsers (md checkbox regex, JSON `{tasks:[]}`, YAML same shape, txt one-per-line) producing a common `TaskEntry`
- [ ] T115 [US4] Implement `src-tauri/src/services/task_file_scanner.rs` walking the project with `ignore` crate heuristics, respecting `.gitignore`
- [ ] T116 [US4] Implement `src-tauri/src/commands/tasks.rs::tasks_scan_files`, `tasks_parse_file`, `tasks_pick_file` (restricted OS dialog), `tasks_state_set_selected`
- [ ] T117 [US4] Extend `state_store` to persist `selectedTaskFilePaths` and `activeTaskFilePath` per project and reload automatically (FR-052)
- [ ] T118 [P] [US4] Create `src/components/tasks/TasksPanel.tsx` wiring the picker → file-bar → list flow
- [ ] T119 [P] [US4] Create `src/components/tasks/TaskFilePicker.tsx` with scan results + manual "select path" + "select file" affordances, nothing pre-checked
- [ ] T120 [P] [US4] Create `src/components/tasks/TaskFileBar.tsx` with always-visible rendering, parent-directory disambiguation, completion badge, close button, overflow arrows, "+ add file" control
- [ ] T121 [P] [US4] Create `src/components/tasks/TaskFilter.tsx` (All / Completed / Incomplete chips, search box, sort incomplete-first) scoped per tab
- [ ] T122 [P] [US4] Create `src/components/tasks/TaskItem.tsx` and `TaskDetail.tsx` with expandable detail, related-doc links that jump to US2's DocumentPreview, 30 s terminal slice via `terminal_get_slice`, error summary
- [ ] T123 [P] [US4] Create `src/components/tasks/TasksProgressPanel.tsx` showing combined + per-file progress bars with the FR-055 color thresholds
- [ ] T124 [US4] Wire `TasksPanel` into `PhaseTabPanel` for the `tasks` step only and emit the first-write `.gitignore` recommendation prompt per FR-091

**Checkpoint**: All four P1 stories are independently demo-able; this is the product MVP.

---

## Phase 7: User Story 5 — Environment Check & Update Flow (Priority: P2)

**Goal**: Every project open runs an env check; block the workspace if Spec-Kit CLI is missing; offer in-app install with live progress; show an update banner with changelog preview when outdated; keep working offline.

**Independent Test**: Set `PATH` to a dir without `spec-kit` → open a project → env dialog blocks workspace with install guidance → click install (mocked) → progress shown → on success the workspace loads. With an outdated version fixture, an amber update banner is shown.

### Tests for User Story 5 ⚠️ (write first, ensure FAILING)

- [ ] T125 [P] [US5] Rust unit test `src-tauri/src/services/version_checker.rs::tests` asserting SemVer comparison, network-failure fallback (FR-013), and cached `latestKnownVersion`
- [ ] T126 [P] [US5] Contract test `src-tauri/tests/cmd_env_check.rs` with three scenarios: not installed, outdated, current
- [ ] T127 [P] [US5] Contract test `src-tauri/tests/cmd_env_get_install_guide.rs` asserting bundled payload per platform
- [ ] T128 [P] [US5] Vitest `src/components/env/EnvCheckDialog.test.tsx` asserting green/amber/red rendering and actionable messages per Constitution IV
- [ ] T129 [P] [US5] Playwright `tests/e2e/env-check.spec.ts` covering the three scenarios end-to-end

### Implementation for User Story 5

- [ ] T130 [US5] Implement `src-tauri/src/services/version_checker.rs` probing `spec-kit --version`, comparing against a cached manifest, and falling back gracefully offline
- [ ] T131 [US5] Implement `src-tauri/src/commands/speckit.rs::env_check` and `env_get_install_guide` returning `EnvironmentStatus` and a static guide payload
- [ ] T132 [P] [US5] Create `src/components/env/EnvCheckDialog.tsx` blocking modal with green/amber/red states and actionable messages
- [ ] T133 [P] [US5] Create `src/components/env/InstallGuide.tsx` and `src/components/env/UpdateBanner.tsx` with "Update now" / "Skip" actions and progress indicator
- [ ] T134 [US5] Wire `Workspace.tsx` to call `env_check` on mount, gate the main layout behind a successful result, and listen to `env_status_changed`

**Checkpoint**: P2 env-check protects the P1 flows without blocking users who already have Spec-Kit installed.

---

## Phase 8: User Story 6 — AI Agent Compatibility (Priority: P2)

**Goal**: Detect which of the 12 supported agents is in use via the strict priority chain (Spec-Kit config `--ai` → FS fingerprints → `"generic"` fallback); show agent badge; keep all features working in generic mode.

**Independent Test**: For each of `tests/fixtures/agent-{claude,copilot,gemini,cursor,windsurf,amazonq,codex,qwen,opencode,kilocode,auggie,roo,generic}/`, open the project and confirm the correct agent badge and "auto-detected" / "configured" / "generic" label.

### Tests for User Story 6 ⚠️ (write first, ensure FAILING)

- [ ] T135 [P] [US6] Rust unit test `src-tauri/src/services/agent_detector.rs::tests` covering the three-step priority chain (FR-061) for all 12 agents plus generic fallback
- [ ] T136 [P] [US6] Contract test `src-tauri/tests/cmd_agent_detect.rs` against the 13 fixture projects
- [ ] T137 [P] [US6] Contract test `src-tauri/tests/cmd_agent_list_known.rs` asserting 12 + 1 profiles
- [ ] T138 [P] [US6] Vitest `src/components/agent/AgentBadge.test.tsx` asserting correct icon, name, and label per detection source
- [ ] T139 [P] [US6] Playwright `tests/e2e/agent-generic.spec.ts` asserting that every feature remains available in a `generic` project

### Implementation for User Story 6

- [ ] T140 [US6] Create `tests/fixtures/agent-*` fixture projects covering Claude Code, Copilot, Gemini CLI, Cursor, Windsurf, Amazon Q, Codex CLI, Qwen Code, opencode, Kilo Code, Auggie CLI, Roo Code, and a generic-only project
- [ ] T141 [US6] Implement `src-tauri/src/services/agent_detector.rs` with the priority chain (config → fingerprint → generic) returning `AgentProfile`
- [ ] T142 [US6] Implement `src-tauri/src/commands/agent.rs::agent_detect` and `agent_list_known`
- [ ] T143 [P] [US6] Create `src/components/agent/AgentBadge.tsx` and `AgentIcon.tsx` rendering localized agent names while keeping brand names untranslated (FR-087)
- [ ] T144 [US6] Wire `TopBar.tsx` to show the real agent badge and emit `agent_detected`; update `projectStore` on change

**Checkpoint**: Every feature built so far works for all 12 agents + generic.

---

## Phase 9: User Story 7 — Native System Notifications (Priority: P3)

**Goal**: OS-native notifications on step complete / fail; user-toggleable; environment results surface as in-app toasts.

**Independent Test**: Enable notifications → trigger a fixture step completion → see OS notification; trigger a failure → see failure notification; disable → re-run → no OS notification but in-app state still updates.

### Tests for User Story 7 ⚠️ (write first, ensure FAILING)

- [ ] T145 [P] [US7] Contract test `src-tauri/tests/cmd_notify_system.rs` covering the info / warn / error levels
- [ ] T146 [P] [US7] Vitest `src/stores/configStore.test.ts` asserting the `notificationsEnabled` toggle persists through `config_set_*`
- [ ] T147 [P] [US7] Playwright `tests/e2e/notifications.spec.ts` running the enabled / disabled scenarios

### Implementation for User Story 7

- [ ] T148 [US7] Implement `src-tauri/src/commands/config.rs::notify_system` wrapping `tauri-plugin-notification`
- [ ] T149 [US7] Extend `AppConfig` with `notificationsEnabled: bool` and implement `config_set_notifications_enabled`
- [ ] T150 [P] [US7] Create `src/components/settings/NotificationToggle.tsx` wired to `configStore`
- [ ] T151 [US7] Hook `useSpeckitEvents.ts` to call `notify_system` on `steps_state_changed` transitions to `done` or `failed` when the toggle is on and the window is unfocused

**Checkpoint**: All seven user stories shipped.

---

## Phase 10: Polish & Cross-Cutting Concerns

**Purpose**: Performance gates, a11y, zh-TW completeness, security audit, docs validation.

- [X] T152 [P] Add criterion bench `src-tauri/benches/bench_main.rs::highlight_rules` asserting ≤ 10 µs per line across the default rule set
- [X] T153 [P] Add criterion bench `src-tauri/benches/bench_main.rs::doc_hash_recompute` asserting ≤ 5 ms for a 1 MiB Markdown file
- [ ] T154 [P] Add Playwright performance probe `tests/e2e/perf-cold-start.spec.ts` asserting p95 cold start ≤ 2 s (SC-001 / FR-080)
- [ ] T155 [P] Add Playwright probe `tests/e2e/perf-workspace-render.spec.ts` asserting p95 ≤ 3 s from folder pick to full workspace render (SC-002)
- [ ] T156 [P] Complete `src/i18n/zh-TW.json` translations for every string key used in Phases 3–9, leaving product name, agent names, and Spec-Kit CLI output untranslated (FR-087)
- [X] T157 [P] Add `src/i18n/i18n.test.ts` asserting `en` and `zh-TW` resource files have identical key trees and non-empty string values
- [ ] T158 [P] Add axe-core a11y checks in `tests/e2e/a11y.spec.ts` covering Welcome, Workspace, TasksPanel, and EnvCheckDialog (WCAG 2.1 AA)
- [ ] T159 [P] Add `tests/e2e/safety-two-tier.spec.ts` asserting PTY `cwd` and file writes stay within the project root (FR-085 / FR-093)
- [ ] T160 [P] Add `src-tauri/tests/network_audit.rs` asserting zero outbound network calls in steady-state after env check (SC-010)
- [ ] T161 [P] Add CI job in `.github/workflows/ci.yml` running `cargo llvm-cov --workspace --fail-under-lines 80` and `pnpm test -- --coverage` with the same threshold (Constitution II)
- [ ] T162 [P] Add CI benchmark gate in `.github/workflows/ci.yml` that fails when any criterion group regresses > 10 % vs. the last tagged baseline (Constitution V)
- [ ] T163 [P] Run `specs/001-speclens-desktop/quickstart.md` §4 smoke flow as an automated check and record results in `tests/e2e/quickstart.spec.ts`
- [X] T164 [P] Wire the first-write `.gitignore` recommendation for `.speclens/` (FR-091) into a unit-tested helper `src-tauri/src/services/gitignore_hint.rs`
- [ ] T165 [P] Add `src-tauri/src/commands/window.rs` with `window_open_project`, `window_close`, `window_list`, and tests asserting focus-existing-window behavior (FR-100 / FR-101)
- [ ] T166 [P] Add `tests/e2e/multi-window.spec.ts` asserting that two windows on different projects keep independent PTY sessions, buffers, and tab state (FR-101)
- [X] T167 [P] Verify and update `CLAUDE.md` agent context with the final tech stack after all phases are merged
- [ ] T168 Run `cargo audit` and `pnpm audit --prod` and resolve any high/critical findings (Additional Constraint: Security)
- [ ] T169 Final manual review of every FR in `spec.md` → mapped task → passing test, recorded in `specs/001-speclens-desktop/checklists/requirements.md` Notes section

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies — can start immediately.
- **Phase 2 (Foundational)**: Depends on Phase 1. Blocks all user stories.
- **Phase 3 (US1)**: Depends on Phase 2.
- **Phase 4 (US2)**: Depends on Phase 2 (can run in parallel with US1 if another dev is available; the `TopBar` hand-off in T061 is the only soft coupling).
- **Phase 5 (US3)**: Depends on Phase 2 (can run in parallel with US1/US2). US2 integration point T102 lands after both are otherwise complete.
- **Phase 6 (US4)**: Depends on Phase 2. Uses `terminal_get_slice` from US3 for the 30 s preview (T122), so it is easier to merge after US3.
- **Phase 7 (US5)**: Depends on Phase 2. Independent of US1–US4 but is gated into the `Workspace` mount sequence (T134).
- **Phase 8 (US6)**: Depends on Phase 2. Replaces the placeholder agent badge from T061.
- **Phase 9 (US7)**: Depends on Phase 2 plus US1 event plumbing.
- **Phase 10 (Polish)**: Depends on every targeted user story being merged.

### User Story Dependencies

- **US1 (P1)**: No dependencies beyond Phase 2.
- **US2 (P1)**: No hard dependency on US1 (uses the same stores) — can be built in parallel.
- **US3 (P1)**: Independent; merges a hook into US2's `OverviewPanel` at T102.
- **US4 (P1)**: Independent; uses US3's `terminal_get_slice` at T122.
- **US5 (P2)**: Gated into `Workspace.tsx` (US1) at T134 but otherwise isolated.
- **US6 (P2)**: Replaces the placeholder in US1's `TopBar` at T144.
- **US7 (P3)**: Hooks into `steps_state_changed` (US1 event stream) at T151.

### Within Each User Story

- Contract / unit tests (marked ⚠️) MUST be written and failing before implementation tasks.
- Rust models → services → commands → frontend components → integration.
- Story is considered complete only when its Playwright integration test passes.

### Parallel Opportunities

- Phase 1: T002–T011 are all `[P]`.
- Phase 2: T013–T019, T028–T040 are all `[P]`.
- Phase 3: T041–T050 (tests) run in parallel; T057–T060 run in parallel after T051–T056.
- Phase 4: T063–T071 run in parallel; T077–T080 run in parallel after T072–T076.
- Phase 5: T082–T091 run in parallel; T098–T101 run in parallel after T092–T097.
- Phase 6: T103–T113 run in parallel; T118–T123 run in parallel after T114–T117.
- Phase 7: T125–T129 and T132–T133 parallelize respectively.
- Phase 8: T135–T139 and T143 parallelize respectively.
- Phase 9: T145–T147 and T150 parallelize respectively.
- Phase 10: T152–T167 are almost all `[P]`.

---

## Parallel Example: User Story 1 Tests

```bash
# Run all US1 contract + unit + component tests together (all [P]):
cargo test --test cmd_project_pick_directory      # T041
cargo test --test cmd_project_open                # T042
cargo test --test cmd_project_recent              # T043
cargo test --test cmd_project_close               # T044
cargo test --test cmd_steps_list                  # T045
cargo test -p speclens phase_scanner              # T046
pnpm vitest src/pages/Welcome.test.tsx            # T047
pnpm vitest src/components/steps/StepsList.test.tsx   # T048
pnpm vitest src/components/steps/ProgressBar.test.tsx # T049
pnpm playwright test tests/e2e/open-project.spec.ts   # T050 (must fail until impl)
```

---

## Implementation Strategy

### MVP First (US1 only)

1. Complete Phase 1 (Setup).
2. Complete Phase 2 (Foundational) — blocks everything.
3. Complete Phase 3 (US1).
4. **STOP & VALIDATE**: Demo open-project → steps list → progress bar → recent projects → restore last step.
5. Ship MVP 0.1.

### Incremental Delivery (P1 bundle)

1. After MVP, add US2 → MVP 0.2 (drill-down + document preview).
2. Add US3 → MVP 0.3 (live terminal with highlights).
3. Add US4 → MVP 0.4 (tasks checklist). This bundle is the full P1 release.

### Incremental Delivery (P2 + P3)

5. Add US5 → 0.5 (env check protection).
6. Add US6 → 0.6 (full 12-agent + generic coverage).
7. Add US7 → 0.7 (OS notifications).
8. Complete Phase 10 (Polish) → **v1.0** release candidate.

### Parallel Team Strategy

With three developers after Phase 2:

- Dev A: US1 → US5
- Dev B: US2 → US6
- Dev C: US3 → US4 → US7

Each stream writes its failing tests first, merges independently behind a feature flag if needed, and joins at Phase 10 for the cross-cutting polish.

---

## Notes

- `[P]` = different files, no dependencies on incomplete tasks in the same phase.
- `[Story]` label maps each task to its user story for traceability.
- Every user story is independently demo-able at its final checkpoint.
- Tests are authored before implementation in every story; commit failing test and implementation in the same PR per Constitution III.
- Commit after each task or logical group; push to `origin/001-speclens-desktop` at each checkpoint.
- Avoid: same-file conflicts, cross-story hard dependencies, vague descriptions, skipping the failing-test step.
