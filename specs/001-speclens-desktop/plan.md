# Implementation Plan: SpecLens — Cross-Platform Visual Management for GitHub Spec-Kit

**Branch**: `001-speclens-desktop` | **Date**: 2026-04-11 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-speclens-desktop/spec.md`

## Summary

SpecLens is a cross-platform desktop application that gives AI developers a visual
management UI for GitHub Spec-Kit. It opens a local project, verifies the Spec-Kit CLI
environment, shows every Spec-Kit step with its status, exposes per-phase Overview /
Documents / Tasks sub-pages, streams PTY output with universal highlight rules, and
supports all 12 official AI Agents plus a generic fallback. Per the 2026-04-11
clarifications, v1 is **strictly observational** (no Run/Retry/Reset actions), stores
cross-project data in the OS app-data directory and per-project data in `.speclens/`,
uses a rolling in-memory terminal buffer with disk spill-over, ships UI in English and
Traditional Chinese, and adopts a multi-window model with one project per window.

Technical approach (per `TECH.md` v1.3.0): Tauri v2 + Rust backend + React + TypeScript
frontend. Rust owns project I/O, PTY bridging, timestamped output buffering, document
hash tracking, agent detection, and task-file parsing. React (Vite + Tailwind + shadcn/ui
+ xterm.js + Zustand) owns the entire UI layer and communicates through Tauri IPC
commands and events. Every user-facing feature is either observation or read-only
retrieval, consistent with FR-044 and the two-tier safety model.

## Technical Context

**Language/Version**:
- Backend: Rust stable ≥ 1.78.0 (2024 edition OK on stable)
- Frontend: TypeScript 5.4+, Node.js ≥ 20, pnpm ≥ 9

**Primary Dependencies**:
- Tauri v2 core (`tauri`, `tauri-plugin-fs`, `tauri-plugin-shell`,
  `tauri-plugin-notification`, `tauri-plugin-store`)
- `tokio` (async runtime), `serde` / `serde_json` / `serde_yaml`, `reqwest`,
  `portable-pty`, `notify` (FS watcher), `sha2` + `hex` (content hashing),
  `semver`, `chrono`, `tracing` / `tracing-subscriber`, `thiserror` / `anyhow`
- React 18, Vite, Tailwind CSS, shadcn/ui, `@xterm/xterm` v5,
  `@tanstack/react-virtual`, Zustand, `marked` + `highlight.js`, `lucide-react`,
  `i18next` + `react-i18next` (for FR-087 / FR-088)

**Storage**:
- Cross-project data (FR-090): OS app-data directory via `tauri-plugin-store`
  (`~/Library/Application Support/SpecLens/` on macOS, `%APPDATA%\SpecLens\` on
  Windows, `~/.config/speclens/` on Linux)
- Per-project data (FR-091): `.speclens/` directory at the project root — JSON
  files for last step/tab, task-file selection, document hashes; `.speclens/logs/`
  for rolling terminal buffer spill-over (FR-047 / FR-048)
- Credentials: OS secret store (Keychain / Credential Manager / Secret Service)
  via `tauri-plugin-store` backing (FR-086)

**Testing**:
- Rust: `cargo test` (unit and `#[tokio::test]` async) targeting
  `tasks_parser`, `agent_detector`, `doc_hash_store`, `output_buffer`,
  `phase_scanner`, `highlight_rules`. Target ≥ 80 % coverage on changed code
  (Constitution II).
- Frontend: Vitest + React Testing Library for components and stores;
  `TaskFileBar` (0/1/many tabs), `TaskFilePicker` (first-entry required),
  `OverviewPanel` (slice expansion), highlight-rule application.
- End-to-end: Playwright + tauri-driver (webdriver shim) for the
  open-project → first-tasks-tab → picker → file-bar smoke flow.
- Benchmarks: `criterion` for hot paths called out in Constitution V (terminal
  buffer slice/search, task-file parsing, hash check).

