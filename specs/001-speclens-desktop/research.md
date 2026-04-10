# Phase 0 Research: SpecLens — Cross-Platform Visual Management for GitHub Spec-Kit

**Branch**: `001-speclens-desktop` | **Date**: 2026-04-11 | **Plan**: [plan.md](./plan.md)

## Purpose

Consolidate the technology and design decisions needed before Phase 1 (contracts,
data model, quickstart). Every item below is derived from `PRD.md v1.3.0`,
`TECH.md v1.3.0`, the feature spec (`spec.md`), and the 2026-04-11 clarification
session. The Technical Context in `plan.md` contains **no NEEDS CLARIFICATION
markers**; this document captures the *why* behind each resolved choice and the
alternatives rejected.

## Summary of Clarification-Driven Decisions

These five decisions flow directly from the spec's `## Clarifications` session
and set hard constraints for the rest of the plan.

### D1. v1 is strictly observational (no Run/Retry/Reset)

- **Decision**: SpecLens v1 does not expose any control that starts, retries,
  cancels, or resets a Spec-Kit step. It only observes what the user or an AI
  Agent does in the attached terminal.
- **Rationale**: A purely observational v1 eliminates the need to model
  execution state-machines, recovery flows, and partial-failure UX, and lets
  us ship safely without risking the user's project state. FR-044 is tightened
  to forbid such controls; SC-008 (100 % of observed steps correctly reported)
  only makes sense under an observe-only contract.
- **Alternatives considered**:
  - *Full execution orchestration* (Run / Retry / Reset buttons): adds
    implementation and safety surface area without a v1 user need.
  - *Read-only + "Copy Command"*: acceptable alternative for later versions;
    kept as a future extension point behind FR-044.

### D2. Hybrid state persistence (OS app-data + `.speclens/`)

- **Decision**: Cross-project data (recent projects, preferences, language,
  global window state) lives in the OS application-data directory via a Tauri
  store; per-project data (last step, last tab, picked task file, document
  hashes, rolling-buffer spill files) lives in `.speclens/` at the project
  root. Credentials (none in v1, but the mechanism exists) use the OS secret
  store.
- **Rationale**: Cross-project settings must follow the user across projects;
  per-project state must travel with the project (Git clones, machine moves).
  Splitting along that boundary is the minimum surface that satisfies both.
  `.speclens/` stays out of Spec-Kit's own namespace.
- **Alternatives considered**:
  - *All in OS app-data*: loses per-project portability and pollutes global
    state with project-specific data.
  - *All in `.speclens/` under each project*: forces recent-projects and
    language to be reconstructed per-workspace and fails when no project is
    open.

### D3. Rolling in-memory buffer with disk spill-over

- **Decision**: The terminal bridge keeps the most recent 10,000 lines per
  window in memory and spills older output to `.speclens/logs/<session>.ndjson`.
  The per-project on-disk cap defaults to 512 MiB and is user-adjustable
  (FR-047 / FR-048). Each line carries a millisecond timestamp to correlate
  with steps.
- **Rationale**: Agent runs can be long and verbose (10 k+ lines). Keeping
  everything in RAM risks memory pressure; dropping old output breaks
  retrospective diagnosis. A rolling buffer with a bounded on-disk tail
  preserves both responsiveness and history within a declared budget.
- **Alternatives considered**:
  - *Pure in-memory ring buffer*: can lose critical context during long runs.
  - *Unbounded disk log*: unbounded growth violates Constitution V
    ("no unbounded growth; all caches MUST have eviction policies").
  - *External log file managed by the OS*: requires a shell shim and loses
    ANSI fidelity; rejected.

### D4. UI languages: English + Traditional Chinese (en, zh-TW)

- **Decision**: v1 ships with `en` and `zh-TW` bundles, auto-detects the OS
  language on first launch, and allows hot language switching from settings.
  All user-facing strings are externalized (FR-087 / FR-088).
- **Rationale**: The primary audience is spec-kit adopters in both English
  and Traditional Chinese developer communities. Externalizing from day one
  avoids a painful retrofit and is required by Constitution IV.
- **Alternatives considered**:
  - *English only + community translations later*: cheap up-front, expensive
    retrofit, blocks zh-TW adopters.
  - *Full i18n matrix (zh-CN, ja, ko, …)*: unbounded QA burden for v1; deferred.

### D5. Multi-window process model (one project per window)

- **Decision**: A single SpecLens process can host multiple windows, each
  pinned to exactly one project. Cross-project settings are shared through
  the OS app-data store; per-project state stays isolated per window
  (FR-100 – FR-103).
- **Rationale**: Developers often compare specs across projects; a multi-
  window model lets them do that without losing per-window PTY sessions.
  A shared process keeps memory and update flows simple.
