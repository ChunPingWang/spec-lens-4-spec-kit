import { defineConfig } from "@playwright/test";

/**
 * Playwright configuration for SpecLens E2E smoke tests.
 *
 * The real tauri-driver spawn is wired up in `tests/e2e/*.spec.ts` —
 * this config only sets sane defaults (timeouts, reporters, retries).
 */
export default defineConfig({
  testDir: "tests/e2e",
  timeout: 60_000,
  expect: { timeout: 10_000 },
  retries: process.env.CI ? 1 : 0,
  fullyParallel: false,
  reporter: [["list"], ["html", { open: "never" }]],
  use: {
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
  },
});