**Target Platform**:
- macOS 12+ (Apple Silicon and Intel)
- Windows 10 and 11 (x64)
- Ubuntu 22.04+, Debian 12+, Fedora 38+ (x64)

**Project Type**: desktop-app (Tauri — single binary with embedded webview)

**Performance Goals**:
- Cold start ≤ 2 s (SC-001 / FR-080)
- Terminal render latency p95 ≤ 100 ms (SC-003 / FR-081)
- Step list scroll ≥ 60 fps (FR-082)
- Workspace fully rendered ≤ 3 s after folder pick (SC-002)
- Interactive user actions p95 < 200 ms (Constitution V default)

**Constraints**:
- Two-tier safety model (FR-085): PTY `cwd` and writes locked to project dir;
  read-only access may roam via OS dialogs.
- No user data ever leaves the device (FR-086 / SC-010). All computation local.
- Observe-only for v1: zero Run/Retry/Reset controls (FR-044, clarification 1).
- Rolling buffer with disk spill-over and a per-project size cap (default
  512 MiB, user-adjustable — FR-047 / FR-048).
- UI must ship `en` and `zh-TW`; string resources externalized from day one
  (FR-087 / FR-088).

**Scale/Scope**:
- Single user, single device; multi-window within one process (FR-100–103).
- Up to ~10,000 terminal lines in memory per window; disk history bounded by
  per-project cap.
- Up to ~50 Spec-Kit steps per project (typical), 1–20 task files per project.
- Source code scope: ~8–12k LOC Rust + ~10–15k LOC TS/React for v1 MVP.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Constitution: `.specify/memory/constitution.md` v1.0.0 (five core principles).

| Principle | Status | Evidence |
|-----------|--------|----------|
| I. Code Quality | PASS | Rust (rustfmt + clippy `-D warnings`) and TS (ESLint + Prettier + `tsc --noEmit`) are enforced in CI on every PR. Complexity target ≤ 10 per function; PR review required; dead code removed before merge. |
| II. Testing Standards | PASS | Unit + integration + benchmark layers defined above. ≥ 80 % coverage on changed code enforced in CI. Flaky tests quarantined within 1 working day. Full suite runs in CI on every PR. |
| III. TDD (NON-NEGOTIABLE) | PASS | Task generation (`/speckit.tasks`) will emit failing test tasks before implementation tasks per user story. Every FR that introduces behavior will land with a matching failing test committed alongside or before implementation. |
| IV. User Experience Consistency | PASS | Single shared design system (shadcn/ui + Tailwind tokens) used by every surface. All error messages follow the "what happened / why / what to do next" shape. i18n externalizes all user-visible text (FR-087 / FR-088). WCAG 2.1 AA baseline enforced via an a11y component checklist. |
| V. Performance Requirements | PASS | Explicit per-feature budgets declared above and mapped to SC-001 / SC-002 / SC-003 / FR-080 / FR-081 / FR-082. Benchmarks for terminal buffer slice, task-file parsing, and highlight-rule application will be checked into the repo and executed in CI; > 10 % regression blocks merge (Constitution V). |

**Gate result: PASS on all five principles.** No violations — the Complexity Tracking
table below is intentionally empty.

## Project Structure

### Documentation (this feature)

```text
specs/001-speclens-desktop/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/
│   └── ipc.md           # Phase 1 output — Tauri IPC command & event contracts
├── checklists/
│   └── requirements.md  # Spec quality checklist (already produced by /speckit.specify)
├── spec.md              # Feature spec (already produced by /speckit.specify)
└── tasks.md             # Phase 2 output — produced later by /speckit.tasks
```

### Source Code (repository root)

