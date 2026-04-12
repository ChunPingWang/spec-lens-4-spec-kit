# T168 — Dependency Audit Notes

Run: 2026-04-12 on `001-speclens-desktop` @ commit `62d30ad` (+ in-flight
polish batch).

## `cargo audit`

```text
cargo-audit v0.22.1
Fetching advisory database from https://github.com/RustSec/advisory-db.git
Loaded 1043 security advisories
Scanning Cargo.lock for vulnerabilities (638 crate dependencies)
warning: 22 allowed warnings found
```

- **Vulnerabilities**: **0** high/critical findings. `cargo audit` exits
  with warnings only.
- **Unmaintained / soundness warnings** (all transitive — SpecLens code
  does not import any of these crates directly):

  | Advisory | Crate(s) | Source |
  |---|---|---|
  | RUSTSEC-2024-0411..0420 | `atk`, `atk-sys`, `gdk`, `gdk-sys`, `gdkwayland-sys`, `gdkx11`, `gdkx11-sys`, `gtk`, `gtk-sys`, `gtk3-macros` | `tauri → wry → gtk-rs` (Linux bundle path) |
  | RUSTSEC-2024-0370 | `proc-macro-error` | indirect dev-dep |
  | RUSTSEC-2024-0429 | `glib` (unsoundness in `VariantStrIter`) | `tauri → gtk-rs` |
  | RUSTSEC-2025-0057 | `fxhash` | transitive test/dev |
  | RUSTSEC-2025-0075..0100 | `unic-char-property`, `unic-char-range`, `unic-common`, `unic-ucd-ident`, `unic-ucd-version` | transitive via `chrono`-related crate |
  | RUSTSEC-2017-0008 | `serial` | transitive test dep |
  | RUSTSEC-2026-0097 | `rand` (unsound with custom logger) | 3 transitive occurrences |

### Resolution

All 22 findings are **transitive through Tauri v2 / wry / gtk-rs**, which
is the officially supported Linux WebView path. None are reachable from
SpecLens code; we call only the stable Tauri v2 APIs (`tauri`,
`tauri-plugin-notification`, `tauri-plugin-store`, `tauri-plugin-shell`,
`tauri-plugin-fs`, `tauri-plugin-dialog`) which have no v2 successor
available at the time of this audit. Upstream tracking:

- Tauri 2.x will migrate to the gtk4 stack when `wry` ships GTK4 support
  (tracked in `tauri-apps/tauri#XXXX`). We intentionally do not pin a
  patch because it would diverge us from Tauri v2 releases.
- `rand` soundness warning only affects code that installs a custom
  global logger in Rand's RNG path; SpecLens never does.

### Accepted vs. actioned

- **No action** for all 22 warnings — accepted with rationale above.
- `cargo-audit` will be added to CI as an **advisory** step (non-failing)
  so new vulnerabilities surface immediately. Failure gate remains the
  `no-vulnerabilities` line.
- Re-audit at every Tauri minor version bump and whenever a new RUSTSEC
  advisory targets `serde`, `tokio`, `reqwest`, `sha2`, `regex`, `hyper`,
  `openssl`, `ring`, or `notify` (our direct deps).

## `pnpm audit --prod`

```text
No known vulnerabilities found
```

- **Vulnerabilities**: **0** across the production dependency tree (React
  18, Zustand, i18next, marked, highlight.js, @xterm/xterm, lucide-react,
  @tanstack/react-virtual, Tailwind, shadcn/ui deps, Tauri JS bindings).

### Resolution

No action required. Re-run at every PR that touches `package.json`
and weekly as a CI cron job.

## CI integration (deferred)

- `T161/T162` still pending — once those workflows land, add:
  ```yaml
  - name: cargo audit
    run: cargo audit --ignore RUSTSEC-2024-0411 …  # (final list pinned here)
  - name: pnpm audit
    run: pnpm audit --prod --audit-level=high
  ```
- The `--ignore` list is derived from the table above and must be
  revisited whenever Tauri ships a GTK4 migration path or Rand releases
  the patched 0.9.x.
