# Feature Specification: SpecLens — Cross-Platform Visual Management for GitHub Spec-Kit

**Feature Branch**: `001-speclens-desktop`
**Created**: 2026-04-11
**Status**: Draft
**Input**: User description: "PRD.md v1.3.0 — SpecLens: a cross-platform desktop application that gives AI developers a visual management UI for GitHub Spec-Kit, covering project management, environment checks, step/phase navigation, task checklists, terminal bridging, and multi-AI-agent support."

## Clarifications

### Session 2026-04-11

- Q: Execution control scope — should SpecLens expose step action controls (Run / Retry / Reset) or stay strictly observational? → A: Observe-only. No UI buttons execute or mutate Spec-Kit steps. The optional PTY command-forwarding capability in FR-044 remains a power-user toggle, not a first-class workflow control.
- Q: Where are per-project and cross-project state (recent list, last step/tab, task file selections, document hash history) stored? → A: Hybrid. Cross-project user data (recent projects list, global preferences, window layout) lives in the OS application-data directory (`~/Library/Application Support/SpecLens/` on macOS, `%APPDATA%\SpecLens\` on Windows, `~/.config/speclens/` on Linux). Per-project state (last step and phase tab, task file selection, document hash history) lives in a `.speclens/` directory at the project root; SpecLens MUST recommend adding `.speclens/` to the project's `.gitignore`.
- Q: What is the terminal buffer retention model? → A: Rolling in-memory window with disk spill-over. The system keeps the most recent ~10,000 lines (user-adjustable) in memory; older lines are flushed to `.speclens/logs/<timestamp>.log` inside the project. Search, Overview slices, and export MUST read transparently across both tiers so the full run history remains recoverable while memory usage stays bounded and the p95 latency target (SC-003 / FR-081) is preserved.
- Q: Which UI languages does v1 ship? → A: English and Traditional Chinese (`en`, `zh-TW`). v1 builds an i18n foundation with string resource files, defaults the UI language to the OS language where a translation exists, falls back to English otherwise, and exposes an explicit language override in settings. Product name, AI Agent brand names, and Spec-Kit CLI output remain untranslated.
- Q: Can users work on multiple projects simultaneously, and what is the window model? → A: Multi-window, one project per window. A single SpecLens process MAY host multiple top-level windows, each bound to exactly one project with its own PTY session, terminal buffer, phase tab state, and Documents/Tasks view. Cross-project user state (recent list, preferences, UI language, highlight rules) is shared by the process; per-project state (FR-091) is strictly isolated per window.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Open a project and see the current Spec-Kit progress at a glance (Priority: P1)

An AI developer launches SpecLens, selects a local project directory (or drags it onto the
window), and immediately sees: the project name, which AI Agent is in use, the full list of
Spec-Kit steps, and a visual progress summary of what is completed, running, failed, skipped,
or blocked. Prior open projects are surfaced as a "recent projects" list so the developer can
reopen a workspace with one click and return to the step and tab they were last viewing.

**Why this priority**: This is the core reason the product exists. Without the ability to
open a project and perceive overall progress visually, no other feature delivers value. It is
the minimum viable slice.

**Independent Test**: Open SpecLens, pick a folder that contains a valid Spec-Kit
configuration, and confirm the project name, agent, step list with correct status indicators,
and overall progress bar all appear. Close and reopen the app and confirm the project shows
up in "recent projects" and restores the last viewed step and tab.

**Acceptance Scenarios**:

1. **Given** a local folder containing a valid Spec-Kit configuration, **When** the user
   opens it via the file picker, **Then** the app displays the project name, path, last
   modified time, Spec-Kit version, detected AI Agent, and the complete list of steps with
   their current status.
2. **Given** a local folder containing a valid Spec-Kit configuration, **When** the user
   drags the folder onto the app window, **Then** the project opens with the same result as
   the file picker path.
3. **Given** the user has previously opened one or more projects, **When** the user launches
   the app, **Then** the welcome screen shows up to 10 recent projects ordered by most
   recently opened, and selecting one reopens it without a file picker.
4. **Given** a project is open, **When** the user closes and reopens the same project, **Then**
   the previously selected step and phase tab are restored.
5. **Given** an opened project, **When** the overall progress changes, **Then** the top
   progress bar updates to show `X / Y steps completed` with a percentage.

---

### User Story 2 - Drill into a phase and see its Overview, Documents, and (for tasks) Tasks (Priority: P1)

Within an opened project, the developer clicks any phase tab (e.g., `spec`, `tasks`,
`implement`, `review`, `test`). Each phase exposes three consistent sub-pages: an Overview
that summarizes the step and shows the related terminal output slice; a Documents sub-page
that lists every artifact produced by that phase with status badges and an inline preview;
and, only for the `tasks` phase, a Tasks sub-page showing an interactive checklist. Left
sidebar selection and tab selection stay in sync.

**Why this priority**: Viewing progress is not enough—developers need to inspect what each
phase produced and why. This is what turns SpecLens from a dashboard into a working tool, and
it is required for the P1 experience to be useful in real work.

**Independent Test**: With a project open that has run at least one phase, click each phase
tab and confirm Overview shows step details and a related terminal slice; Documents shows
generated files with badges and inline preview; the Tasks sub-page appears only on the
`tasks` phase; and selecting a step in the left sidebar changes the active phase tab and
vice versa.

**Acceptance Scenarios**:

1. **Given** a project with at least one completed phase, **When** the user clicks a phase
   tab, **Then** the Overview sub-page shows the step name, current status, run duration,
   description, declared inputs, expected outputs, start time, and a collapsible terminal
   output slice covering the `running → completed/failed` time range of that step.
2. **Given** a phase with produced files, **When** the user opens the Documents sub-page,
   **Then** each file is shown with name, icon, last modified time, size, and one of the
   status badges `generated`, `modified`, or `missing`; when no historical record exists the
   badge area shows an "unable to confirm original state" hint.
3. **Given** a document row, **When** the user clicks it, **Then** an inline preview opens
   with Markdown rendering, syntax highlighting, or JSON/YAML pretty-printing as appropriate
   to the file type.
4. **Given** the `tasks` phase is selected, **When** the user opens sub-pages, **Then** a
   third sub-page named Tasks is present in addition to Overview and Documents; for every
   other phase no such third sub-page appears.
5. **Given** the user clicks a step in the left sidebar, **When** the click is handled,
   **Then** the right-side phase tab and its sub-page selection update to match the step.
6. **Given** a phase that has not yet executed, **When** the user views the tab row, **Then**
   the tab is greyed out with an indicator dot, and a settings option controls whether the
   tab is locked or freely clickable.

---

### User Story 3 - Watch AI Agent terminal output live, correlated with steps (Priority: P1)

While an AI Agent executes Spec-Kit steps, the developer sees the PTY output streaming in
real time in a bottom panel with ANSI color rendering, universal highlight rules (success,
error, running, warning, JSON, file paths) that work across every agent, and per-step
correlation so they can jump from a step to the exact output slice that corresponds to that
step's run window.

**Why this priority**: The terminal bridge is the second pillar of the product's value
proposition. Without live, correlated output, the dashboard loses its ability to help the
developer diagnose what the AI is doing.

**Independent Test**: Run any supported agent that produces output, confirm the bottom
terminal panel streams output line-by-line with ANSI colors, that highlight rules recolor
matching lines, that each line carries a timestamp, and that opening a completed step's
Overview shows a terminal slice bounded by that step's start and end times.

**Acceptance Scenarios**:

1. **Given** an AI Agent producing PTY output, **When** output arrives, **Then** the bottom
   terminal panel renders it line-by-line with ANSI color support and each line carries a
   millisecond timestamp.
2. **Given** the default highlight rules, **When** a line begins with any configured success,
   error, running, or warning marker, **Then** the line is rendered with the corresponding
   visual style; JSON lines are syntax-highlighted; file paths are underlined and clickable
   to open a preview.
3. **Given** the user provides a filter keyword or a regex search, **When** applied, **Then**
   the terminal view updates to show only matching (or excluded) lines without clearing the
   underlying history buffer.
4. **Given** a completed step, **When** the user opens its Overview sub-page, **Then** the
   related terminal slice shows exactly the output produced between that step's `running`
   start timestamp and its terminal state timestamp.
5. **Given** an ongoing output stream, **When** the user locks auto-scroll, **Then** the
   viewport stays fixed until the user unlocks it; unlocking resumes auto-scroll.
6. **Given** the user requests an export, **When** the user chooses a format, **Then** the
   current visible output (or full buffer) is exported as `.log` or `.txt` with an ASCII
   header describing the project, agent, and time range.

---

### User Story 4 - Manage Tasks through a user-controlled checklist (Priority: P1)

When entering the `tasks` phase for the first time, the user is always shown a Task File
Picker—SpecLens never auto-loads a task file. The user selects one or more files (scan
results plus manual path/file selection), and from then on each selected file appears as a
tab in a persistent Task File Bar. Every task is rendered as an interactive checklist with
search, filter, and per-task expansion that shows description, related documents, and an
approximate terminal slice near completion.

**Why this priority**: Task tracking is the primary day-to-day interaction surface for the
developer and embodies the "do not impose conventions" design principle from the PRD. It is
required to call the product complete.

**Independent Test**: Open a project, switch to the `tasks` tab, confirm the Task File
Picker appears with scan results that are not pre-checked; pick one or more files; confirm
the Task File Bar shows one tab per selected file with a completion badge; add and remove
files; confirm per-tab filter/search state is independent; expand a task and see its detail,
related docs, and terminal slice.

**Acceptance Scenarios**:

1. **Given** the user opens the `tasks` tab for the first time in a project, **When** the
   view loads, **Then** a Task File Picker is shown containing auto-scan results and manual
   path/file pickers; no item is pre-checked; the user must confirm a selection before any
   task file is loaded.
2. **Given** one or more selected task files, **When** the selection is confirmed, **Then**
   each file appears as a tab in a persistent Task File Bar that is always visible, even
   when only a single file is selected; a "⚙ change source" control is always available in
   the top-right corner.
3. **Given** the task file selection, **When** the user reopens the project later, **Then**
   the previously chosen task files load automatically without showing the picker again,
   until the user explicitly changes the source.
4. **Given** multiple task files are loaded, **When** the user adds another file via the
   "+ add file" control, **Then** a new tab is appended and its filter/search state is
   independent from other tabs.
5. **Given** loaded tasks, **When** the user views the progress summary, **Then** the app
   shows a combined progress bar across all files AND a per-current-file progress bar, with
   color coding: 0–49% amber, 50–79% blue, 80–100% green.
6. **Given** loaded tasks, **When** the user clicks the filter chips, **Then** the list
   filters to All / Completed / Incomplete for the current tab only; incomplete tasks sort
   above completed ones.
7. **Given** a task row, **When** the user expands it, **Then** the detail view shows the
   full description, links to related documents (clickable to jump), a terminal output slice
   approximating the 30 seconds before completion labeled "≈ 30 s before completion", and an
   error summary if the task failed.

---

### User Story 5 - Ensure Spec-Kit CLI is installed and up to date before use (Priority: P2)

Each time a project is opened, SpecLens runs an environment check: it detects whether the
Spec-Kit CLI is installed, offers an in-app install flow if missing, checks the currently
installed version against the latest release, and presents an in-app update option with a
progress indicator. Only when the environment is ready does the main workspace load.

**Why this priority**: Environment problems silently break the rest of the experience, so
this check protects the core workflows (P1 stories). But users who already have a working
Spec-Kit install can still complete P1 stories without ever seeing it, so it is P2.

**Independent Test**: Start the app on a machine without Spec-Kit installed and verify the
install guidance and in-app install flow; then start it on a machine with an outdated
version and verify the update banner with changelog preview and background update.

**Acceptance Scenarios**:

1. **Given** Spec-Kit CLI is not present on the machine, **When** the user opens a project,
   **Then** the environment check blocks the main workspace, shows install guidance with
   OS-appropriate commands, and offers an in-app install action that shows live progress.
2. **Given** Spec-Kit CLI is installed but outdated, **When** the environment check runs,
   **Then** an update banner is shown with a changelog preview sourced from upstream release
   notes and "Update now" / "Skip" actions; updates run in the background with a progress
   indicator.
3. **Given** Spec-Kit CLI is current, **When** the environment check completes, **Then** the
   user proceeds directly to agent detection and the main workspace without interruption.
4. **Given** the update action fails, **When** the failure occurs, **Then** the user sees an
   actionable error message and the previous working version remains installed and usable.

---

### User Story 6 - Work with any supported (or unsupported) AI Agent (Priority: P2)

The app identifies which of the 12 officially supported AI Agents is in use by inspecting
the project's explicit Spec-Kit configuration first, then falling back to filesystem
fingerprints, and finally to a "generic mode" when nothing matches. Regardless of which
agent the project uses, the terminal bridge, highlight rules, and phase/task views all work.

**Why this priority**: Agent compatibility broadens the product's addressable audience and
is essential for any team that isn't on Claude Code, but P1 users on a single agent can
still get full value without it. Hence P2.

**Independent Test**: Open projects wired to different agents in turn and confirm each shows
the correct agent name and icon; open a project with no recognizable agent fingerprint and
confirm "generic mode" is shown with full functionality available.

**Acceptance Scenarios**:

1. **Given** a project whose Spec-Kit configuration records an explicit agent, **When** the
   project opens, **Then** that agent's icon and name are shown at the top of the workspace
   and take highest priority over any filesystem fingerprint.
2. **Given** a project with no explicit agent record but a recognizable fingerprint (e.g., a
   known per-agent directory or instruction file), **When** the project opens, **Then** the
   detected agent is shown with an "auto-detected" label.
3. **Given** a project with no recognizable agent at all, **When** the project opens,
   **Then** a "generic mode" icon is shown and every feature (terminal bridge, phases,
   tasks, documents, notifications) remains available.
4. **Given** any supported agent is running, **When** PTY output is received, **Then** the
   universal highlight rules apply without any agent-specific semantic parsing.

---

### User Story 7 - Receive native system notifications for step outcomes (Priority: P3)

When a Spec step completes or fails, the user receives an OS-native notification. The user
can disable these notifications in settings. Environment check results are surfaced as
in-app toasts.

**Why this priority**: Notifications are a convenience, not a prerequisite. They improve the
experience but the product is functional without them.

**Independent Test**: Enable notifications; run a step to completion and then one to
failure; confirm two distinct OS notifications appear. Disable notifications in settings and
re-run; confirm none appear while in-app state still updates.

**Acceptance Scenarios**:

1. **Given** notifications are enabled, **When** a step completes successfully, **Then** the
   OS shows a native success notification identifying the project and step.
2. **Given** notifications are enabled, **When** a step fails, **Then** the OS shows a
   native failure notification with the step name and a short error summary.
3. **Given** the user disables notifications in settings, **When** any step completes or
   fails, **Then** no OS notification is shown while the in-app state still updates.
4. **Given** an environment check produces a result, **When** the result is ready, **Then**
   the user sees an in-app toast describing the outcome.

---

### Edge Cases

- Opening a folder that is not a Spec-Kit project: the app must refuse to open the workspace
  and explain why, without crashing or showing a broken workspace.
- Opening a project whose configured task file paths have since been deleted or moved: each
  missing tab is marked as unavailable with a clear message and a "change source" shortcut.
- The Task File Picker scan returns zero candidates: the manual path/file pickers remain
  available and the user can still proceed without auto-detected results.
- PTY process crashes or closes unexpectedly: the terminal panel remains usable as a buffer
  viewer, and the current step is marked failed with a clear message.
- Very large terminal output buffers: searching, filtering, and exporting must remain
  responsive and must not freeze the UI.
- A file's content hash is not yet known because of a cold start without history: the
  Documents badge shows the "unable to confirm original state" hint instead of a false
  `modified` badge.
- Multiple Spec-Kit projects opened in rapid succession: recent-projects list must not lose
  entries, and per-project workspace memory (last step and tab) must not leak between
  projects.
- Network unavailable during environment check: the app falls back to using the installed
  version and informs the user that the update check could not run.
- Two task files under the same name but different directories: the Task File Bar must
  distinguish them (e.g., by showing the parent directory) so the user can tell them apart.
- User attempts to write outside the project directory from the app (e.g., export path
  selection): the app must enforce the safety model and prevent the write.

## Requirements *(mandatory)*

### Functional Requirements

#### Project Management

- **FR-001**: The system MUST allow users to open a local project directory via a file
  picker or by drag-and-drop of a folder onto the application.
- **FR-002**: The system MUST read the project's Spec-Kit configuration file and refuse to
  open the workspace, with an explanatory message, if no valid configuration is present.
- **FR-003**: The system MUST maintain a recent-projects list containing up to 10 entries,
  ordered most-recent-first, and MUST allow one-click reopening of any listed project.
- **FR-004**: The system MUST treat each project as an independent workspace and MUST
  remember, per project, the last selected step and the last selected phase tab across
  sessions.
- **FR-005**: The system MUST display the project name, path, last modified time, Spec-Kit
  version, and the detected AI Agent icon and name in the workspace header.

#### Environment Management

- **FR-010**: On each project open, the system MUST run an environment check that detects
  whether Spec-Kit CLI is installed and blocks the main workspace until the environment is
  ready.
- **FR-011**: When Spec-Kit CLI is not installed, the system MUST show OS-appropriate
  installation guidance and MUST offer an in-app install action that reports progress; on
  success it MUST re-run the environment check automatically.
- **FR-012**: When Spec-Kit CLI is installed, the system MUST compare the installed version
  against the latest available release and, if an update is available, MUST show an update
  banner with a changelog preview and "Update now" / "Skip" actions; updates MUST run in
  the background with a progress indicator.
- **FR-013**: If the network is unavailable during the update check, the system MUST still
  allow the user to proceed with the currently installed version and MUST inform the user
  that the update check could not complete.

#### Steps Overview

- **FR-020**: The system MUST parse the project's Spec-Kit configuration and list all
  defined steps with at least: step number, name/ID, short description, step type, any
  prerequisite steps, and (if defined in the configuration) an estimated duration.
- **FR-021**: The system MUST visually render each step's status using the six defined
  states: `pending`, `running`, `completed`, `failed`, `skipped`, `blocked`, each with a
  distinct visual treatment documented in the product's design system.
- **FR-022**: The system MUST display a top-of-workspace overall progress bar showing
  completed steps over total steps, including a percentage and an `X / Y steps completed`
  label.

#### Phase Tabs and Sub-pages

- **FR-030**: Each phase MUST be represented by its own phase tab, and each phase tab MUST
  expose a consistent three-sub-page structure: Overview, Documents, and (only for the
  `tasks` phase) Tasks.
- **FR-031**: Selecting a step in the left-side step list MUST update the active phase tab
  and its sub-page selection, and vice versa; phase tabs MUST display a status dot matching
  the step's current state.
- **FR-032**: Phase tabs for phases that have not yet executed MUST be shown in a greyed-out
  style, and the system MUST expose a setting to control whether those tabs are locked or
  freely clickable.
- **FR-033**: The Overview sub-page MUST show the step name, current status, elapsed time,
  full description, declared inputs, declared expected outputs, start time, and a
  collapsible section showing the terminal output slice for the step's run window, plus a
  shortcut that focuses the full terminal panel on that slice.
- **FR-034**: The Overview sub-page MUST provide a history toggle that allows reviewing a
  summary of the most recent past execution of the step.
- **FR-035**: The Documents sub-page MUST scan the output path associated with the phase
  and MUST show each file with name, icon, last modified time, size, and a status badge
  chosen from `generated`, `modified`, or `missing`; when no historical record exists, the
  badge area MUST display an "unable to confirm original state" hint instead of `modified`.
- **FR-036**: Clicking a document row MUST open an inline preview that renders Markdown,
  applies syntax highlighting to code, and pretty-prints JSON/YAML.

#### Terminal Bridge

- **FR-040**: The system MUST expose a bottom terminal panel that streams PTY output in
  real time with ANSI color rendering and whose height is user-resizable.
- **FR-041**: Every rendered terminal line MUST carry a millisecond-precision timestamp
  used for correlating output with step run windows.
- **FR-042**: The system MUST provide keyword include/exclude filters, log-level filters
  (INFO/WARN/ERROR), and a regex search over the terminal buffer; filters MUST NOT
  destructively alter the underlying history buffer.
- **FR-043**: The system MUST provide operations to copy the visible output to the
  clipboard, export as `.log` or `.txt` with an ASCII header, clear the view without
  erasing history, and lock or unlock auto-scroll.
- **FR-044**: The system MAY allow forwarding user-typed commands into the PTY process and
  MAY expose quick-command buttons for common Spec-Kit invocations, gated behind an
  explicit setting; this capability is OPTIONAL and is the ONLY path through which the
  product may cause command execution. The main workspace MUST NOT expose Run, Retry,
  Reset, or any other action controls that mutate Spec-Kit state; SpecLens is strictly
  observational for v1 (see Clarifications 2026-04-11).
- **FR-045**: The system MUST apply universal visual highlight rules that are independent
  of any specific agent: success markers (`✓`, `✅`, `✔`, `[done]`, `[ok]` at line start)
  produce a green background; error markers (`✗`, `❌`, `Error`, `Failed`, `FAILED`)
  produce a red background; running markers (`→`, `▶`, `Running`, `[...]`) produce blue
  text; warning markers (`⚠`, `Warning`, `WARN`) produce yellow text; JSON lines are
  syntax-highlighted; file paths starting with `./` or `/` are underlined and clickable to
  open a preview.
- **FR-046**: The highlight rules MUST be defined in a regex-based configuration file that
  users can edit to add, modify, or remove rules.
- **FR-047**: The terminal buffer MUST use a two-tier retention model: the most recent
  10,000 lines (user-adjustable in settings) are held in memory for low-latency rendering;
  older lines MUST be flushed to `.speclens/logs/<ISO-8601-timestamp>.log` under the
  active project root. Search (FR-042), Overview slices (FR-033), and export (FR-043)
  MUST read transparently across both tiers so the complete run history is always
  recoverable.
- **FR-048**: Disk spill-over files MUST be rotated and capped per project with a
  configurable total-size ceiling (default: 512 MiB per project). When the cap is
  reached, the oldest rotation files MUST be deleted first; the user MUST see a
  non-blocking notice the first time a rotation deletes data so they can adjust the cap
  or export history before further loss.

#### Tasks Phase

- **FR-050**: The system MUST NOT automatically load any task file. On first entry to the
  `tasks` tab for a project, the system MUST show a Task File Picker containing auto-scan
  results that are not pre-checked, plus manual "select path" and "select file" affordances.
- **FR-051**: Supported task file formats are Markdown (`.md`), JSON (`.json`), YAML
  (`.yaml`), and plain text (`.txt`).
- **FR-052**: The task-file selection MUST be persisted per project and MUST be reloaded
  automatically on subsequent project opens without showing the picker, until the user
  invokes the always-visible "⚙ change source" control.
- **FR-053**: A Task File Bar MUST always be visible when viewing the Tasks sub-page, even
  when only one file is selected; scroll arrows appear when tabs overflow; each tab shows
  filename (with parent directory when needed for disambiguation), a completion status
  badge, and a close button.
- **FR-054**: A "+ add file" control MUST allow appending more task files after the initial
  selection; per-tab filter and search state MUST be maintained independently for each tab.
- **FR-055**: When more than one task file is loaded, the system MUST display a combined
  progress bar across all files AND a per-current-file progress bar; with a single file the
  system MUST display a single progress bar. Progress bar color MUST be amber for 0–49%,
  blue for 50–79%, and green for 80–100%.
- **FR-056**: The system MUST provide filter chips (All / Completed / Incomplete) and a
  search box that act only on the currently active task file tab, and incomplete tasks MUST
  sort above completed tasks.
- **FR-057**: Each task row MUST be expandable to show the full description, links to
  related documents that can be clicked to jump to the Documents sub-page, a terminal slice
  approximating the 30 seconds before completion labeled "≈ 30 s before completion", and an
  error summary when the task failed.

#### AI Agent Compatibility

- **FR-060**: The system MUST officially support the 12 AI Agents enumerated in the PRD:
  Claude Code, GitHub Copilot, Gemini CLI, Cursor, Windsurf, Amazon Q Developer, Codex CLI,
  Qwen Code, opencode, Kilo Code, Auggie CLI, and Roo Code.
- **FR-061**: Agent detection MUST follow a strict priority order: (1) read the `--ai`
  value recorded in the project's Spec-Kit configuration; (2) scan for the known per-agent
  filesystem fingerprints and show an "auto-detected" label when matched; (3) fall back to
  a "generic mode" label when no match is found. All product features MUST remain available
  in generic mode.

#### Notifications

- **FR-070**: The system MUST deliver OS-native notifications when a Spec step completes or
  fails, and MUST expose a setting to disable these notifications; environment-check
  outcomes MUST be surfaced as in-app toast notifications regardless of the OS notification
  setting.

#### Non-Functional Requirements

- **FR-080**: Cold-start application launch MUST complete in under 2 seconds on the
  supported hardware baselines.
- **FR-081**: End-to-end terminal output rendering latency MUST stay under 100 ms at the
  95th percentile.
- **FR-082**: Scrolling the step list MUST sustain at least 60 frames per second on
  supported hardware.
- **FR-083**: The system MUST support macOS 12 or later (Apple Silicon and Intel), Windows
  10 and 11 (x64), and Ubuntu 22.04+, Debian 12+, and Fedora 38+ (x64), providing a
  consistent user experience across these platforms.
- **FR-084**: The system MUST provide global keyboard shortcuts, at minimum "open project"
  (Cmd/Ctrl+O) and "clear terminal output" (Cmd/Ctrl+K), and MUST respect the operating
  system's light/dark theme preference automatically. Terminal font size MUST be
  user-adjustable.
- **FR-087**: The user interface MUST ship in two locales for v1: English (`en`) and
  Traditional Chinese (`zh-TW`). All user-facing UI strings MUST live in externalized
  resource files; no translatable copy may be hard-coded. Product name, AI Agent brand
  names, and raw Spec-Kit CLI output are NOT translated.
- **FR-088**: On first launch, the UI language MUST default to the OS language when a
  matching locale exists, otherwise fall back to English. The user MUST be able to
  override the UI language at any time via a setting that takes effect without requiring
  an application restart.

#### Window & Session Model

- **FR-100**: A single SpecLens process MUST support multiple top-level windows running
  concurrently, each bound to exactly one project. Opening a second project MUST open a
  new window rather than replace the current one; the user MUST also be able to open a
  second window on the same project if they choose.
- **FR-101**: Each window MUST own an independent PTY session, terminal buffer
  (in-memory tier and `.speclens/logs/` spill-over per FR-047), phase tab state,
  Documents state, and Tasks state. State changes in one window MUST NOT affect another
  window.
- **FR-102**: Cross-project user state (FR-090), the UI language (FR-087), highlight
  rules (FR-046), and any credentials stored per FR-086 MUST be shared by all windows
  of the same process; a change made in one window MUST become visible to the others
  without requiring a restart.
- **FR-103**: Closing the last window of a project MUST flush pending per-project state
  to `.speclens/` before the window is destroyed. Closing the final window of the
  process MUST persist all cross-project state before exit.
- **FR-085**: The system MUST enforce a two-tier safety model: PTY command execution MUST
  be rooted to the project directory; file writes initiated by the app (e.g., exports) MUST
  be confined to the project directory; read-only access (task-file selection, preview) MAY
  use paths chosen through the OS file dialog.
- **FR-086**: The system MUST store any credentials or secrets in the OS-native secret
  store (Keychain on macOS, Credential Manager on Windows, Secret Service on Linux) and
  MUST NOT collect user data. All computation MUST stay on the local machine.

#### State Persistence

- **FR-090**: Cross-project user state — the recent-projects list (FR-003), global
  preferences, window/panel layout, and the user-editable highlight-rule configuration
  (FR-046) — MUST be persisted in the OS application-data directory: macOS
  `~/Library/Application Support/SpecLens/`, Windows `%APPDATA%\SpecLens\`, Linux
  `~/.config/speclens/`. This data MUST NOT be written anywhere inside a project
  directory.
- **FR-091**: Per-project state — the last selected step and phase tab (FR-004), the
  per-project task-file selection (FR-052), and the document content-hash history used
  by the Documents badges (FR-035) — MUST be persisted in a `.speclens/` directory at
  the project root. SpecLens MUST, on first write to `.speclens/` in a given project,
  surface a one-time prompt recommending that the user add `.speclens/` to that project's
  `.gitignore`.
- **FR-092**: A project MUST remain usable even when its `.speclens/` directory is
  absent or unreadable: all per-project state reverts to sensible defaults (no history
  for Documents badges, no prior task-file selection, no remembered step/tab) and the
  directory is re-created on next successful write.
- **FR-093**: No user-state file written by SpecLens MAY be written outside the two
  locations defined in FR-090 and FR-091. This rule is enforced in addition to the
  two-tier safety model in FR-085.

### Key Entities

- **Project (Workspace)**: A local directory containing a Spec-Kit configuration. Has a
  name, path, last-modified timestamp, detected or recorded AI Agent, workspace memory
  (last step and tab), and a pointer to its selected task files.
- **Step**: One item in the Spec-Kit step list. Has an ID/name, description, type,
  prerequisite relationships, status (`pending` / `running` / `completed` / `failed` /
  `skipped` / `blocked`), start time, end time, and a link to the terminal output slice
  for its run window.
- **Phase Tab**: The UI representation of a phase (spec, tasks, implement, review, test,
  etc.). Exposes three sub-pages (Overview, Documents, and Tasks for `tasks` only) and
  shows a status dot that mirrors the underlying step's state.
- **Document Artifact**: A file produced by a phase. Has a filename, file type, size, last
  modified timestamp, and a content hash used to derive its status badge (`generated`,
  `modified`, `missing`, or "unable to confirm original state").
- **Task File**: A user-selected file (`.md` / `.json` / `.yaml` / `.txt`) containing
  tasks. Has a path, parent directory, per-file filter/search state, and a completion
  ratio.
- **Task**: An individual item inside a task file. Has a title, full description, status
  (complete or incomplete), related document references, an approximate terminal slice
  near completion, and an error summary when failed.
- **Terminal Line**: One line of PTY output. Has a millisecond timestamp, raw text, and
  rendered text with ANSI color and highlight-rule annotations.
- **AI Agent**: The currently detected agent. Has an ID, display name, icon, detection
  source (`configured`, `auto-detected`, or `generic`), and filesystem fingerprint rules.
- **Recent Project Entry**: A pointer to a previously opened project. Has a project name,
  path, and last-opened timestamp; the list holds at most 10 entries.
- **Highlight Rule**: A regex-based pattern plus visual treatment used to decorate terminal
  output; user-editable.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: From pressing the application icon to seeing the welcome screen, 95% of cold
  starts complete in under 2 seconds.
- **SC-002**: After selecting a valid project folder, 95% of users reach a fully rendered
  workspace (steps, tabs, and terminal panel visible) within 3 seconds.
- **SC-003**: End-to-end latency between an agent emitting a line of output and that line
  appearing in the terminal panel stays at or below 100 milliseconds at the 95th
  percentile.
- **SC-004**: When a user selects a completed step, the Overview sub-page surfaces the
  correctly correlated terminal slice at least 90% of the time as measured against ground
  truth in evaluation datasets.
- **SC-005**: A new user can open a project, pick a task file via the Task File Picker, and
  check off their first task in under 2 minutes during onboarding.
- **SC-006**: The application successfully launches and opens a project on at least 99.5%
  of attempts across the supported operating systems.
- **SC-007**: Across every supported AI Agent, users can complete the same core flow (open
  project → view steps → inspect documents → manage tasks) with no agent-specific
  breakage; the share of bug reports flagged as "only reproduces on one OS or one agent"
  stays below 5% of total reported issues.
- **SC-008**: At least 90% of users successfully complete the environment check (install or
  update Spec-Kit CLI, or confirm it is current) on their first attempt without manual
  troubleshooting.
- **SC-009**: Net Promoter Score from target users (AI developers using Spec-Kit) reaches
  40 or higher within 6 months of first release.
- **SC-010**: Zero user data leaves the local machine in normal operation, verified by
  automated traffic audits in each release.

## Assumptions

- Spec-Kit CLI is installable via a publicly documented command that SpecLens can invoke
  programmatically on each supported OS, and its release metadata is accessible (e.g., via
  the upstream release API) for version comparison and changelog preview.
- Each project directory is the single authoritative root for all Spec-Kit outputs for that
  project; output paths used by phases are either configured or discoverable relative to
  that root.
- Each step has an unambiguous run window (a `running` start timestamp and a terminal state
  timestamp) available from the execution environment, enabling terminal-output correlation.
- File change detection uses content hashing plus a filesystem watcher; the cost of hashing
  phase output files at their expected sizes is low enough to keep the UI responsive.
- The set of "known per-agent fingerprints" used for detection matches the fingerprints
  listed in the PRD v1.3.0 agent support matrix and is treated as configuration that can be
  updated over time without a product rewrite.
- Users have permission to read and write within the project directory; file operations
  outside the project directory are not a product requirement except for read-only previews
  initiated through the OS file dialog.
- "Generic mode" coverage (full feature availability for unknown agents) is acceptable to
  users as a fallback and does not require any agent-specific semantic parsing.
- Notification delivery uses the OS-native notification APIs available on macOS, Windows,
  and the supported Linux distributions, and the user can disable it at any time.
- The product is single-user and single-device: no cloud sync, multi-user collaboration,
  Spec-Kit configuration editor, direct AI-model integration, agent-specific semantic
  output parsing, or mobile version is in scope for v1.0.
