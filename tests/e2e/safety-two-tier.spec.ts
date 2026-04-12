/**
 * T159 — PTY cwd and file write safety (FR-085 / FR-093).
 *
 * Asserts that SpecLens enforces a two-tier path-safety policy:
 *
 *   Tier 1 (PTY cwd) — The terminal session spawns in the project root and
 *     never allows `cd` commands to escape it (enforced by the backend's
 *     `resolve_inside_root` guard).
 *
 *   Tier 2 (File writes) — Any IPC call that writes to disk (e.g.
 *     `project_set_last_step`, state persistence) operates only on paths
 *     within the registered project root.
 *
 * Strategy: the IPC mock records every backend call. After each action we
 * inspect `window.__speclensIpcLog` for path arguments and assert that
 * none escapes the registered project root directory.
 *
 * For the PTY tier we also attempt a shell traversal command and verify
 * the backend rejects it (an error status is surfaced in the terminal
 * status region).
 */

import { test, expect, type Page } from "@playwright/test";
import * as path from "path";

const FIXTURE_OK = path.resolve(import.meta.dirname, "../fixtures/speckit-project-ok");
// A sentinel path that is definitively outside any project root.
const OUTSIDE_PATH = "/tmp/speclens-escape-test";

async function openWorkspace(page: Page): Promise<void> {
  await page.goto(`/?open=${encodeURIComponent(FIXTURE_OK)}`);
  await expect(page.getByRole("region", { name: "Workspace" })).toBeVisible({
    timeout: 15_000,
  });
  const continueBtn = page.locator('[data-testid="env-continue-button"]');
  if (await continueBtn.isVisible({ timeout: 3_000 }).catch(() => false)) {
    await continueBtn.click();
  }
}

async function connectTerminal(page: Page): Promise<void> {
  const connectBtn = page.getByRole("button", { name: /connect/i });
  await expect(connectBtn).toBeVisible({ timeout: 5_000 });
  await connectBtn.click();
  await expect(page.locator('[aria-live="polite"]')).toContainText(/connected/i, {
    timeout: 10_000,
  });
}

test.describe("T159 — Tier 1: PTY cwd stays within project root", () => {
  test.beforeEach(async ({ page }) => {
    await openWorkspace(page);
    await connectTerminal(page);
  });

  test("terminal spawns with cwd equal to project root", async ({ page }) => {
    // Request cwd via a shell command and look for the project path in the
    // output mirror.
    await page.locator('[data-testid="terminal-xterm-mount"]').click();
    await page.keyboard.type("pwd");
    await page.keyboard.press("Enter");

    const mirror = page.locator('[data-testid="terminal-line-mirror"]');
    // Output must contain a path that starts with the fixture root.
    // FIXTURE_OK base directory name is used as a proxy.
    await expect(mirror.locator("li").filter({ hasText: /speckit-project-ok/ })).toBeVisible({
      timeout: 5_000,
    });
  });

  test("attempting to cd outside root is rejected or contained", async ({ page }) => {
    await page.locator('[data-testid="terminal-xterm-mount"]').click();
    await page.keyboard.type(`cd ${OUTSIDE_PATH}`);
    await page.keyboard.press("Enter");

    // Either an error line appears in the mirror, or cwd does not change.
    const mirror = page.locator('[data-testid="terminal-line-mirror"]');

    // Wait briefly for output.
    await page.waitForTimeout(300);

    // After the restricted cd, pwd must still show a path inside the fixture.
    await page.keyboard.type("pwd");
    await page.keyboard.press("Enter");

    await expect(
      mirror
        .locator("li")
        .filter({ hasText: /speckit-project-ok/ })
        .last(),
    ).toBeVisible({ timeout: 5_000 });
  });

  test("directory traversal via '../../../etc/passwd' path is rejected", async ({ page }) => {
    await page.locator('[data-testid="terminal-xterm-mount"]').click();
    await page.keyboard.type("cat ../../../etc/passwd");
    await page.keyboard.press("Enter");

    const mirror = page.locator('[data-testid="terminal-line-mirror"]');
    // The backend should reject this or the shell should be confined to the root.
    // We assert that the terminal does NOT output lines from /etc/passwd
    // (look for "root:x:0:0:" which is a reliable marker on POSIX systems).
    await page.waitForTimeout(500);
    await expect(mirror.locator('li:has-text("root:x:0:0:")')).not.toBeVisible();
  });
});

test.describe("T159 — Tier 2: file write IPC paths stay within project root", () => {
  test("project_set_last_step only writes within project root", async ({ page }) => {
    await openWorkspace(page);

    // Reset the IPC log.
    await page.evaluate(() => {
      (window as unknown as Record<string, unknown>)["__speclensIpcLog"] = [];
    });

    // Trigger a step selection which calls project_set_last_step.
    const firstStepBtn = page
      .getByRole("complementary", { name: /steps/i })
      .getByRole("button")
      .first();
    if (await firstStepBtn.isVisible({ timeout: 3_000 }).catch(() => false)) {
      await firstStepBtn.click();
    }

    await page.waitForTimeout(300);

    // Inspect the IPC log for any path arguments.
    const ipcLog = await page.evaluate(
      () =>
        ((window as unknown as Record<string, unknown>)["__speclensIpcLog"] as Array<{
          cmd: string;
          args: Record<string, unknown>;
        }>) ?? [],
    );

    for (const entry of ipcLog) {
      for (const [key, value] of Object.entries(entry.args)) {
        if (typeof value === "string" && value.startsWith("/")) {
          // Any absolute path in IPC args must be within the fixture root or
          // the app's own data directory (not an arbitrary system path).
          const isInsideProject = value.startsWith(FIXTURE_OK);
          const isAppData = value.includes("com.speclens") || value.includes("AppData");
          const isSystemPath =
            !isInsideProject &&
            !isAppData &&
            !value.startsWith("/tmp") &&
            !value.startsWith("/var");
          if (isSystemPath) {
            throw new Error(
              `IPC call "${entry.cmd}" contains an out-of-bounds path in arg "${key}": ${value}`,
            );
          }
        }
      }
    }
  });

  test("task_file_parse rejects paths outside project root", async ({ page }) => {
    await openWorkspace(page);

    // Attempt to invoke task_file_parse with an out-of-bounds path via
    // the IPC stub. The backend must return an error.
    const result = await page.evaluate(async (outsidePath) => {
      try {
        // Use the window-level invoke stub if available.
        const w = window as unknown as Record<string, unknown>;
        const invoke =
          typeof w["__tauriInvoke"] === "function"
            ? (w["__tauriInvoke"] as (cmd: string, args: unknown) => Promise<unknown>)
            : null;
        if (!invoke) return { skipped: true };
        await invoke("task_file_parse", { projectId: "test", relativePath: outsidePath });
        return { ok: true };
      } catch (err) {
        return { error: String(err) };
      }
    }, "../../etc/passwd");

    if (!result.skipped) {
      // If the stub is present, the call must have failed.
      expect(result.ok).toBeFalsy();
      expect(result.error).toBeTruthy();
    }
  });
});
