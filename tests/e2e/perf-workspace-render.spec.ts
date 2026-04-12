/**
 * T155 — Workspace render performance (SC-002).
 *
 * Asserts that the full three-pane Workspace is visible within 3 s (p95)
 * from the moment a folder is "picked" (i.e. project_open IPC resolves).
 *
 * The measurement starts immediately after the Welcome page is shown and
 * the project path is injected via the `?open=` query param (which
 * simulates the dialog confirm). The end-point is the first moment that
 * all three workspace panes are in the DOM:
 *   - Left steps sidebar  (role="complementary", name matches steps label)
 *   - Centre phase/tabs panel (role="tablist")
 *   - Right terminal panel (data-testid="terminal-xterm-mount")
 */

import { test, expect, type Page } from "@playwright/test";
import * as path from "path";

const FIXTURE_OK = path.resolve(import.meta.dirname, "../fixtures/speckit-project-ok");
const BUDGET_MS = 3_000;
const SAMPLE_COUNT = 10;

async function measureWorkspaceRender(page: Page, projectPath: string): Promise<number> {
  // Navigate to Welcome first to simulate a clean state.
  await page.goto("/");
  await expect(page.getByRole("button", { name: /open project/i })).toBeEnabled({
    timeout: 5_000,
  });

  // Simulate folder pick by injecting the path.
  const t0 = Date.now();
  await page.goto(`/?open=${encodeURIComponent(projectPath)}`);

  // Dismiss env dialog immediately if it blocks (green → auto-dismissed;
  // amber/red → click continue to keep timing focused on render).
  const continueBtn = page.locator('[data-testid="env-continue-button"]');
  if (await continueBtn.isVisible({ timeout: 2_000 }).catch(() => false)) {
    await continueBtn.click();
  }

  // Wait for all three panes to be present simultaneously.
  await Promise.all([
    expect(page.getByRole("region", { name: "Workspace" })).toBeVisible({
      timeout: BUDGET_MS * 3,
    }),
    expect(page.locator('[data-testid="terminal-xterm-mount"]')).toBeVisible({
      timeout: BUDGET_MS * 3,
    }),
    expect(page.getByRole("tablist")).toBeVisible({ timeout: BUDGET_MS * 3 }),
  ]);

  return Date.now() - t0;
}

test.describe("T155 — Workspace render performance ≤ 3 s p95 (SC-002)", () => {
  test("single workspace render is within 3× budget (sanity)", async ({ page }) => {
    const elapsed = await measureWorkspaceRender(page, FIXTURE_OK);
    console.log(`Workspace render single sample: ${elapsed} ms`);
    expect(elapsed).toBeLessThan(BUDGET_MS * 3);
  });

  test(`p95 workspace render ≤ ${BUDGET_MS} ms over ${SAMPLE_COUNT} samples`, async ({
    page,
  }) => {
    const latencies: number[] = [];

    for (let i = 0; i < SAMPLE_COUNT; i++) {
      latencies.push(await measureWorkspaceRender(page, FIXTURE_OK));
    }

    latencies.sort((a, b) => a - b);
    const p95 = latencies[Math.ceil(SAMPLE_COUNT * 0.95) - 1]!;
    const p50 = latencies[Math.floor(SAMPLE_COUNT * 0.5)]!;

    console.log(
      `Workspace render latencies (ms): ${latencies.join(", ")}\n` +
        `p50=${p50} ms, p95=${p95} ms`,
    );

    expect(p95).toBeLessThanOrEqual(BUDGET_MS);
  });

  test("all three workspace panes are visible after render", async ({ page }) => {
    await page.goto(`/?open=${encodeURIComponent(FIXTURE_OK)}`);
    const continueBtn = page.locator('[data-testid="env-continue-button"]');
    if (await continueBtn.isVisible({ timeout: 3_000 }).catch(() => false)) {
      await continueBtn.click();
    }

    // Steps sidebar.
    await expect(page.getByRole("region", { name: "Workspace" })).toBeVisible({
      timeout: BUDGET_MS,
    });

    // Phase / tablist.
    await expect(page.getByRole("tablist")).toBeVisible({ timeout: BUDGET_MS });

    // Terminal mount.
    await expect(page.locator('[data-testid="terminal-xterm-mount"]')).toBeVisible({
      timeout: BUDGET_MS,
    });
  });

  test("progress bar is rendered before the budget elapses", async ({ page }) => {
    await page.goto(`/?open=${encodeURIComponent(FIXTURE_OK)}`);
    const continueBtn = page.locator('[data-testid="env-continue-button"]');
    if (await continueBtn.isVisible({ timeout: 3_000 }).catch(() => false)) {
      await continueBtn.click();
    }

    // The workspace renders a progressbar for step completion.
    await expect(page.getByRole("progressbar")).toBeVisible({ timeout: BUDGET_MS });
  });
});
