# SpecLens — Developer Quickstart

**Branch**: `001-speclens-desktop` | **Date**: 2026-04-11 | **Plan**: [plan.md](./plan.md)

This quickstart walks a new contributor from a clean clone to a running
SpecLens build plus a green test suite. It is the hands-on companion to
`plan.md` and is validated by Playwright smoke tests in CI.

## 1. Prerequisites

Install these once per machine. Versions below are the minimum supported.

| Tool | Minimum | Notes |
|------|---------|-------|
| **Rust** | stable ≥ 1.78.0 | Install via [rustup](https://rustup.rs). Installs `cargo`, `rustc`, `rustfmt`, `clippy`. |
| **Node.js** | ≥ 20 LTS | Install via [fnm](https://github.com/Schniz/fnm) or [nvm](https://github.com/nvm-sh/nvm). |
| **pnpm** | ≥ 9 | `corepack enable && corepack prepare pnpm@latest --activate` |
| **Tauri v2 prerequisites** | per OS | See [tauri.app/start/prerequisites](https://tauri.app/start/prerequisites). |

### Platform-specific Tauri prerequisites

- **macOS**: Xcode Command Line Tools (`xcode-select --install`).
- **Windows**: Visual Studio 2022 Build Tools with the "Desktop
  development with C++" workload; WebView2 Runtime.
- **Linux**: `libwebkit2gtk-4.1-dev`, `build-essential`, `curl`, `wget`,
  `file`, `libxdo-dev`, `libssl-dev`, `libayatana-appindicator3-dev`,
  `librsvg2-dev` (package names vary by distro).

### Optional but recommended

- `cargo install cargo-nextest` — faster, prettier Rust test runs.
- `cargo install cargo-watch` — rerun Rust tests on file changes.

## 2. Clone and install

```bash
git clone https://github.com/ChunPingWang/spec-lens-4-spec-kit.git
cd spec-lens-4-spec-kit
git switch 001-speclens-desktop       # active feature branch
pnpm install                           # frontend deps (src/)
cargo fetch --manifest-path src-tauri/Cargo.toml   # prime Rust deps
```

`pnpm install` installs React, Vite, Tailwind, shadcn/ui, @xterm/xterm,
Zustand, i18next, and the test harness (Vitest, Playwright). The Rust
workspace under `src-tauri/` is compiled on the first `pnpm tauri dev`.

## 3. Run the app in dev mode

```bash
pnpm tauri dev
```

This starts Vite on `http://localhost:5173` and launches the Tauri native
window pointed at it. Hot-reload is enabled for React and TypeScript;
Rust changes trigger a full `cargo` rebuild and window restart.

On first launch you should see:

1. The **Welcome** page with an empty recent-projects list.
2. A "Pick a project" button.
3. A language selector defaulting to your OS locale (`en` or `zh-TW`).

## 4. Smoke flow

Run through this in dev mode to verify your environment:

1. Click **Pick a project** and choose any folder that contains a
   `.specify/` directory (the `spec-lens-4-spec-kit` repo itself works).
2. A **Workspace** window opens with:
   - The Spec-Kit environment badge (green / amber / red) in the top bar.
   - The **Steps** list on the left.
   - A **Phase Tabs** panel showing Overview / Documents / Tasks for the
     currently selected step.
   - The **Terminal** panel collapsed at the bottom.
3. Click a step that has generated files (e.g., `plan`); the Documents
   tab should show `plan.md`, `research.md`, `data-model.md`,
   `contracts/ipc.md`, `quickstart.md` with a **generated** badge.
4. Click the **Tasks** tab. If no task files exist yet, the **Task File
   Picker** modal opens. Cancel it; confirm the task-file bar stays empty.
5. Expand the terminal panel. Type `echo hello` and press enter. The
   output should stream back with a millisecond timestamp and a default
   `info` highlight.
6. Close the window. Reopen the project from the Welcome page. Your last
   step, last tab, and terminal collapse state should be restored.

If any of those steps fail, open the **Troubleshooting** section below.

## 5. Tests

Run all tests locally before pushing:

```bash
# Rust unit + integration tests
cargo test --manifest-path src-tauri/Cargo.toml
#   or: cargo nextest run --manifest-path src-tauri/Cargo.toml

# Rust benches (criterion; also runs in CI on changed paths)
cargo bench --manifest-path src-tauri/Cargo.toml

# Frontend unit tests (Vitest + React Testing Library)
pnpm test

# End-to-end (Playwright + tauri-driver)
pnpm test:e2e
```

**Coverage**: CI enforces ≥ 80 % line coverage on changed code per
Constitution II. Locally, use `cargo llvm-cov --workspace` and
`pnpm test -- --coverage`.

**Linting & type-checking** (run before every commit):

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
pnpm lint
pnpm typecheck
```

## 6. Producing a release build

```bash
pnpm tauri build
```

Output artifacts per platform:

- **macOS**: `src-tauri/target/release/bundle/dmg/SpecLens_<version>_<arch>.dmg`
- **Windows**: `src-tauri/target/release/bundle/msi/SpecLens_<version>_x64_en-US.msi`
- **Linux**: `.AppImage` and `.deb` under
  `src-tauri/target/release/bundle/`

Cold-start budget: ≤ 2 s (SC-001 / FR-080). Benchmark it with
`pnpm bench:cold-start` (Playwright-driven).

## 7. Project layout (cheat sheet)

```text
spec-lens-4-spec-kit/
├── src/                          # React + TypeScript (Vite)
│   ├── i18n/{en,zh-TW}.json      # UI strings (FR-087 / FR-088)
│   ├── pages/                    # Welcome, Workspace
│   ├── components/               # layout, steps, phase, tasks, terminal, ...
│   ├── stores/                   # Zustand slices
│   ├── hooks/                    # useSpeckitEvents, usePtySession, ...
│   └── lib/                      # tauri bridge, markdown, highlightRules
├── src-tauri/                    # Rust backend
│   ├── src/commands/             # IPC handlers (see contracts/ipc.md)
│   ├── src/services/             # speckit_engine, terminal_bridge, ...
│   ├── src/models/               # serde-serializable entities
│   └── benches/                  # criterion benches
├── tests/e2e/                    # Playwright + tauri-driver specs
└── specs/001-speclens-desktop/   # This feature's spec-kit artifacts
```

## 8. Environment variables

| Name | Purpose | Default |
|------|---------|---------|
| `SPECLENS_LOG` | `tracing` filter (e.g., `speclens=debug`) | `info` |
| `SPECLENS_APP_DATA_DIR` | Override OS app-data dir (tests) | OS default |
| `SPECLENS_DISABLE_FS_WATCH` | Disable `notify` for deterministic tests | unset |

Put local overrides in `.env.local` (git-ignored). Never put secrets
there — SpecLens has no secrets to load in v1.

## 9. Troubleshooting

**"Tauri CLI not found"**
```bash
pnpm add -D @tauri-apps/cli
```

**macOS: "cannot open because the developer cannot be verified"**
- Remove quarantine during dev: `xattr -dr com.apple.quarantine target/debug/bundle`
- Or use `pnpm tauri dev` which skips bundling.

**Windows: WebView2 missing**
- Install the Evergreen WebView2 runtime from Microsoft.

**Linux: "libwebkit2gtk-4.1" package not found**
- On Ubuntu 22.04 pin the `4.0` variant:
  `sudo apt install libwebkit2gtk-4.0-dev` and update `tauri.conf.json`
  accordingly (already handled in CI).

**PTY fails to start on Windows**
- Ensure Windows 10 build 19041 or later (ConPTY requirement).

**Terminal output is stuck / not rendering**
- Check `SPECLENS_LOG=speclens::services::terminal_bridge=trace pnpm tauri dev`
- Confirm the buffer cap via the **Settings > Terminal** panel.

**zh-TW strings not showing**
- Verify `src/i18n/zh-TW.json` is present and the language is selected in
  settings; fall back to OS locale detection by choosing "System" in the
  language dropdown.

## 10. Where to go next

- Read [plan.md](./plan.md) for the implementation plan.
- Read [research.md](./research.md) for the Decision / Rationale
  / Alternatives log.
- Read [data-model.md](./data-model.md) for entity definitions.
- Read [contracts/ipc.md](./contracts/ipc.md) for the IPC surface.
- Run `/speckit.tasks` to generate the task list once the plan is merged.