- **Alternatives considered**:
  - *Single-window, one project at a time*: forces user to quit/reopen to
    switch projects; rejected as a regression from typical IDE UX.
  - *Multiple processes, one per window*: would duplicate the rolling buffer
    caches and complicate the shared store; rejected.

## Platform & Runtime Decisions

### D6. Desktop shell: Tauri v2

- **Decision**: Use Tauri v2 (Rust host + embedded system WebView) for the
  single cross-platform binary.
- **Rationale**:
  - Ships a ~10–20 MB installer vs. ~80–150 MB for Electron equivalents,
    directly supporting SC-001 (cold start ≤ 2 s).
  - Native Rust backend lets us own PTY, FS, and hashing in a typed language
    without cross-boundary marshalling penalties.
  - First-class support for all three target OSes and for keychain-backed
    stores (FR-086).
  - Supports multi-window out of the box (FR-100).
- **Alternatives considered**:
  - *Electron + Node*: larger binary, slower cold start, less safe FS/PTY
    story, JS event loop in the hot path for PTY output.
  - *Native per-platform apps (Swift / WinUI / GTK)*: triples maintenance
    cost and design-system work for zero user-visible benefit in v1.
  - *Flutter desktop*: weak PTY ecosystem and immature xterm equivalent.

### D7. Rust async runtime: tokio

- **Decision**: Use `tokio` as the single async runtime for the backend.
- **Rationale**: Tauri v2 is tokio-native; `portable-pty`, `reqwest`,
  `notify`, and `tracing` all integrate cleanly. A single runtime avoids
  the classic dual-executor deadlocks.
- **Alternatives considered**:
  - *async-std*: smaller ecosystem, no Tauri alignment.
  - *smol*: minimal ecosystem for the features we need (FS watcher, HTTP).

### D8. PTY bridge: portable-pty

- **Decision**: Use `portable-pty` (from the WezTerm project) for cross-
  platform PTY spawning and I/O.
- **Rationale**: The only mature crate that gives ConPTY (Windows), UnixPTY
  (macOS/Linux), and a consistent async API. Battle-tested by WezTerm.
- **Alternatives considered**:
  - *Shell out to `winpty` / `script`*: platform-specific, brittle, and
    loses ANSI fidelity on Windows.
  - *Writing our own ConPTY wrapper*: huge surface for a solved problem.

### D9. Filesystem watcher: notify

- **Decision**: Use `notify` with its recommended `debouncer-full` wrapper.
- **Rationale**: The only crate that abstracts FSEvents / ReadDirectoryChanges
  / inotify behind one API with cross-platform debouncing. Required for
  document-hash re-checks and task-file list refreshes.
- **Alternatives considered**:
  - *Polling with a timer*: kills battery and misses rapid edits.

### D10. Content hashing: sha2 + hex

- **Decision**: SHA-256 via `sha2` crate, rendered as lowercase hex, is the
  canonical document hash recorded in `.speclens/doc-hashes.json`.
- **Rationale**: Collision-resistant, fast enough for the ~MB-sized markdown
  artifacts Spec-Kit produces, and aligns with PRD's original "content hash"
  wording.
- **Alternatives considered**:
  - *BLAKE3*: slightly faster but adds a dependency without a user-visible
    benefit for our file sizes.
  - *xxhash*: fast but not collision-resistant enough to be trusted as a
    "generated vs. modified" witness.

## Frontend Decisions

### D11. UI framework: React 18 + Vite + TypeScript

- **Decision**: React 18 with the Vite dev server and TypeScript 5.4+.
- **Rationale**: Largest component ecosystem for the building blocks we
  need (xterm, virtualization, markdown), first-class TypeScript story,
  Vite HMR is a measurable productivity win.
- **Alternatives considered**:
  - *Svelte 5*: smaller footprint but weaker xterm/virtualization bindings.
  - *SolidJS*: fine-grained reactivity is attractive but would force us to
    port shadcn/ui patterns.

### D12. Design system: shadcn/ui + Tailwind CSS

