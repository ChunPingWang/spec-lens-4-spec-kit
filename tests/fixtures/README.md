# Test Fixtures

Fixture projects used by both Rust unit/integration tests (`src-tauri/tests/**`)
and Playwright E2E tests (`tests/e2e/**`).

## Layout

| Directory                                    | Purpose                                                   |
| --------------------------------------------- | --------------------------------------------------------- |
| `speckit-project-ok/`                         | Valid Spec-Kit project with 1 feature and sample docs     |
| `speckit-project-empty/`                      | Directory with `.specify/` but no features                |
| `speckit-project-broken/`                     | Directory with no Spec-Kit configuration at all           |
| `task-files/`                                 | Sample `tasks.md`, `tasks.json`, `tasks.yaml`, `tasks.txt` |
| `agent-*` (added in US6)                      | One project per supported agent fingerprint               |

Fixtures MUST be small, self-contained, and MUST NOT contain real user data.