```text
speclens/                              # v1 repository root (this repo)
├── src/                               # React + TypeScript frontend (Vite)
│   ├── main.tsx
│   ├── App.tsx
│   ├── i18n/
│   │   ├── index.ts                   # i18next init (FR-087 / FR-088)
│   │   ├── en.json
│   │   └── zh-TW.json
│   ├── pages/
│   │   ├── Welcome.tsx                # recent-projects / onboarding
│   │   └── Workspace.tsx              # main per-window workspace
│   ├── components/
│   │   ├── layout/                    # AppShell, TopBar, ResizablePanel
│   │   ├── steps/                     # StepsList, StepItem, ProgressBar
│   │   ├── phase/                     # PhaseTabs, PhaseTabPanel, OverviewPanel,
│   │   │                              # DocumentList, DocumentItem, DocumentPreview
│   │   ├── tasks/                     # TasksPanel, TaskFileBar, TaskFilePicker,
│   │   │                              # TasksProgressPanel, TaskFilter, TaskItem, TaskDetail
│   │   ├── terminal/                  # TerminalPanel (xterm), TerminalToolbar, QuickCommands
│   │   ├── agent/                     # AgentBadge, AgentIcon
│   │   └── env/                       # EnvCheckDialog, InstallGuide, UpdateBanner
│   ├── stores/                        # Zustand slices (one per domain)
│   ├── hooks/                         # useSpeckitEvents, usePtySession, useOutputSlice, ...
│   ├── lib/                           # tauri bridge, markdown, highlightRules, utils
│   └── types/                         # shared TS types mirroring Rust serde models
├── src-tauri/                         # Rust backend (Tauri crate)
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   ├── icons/                         # app icons per platform
│   └── src/
│       ├── main.rs
│       ├── lib.rs
│       ├── commands/                  # one module per IPC domain
│       │   ├── project.rs
│       │   ├── speckit.rs
│       │   ├── terminal.rs
│       │   ├── phase.rs
│       │   ├── tasks.rs
│       │   ├── agent.rs
│       │   └── config.rs
│       ├── services/
│       │   ├── speckit_engine.rs
│       │   ├── terminal_bridge.rs     # PTY + rolling buffer + disk spill (FR-047/048)
│       │   ├── version_checker.rs
│       │   ├── phase_scanner.rs       # + DocStatus hash logic
│       │   ├── task_file_scanner.rs
│       │   ├── tasks_parser.rs        # md / json / yaml / txt
│       │   ├── agent_detector.rs      # 12 agents + generic fallback
│       │   ├── doc_hash_store.rs
│       │   ├── highlight_rules.rs
│       │   ├── state_store.rs         # per-project .speclens/ I/O (FR-091/092)
│       │   ├── app_data_store.rs      # OS app-data I/O (FR-090)
│       │   └── fs_watcher.rs
│       ├── models/                    # serde-serializable data types
│       └── error.rs
│   └── benches/                       # criterion benches for hot paths
├── tests/
│   ├── e2e/                           # Playwright + tauri-driver specs
│   └── fixtures/                      # sample project dirs used by Rust & e2e tests
├── public/
├── package.json
├── pnpm-lock.yaml
├── vite.config.ts
├── tsconfig.json
├── .github/workflows/
│   ├── ci.yml                         # lint + type + test + bench + coverage
│   └── build.yml                      # macOS / Windows / Linux production build
├── PRD.md                             # product requirements doc (already in repo)
├── TECH.md                            # technical requirements doc (already in repo)
└── README.md
```

**Structure Decision**: **Option — desktop-app with split frontend/backend in a
Tauri monorepo.** `src/` holds the Vite + React + TypeScript frontend; `src-tauri/`
holds the Rust Tauri crate that compiles to the desktop binary on all three target
platforms. Tests live in `src-tauri/src/**` (Rust unit), `src/**` (Vitest unit), and
`tests/e2e/` (Playwright). This layout matches `TECH.md §2 / §10` and is the standard
Tauri v2 scaffold, so it minimises tooling friction and maximises the number of
existing CI recipes we can reuse.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| *(none — all five principles pass)* | — | — |
