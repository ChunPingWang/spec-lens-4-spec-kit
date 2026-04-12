# spec-lens Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-04-12

## Active Technologies

- 001-speclens-desktop — SpecLens desktop workspace (Tauri v2 monorepo)
  - Backend: Rust (stable) + `tauri` v2, `tauri-plugin-notification` v2,
    `portable-pty`, `notify`, `serde`, `sha2`, `regex`, `tracing`,
    `criterion`, `insta`, `tempfile`
  - Frontend: React 18 + TypeScript 5.5 + Vite 5, Zustand stores,
    `i18next` (en / zh-TW), Tailwind CSS, `@testing-library/react` +
    Vitest (jsdom), Playwright (e2e, Phase 10)

## Project Structure

```text
src-tauri/           # Rust backend (commands, services, state, benches)
  src/commands/      # Tauri #[command] IPC handlers
  src/services/      # Disk-touching + pure logic (state_store, doc_hash,
                     #   highlight_rules, terminal_bridge, notification_dispatcher,
                     #   gitignore_hint, agent_detector, version_checker, ...)
  src/models/        # camelCase serde types shared with the frontend
  tests/             # Integration tests (cmd_*.rs) per IPC command
  benches/           # Criterion benches (Constitution V budgets)

src/                 # React frontend (Vite)
  components/        # Feature UI (env/, phase/, tasks/, terminal/, settings/, ...)
  hooks/             # useSpeckitEvents, ...
  stores/            # Zustand: projectStore, configStore, stepsStore,
                     #   tasksStore, envStore, terminalStore
  lib/               # invoke/subscribe wrappers, highlightRules, notify
  i18n/              # en.json + zh-TW.json (parity enforced by i18n.test.ts)
  pages/             # Welcome, Workspace
  test/              # Vitest setup + @tauri-apps/api mocks

specs/001-speclens-desktop/   # plan, spec, tasks, research, data-model, contracts
```

## Commands

```bash
# Backend gates
cd src-tauri
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo bench --bench bench_main            # criterion, Constitution V budgets

# Frontend gates
pnpm typecheck                            # tsc --noEmit
pnpm lint                                 # eslint --max-warnings 0
pnpm test -- --run                        # vitest run
pnpm test:e2e                             # playwright (headed in CI)
pnpm dev                                  # vite + tauri dev
```

## Code Style

- **Rust**: `rustfmt` default profile, clippy `-D warnings`. Services own
  disk IO; commands are thin wrappers that translate errors to
  `IpcResult<T>`. Prefer `#[serde(rename_all = "camelCase")]` on every
  wire type so the frontend never translates keys. Unit tests live in the
  same file under `#[cfg(test)] mod tests`; integration tests live under
  `src-tauri/tests/` with `cmd_<name>.rs` naming.
- **TypeScript**: strict, no `any`. Zustand stores call `invoke<T>` through
  `src/lib/tauri.ts` (never import `@tauri-apps/api/core` directly outside
  that file and test setup). React components stay presentational; all
  state sits in Zustand. Tests mock at the IPC boundary via
  `src/test/setup.ts`.
- **i18n**: every user-facing string lives in `en.json` + `zh-TW.json`.
  Key parity is CI-enforced by `src/i18n/i18n.test.ts` (T157).
- **Paths**: never touch files outside a registered project root.
  `resolve_inside_root` + `FR-085` are non-negotiable.

## Constitution Highlights

- **III (TDD, NON-NEGOTIABLE)**: failing test first, then implementation.
- **V (Performance budgets)**: `highlight_rules::first_match` ≤ 10 µs/line,
  `doc_hash_store::compute_sha256_hex` ≤ 5 ms for a 1 MiB file,
  `terminal stream` ≤ 100 ms end-to-end (FR-081), Workspace first paint
  ≤ 3 s. Enforced via `cargo bench` + Playwright perf probes.
- **IV (UX consistency)**: keyboard-first, axe-clean, en/zh-TW parity,
  dark + light themes.

## Recent Changes

- 2026-04-12 — Phase 10 polish (partial): added criterion benches for
  `highlight_rules` (≤ 10 µs) and `doc_hash_store` (≤ 5 ms) in
  `src-tauri/benches/bench_main.rs`; added `services/gitignore_hint.rs`
  (FR-091 helper, 9 unit tests); added `src/i18n/i18n.test.ts` key-parity
  lint; refreshed this CLAUDE.md (T152, T153, T157, T164, T167).
- 2026-04-12 — Phase 9 US7 (Native Notifications): `notify_system`
  command, `configStore.setNotificationsEnabled`, `NotificationToggle`
  UI, `useSpeckitEvents` hook wiring `steps_state_changed` →
  `notify_system` with focus gating (T145-T151 / FR-070).
- 2026-04-11 — Phase 8 US6 (AI Agent Compatibility): `agent_detector`
  service, `agent_resolve`/`agent_set_override` commands, Workspace
  agent badge (T131-T144).
- 2026-04-11 — Phase 7 US5 (Spec-Kit Env Check): `version_checker`
  service, `env_check`/`env_dismiss_update` commands, blocking
  `EnvCheckDialog` + persistent `UpdateBanner` (T119-T128 / FR-010).
- 2026-04-10 — Phase 6 US4 (Tasks): `tasks_parser` with four-format
  checklist support, `task_file_scanner`, TaskFilePicker / TaskFileBar /
  TaskListPanel / TaskDetailPanel (T103-T118 / FR-050-060).
- 2026-04-10 — Phase 5 US3 (Terminal): `terminal_bridge` + `output_buffer`
  with memory / disk tiers, highlight routing, in-panel TerminalPanel
  with auto-scroll + connect/disconnect (T082-T102 / FR-080-088).
- 2026-04-09 — Phase 4 US2 (Phase tabs): `phase_scanner` +
  `doc_hash_store`, PhaseTabs + DocumentList + DocumentPreview with
  generated/modified/missing badges (T065-T081 / FR-030-046).
- 2026-04-08 — Phase 3 US1 (Open project + Workspace shell): `project_open`
  command, `StateStore` round-trip, Welcome page recent list, three-pane
  Workspace layout with persisted `lastStepId` (T037-T064 / FR-001-009).
- Phase 1-2 Setup + Foundational: Tauri v2 scaffolding, Zustand stores,
  IPC contracts, i18n bundles, integration harness (T001-T036).

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