- **Decision**: Copy-in components from shadcn/ui on top of Tailwind CSS.
- **Rationale**: Gives us a curated, accessible component set with tokens
  we own (no framework lock-in), which Constitution IV requires ("shared
  design system"). Accessibility primitives via Radix UI are WCAG 2.1 AA
  friendly by default.
- **Alternatives considered**:
  - *Material UI / MUI*: heavier, more opinionated theming, harder to
    retheme for the dense IDE-style layout SpecLens needs.
  - *Chakra UI*: fine but duplicates Radix/Tailwind layers we already want.

### D13. Terminal renderer: @xterm/xterm v5 + @xterm/addon-*

- **Decision**: Use the official `@xterm/xterm` v5 package plus
  `addon-fit`, `addon-search`, `addon-web-links`, `addon-unicode11`.
- **Rationale**: The de-facto web terminal. Supports full ANSI color,
  Unicode 11 width rules, and a virtualized renderer that holds 60 fps on
  the workloads we'll throw at it (FR-081 / FR-082).
- **Alternatives considered**:
  - *Hand-rolled canvas renderer*: huge effort for no win.
  - *hterm*: less active, weaker addon story.

### D14. State management: Zustand

- **Decision**: Zustand with one slice per domain (project, steps, phases,
  tasks, terminal, agent, config).
- **Rationale**: Minimal boilerplate, React-friendly selectors, tiny
  runtime, composes with immer when we need immutable updates. Keeps the
  store per-window (matching FR-100 – FR-103) without a Redux-style
  singleton.
- **Alternatives considered**:
  - *Redux Toolkit*: more boilerplate than we need and the action-log
    overhead doesn't help here.
  - *Jotai / Recoil*: atom-centric models don't map well to the step /
    tasks domains, which are collection-heavy.

### D15. List virtualization: @tanstack/react-virtual

- **Decision**: Use `@tanstack/react-virtual` for the steps list, document
  list, task list, and terminal line viewport.
- **Rationale**: Headless, framework-agnostic, TypeScript-first, and the
  only library that composes cleanly with keyboard-driven nav across our
  four virtualized surfaces.
- **Alternatives considered**:
  - *react-window / react-virtualized*: legacy APIs, weaker TS types.

### D16. Markdown + syntax highlighting: marked + highlight.js

- **Decision**: Use `marked` for CommonMark parsing and `highlight.js` for
  fenced code blocks.
- **Rationale**: Small, sync, predictable; enough for the documents Spec-
  Kit produces. highlight.js covers the languages AI agents actually emit.
- **Alternatives considered**:
  - *react-markdown + remark*: flexible but slower and larger.
  - *Shiki*: beautiful output but ships a 5+ MB WASM blob that would blow
    through SC-001's cold-start budget.

### D17. Internationalization: i18next + react-i18next

- **Decision**: `i18next` with the `react-i18next` bindings; one JSON file
  per language at `src/i18n/<lang>.json`; language detection via OS locale
  with override from settings (FR-087 / FR-088).
- **Rationale**: The only widely adopted React i18n stack with lazy loading,
  plural rules, and interpolation. Works well with Zustand for live switching.
- **Alternatives considered**:
  - *FormatJS (react-intl)*: ICU message format is overkill for v1 and adds
    build complexity.
  - *Homegrown JSON + lookup function*: cheap now, expensive once we add a
    third language.

## Testing & Quality Decisions

### D18. Rust test harness: `cargo test` + tokio::test

- **Decision**: Standard `cargo test` with `#[tokio::test]` for async,
  `pretty_assertions` for readable diffs, and `insta` for snapshot testing
  the highlight-rule and task-parser outputs.
- **Rationale**: Zero extra infra; matches community convention; snapshot
  tests are ideal for parser output.
- **Alternatives considered**:
  - *Custom harness*: unnecessary.

### D19. Frontend unit tests: Vitest + React Testing Library

- **Decision**: Vitest (Vite-native) with React Testing Library and
  `@testing-library/user-event` for component tests.
- **Rationale**: Shares Vite's transform pipeline, so tests run under the
  same module resolution as the app, eliminating the classic Jest/ESM
  mismatch. RTL enforces accessibility-oriented queries, aligning with
  Constitution IV.
- **Alternatives considered**:
  - *Jest*: requires a separate transform config and has weaker ESM story.

### D20. End-to-end: Playwright + tauri-driver

- **Decision**: Playwright driving the Tauri binary through `tauri-driver`
  (WebDriver shim). One primary smoke flow for v1: open project → env
  check → first tasks tab → picker → file-bar updates.
- **Rationale**: Playwright is the most capable, actively maintained web
  E2E framework; `tauri-driver` is the official recipe for Tauri WebView
  automation.
- **Alternatives considered**:
  - *Cypress*: weaker cross-browser / cross-platform story with Tauri.
  - *WebDriverIO*: workable alternative, but Playwright has better
    retry/selector ergonomics out of the box.

### D21. Benchmarks: criterion

- **Decision**: `criterion` benches for (a) terminal-buffer slice/search,
  (b) task-file parsing (md / json / yaml / txt), (c) highlight-rule
  application, (d) document-hash recompute.
- **Rationale**: Constitution V requires performance-critical paths to have
  benchmarks in CI with > 10 % regression blocking merge. Criterion's
  statistical analysis makes regressions detectable with small sample sizes.
- **Alternatives considered**:
  - *divan*: newer, faster, but less tooling around CI comparison reports.

## CI / Packaging Decisions

### D22. CI matrix: GitHub Actions, three OSes

- **Decision**: One workflow (`ci.yml`) for lint + type + test + bench on
  Ubuntu / macOS / Windows; a second workflow (`build.yml`) for signed
  production builds triggered by tags.
- **Rationale**: Matches the three target platforms; GitHub Actions has
  native runners for all three; Tauri's official action recipes cover
  signing and universal macOS binaries.
- **Alternatives considered**:
  - *CircleCI / BuildKite*: extra billing and account setup for no win.

### D23. Packaging & updater

- **Decision**: Tauri bundler for `.dmg` / `.msi` / `.AppImage` + `.deb`
  with code signing (Apple notarization on macOS, Authenticode on Windows).
  The auto-update channel is scaffolded behind a feature flag but **off**
  for v1 (FR-011 only surfaces an in-app banner pointing users at release
  notes).
- **Rationale**: Signed installers are non-negotiable for user trust; the
  updater is explicitly out of scope for v1 per PRD §10.
- **Alternatives considered**:
  - *Ship the updater on in v1*: adds cert/keys and a rollback story we
    don't need yet.

## Data & Safety Decisions

### D24. Two-tier safety model

- **Decision**: All PTY spawns and all writes (including `.speclens/` I/O)
  are locked to the currently attached project directory. Read-only
  operations (preview, "open file in OS") may roam via OS dialogs under
  FR-085.
- **Rationale**: Limits blast radius of any future execution feature and
  keeps the app honest about where state changes happen.

### D25. Doc hash store schema

- **Decision**: `.speclens/doc-hashes.json` is a JSON object keyed by
  relative POSIX path, value `{ "sha256": "<hex>", "generatedAt":
  "<ISO-8601>" }`. Recomputed on FS-watch events and on window focus.
- **Rationale**: A plain JSON file is trivial to read in tests and from the
  frontend via Tauri IPC; the timestamp helps users trust the "generated"
  badge.

### D26. Task-file parsing strategy

- **Decision**: A single `tasks_parser` module handles four formats
  (`.md`, `.json`, `.yaml`/`.yml`, `.txt`) and produces a common
  `TaskEntry` shape: `{ id, title, status, sectionPath, filePath, line,
  raw }`. Markdown detection keys off the checkbox pattern
  `^- \[[ xX~-]\] `; JSON/YAML expect `{ tasks: [...] }`; text files
  expect one task per line.
- **Rationale**: Keeps the parser testable with snapshot fixtures and lets
  the UI treat all task sources uniformly. Matches FR-050 – FR-057.

### D27. AI Agent detection (12 + generic)

- **Decision**: The `agent_detector` scans both the project root and the
  OS `PATH` for the 12 official Spec-Kit agents (Claude Code, Copilot,
  Gemini CLI, Cursor, Windsurf, Amazon Q, Codex CLI, Qwen Code, opencode,
  Kilo Code, Auggie CLI, Roo Code). If none are found, the UI falls back
  to a generic agent profile and flags the project as "unrecognized
  agent" without blocking the user (FR-060 / FR-061).
- **Rationale**: Aligns with Spec-Kit's published agent list and still
  serves users running with a custom or private agent.

## Performance Budgets (from plan + constitution)

| Path | Budget | Source |
|------|--------|--------|
| Cold start (open app) | ≤ 2 s | SC-001 / FR-080 |
| Workspace fully rendered after folder pick | ≤ 3 s | SC-002 |
| Terminal render latency p95 | ≤ 100 ms | SC-003 / FR-081 |
| Step list scroll | ≥ 60 fps | FR-082 |
| Interactive actions p95 (click → visual feedback) | < 200 ms | Constitution V default |
| Per-project rolling-buffer disk cap | 512 MiB default (user-adjustable) | FR-047 / FR-048 |

All budgets are enforced via criterion benches (Rust hot paths) and
Playwright performance probes (cold start, workspace render).

## Open Questions

- **None**. All spec-level ambiguities were resolved in the 2026-04-11
  clarification session; every Technical Context field in `plan.md` has a
  concrete value; no NEEDS CLARIFICATION markers remain.

## References

- `specs/001-speclens-desktop/spec.md` — feature spec (post-clarify)
- `specs/001-speclens-desktop/plan.md` — implementation plan (this phase)
- `.specify/memory/constitution.md` v1.0.0 — governing principles
- `PRD.md` v1.3.0 — product requirements
- `TECH.md` v1.3.0 — technical requirements
