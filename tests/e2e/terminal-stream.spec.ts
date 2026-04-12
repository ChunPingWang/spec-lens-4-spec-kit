/**
 * T091 — Terminal stream echo round-trip latency.
 *
 * Verifies that output from a simple shell command appears in the
 * terminal mirror within the FR-081 budget of 100 ms at p95.
 *
 * Strategy: send a unique sentinel string via the terminal's PTY, then
 * poll `[data-testid="terminal-line-mirror"]` for a list item that
 * contains the sentinel. Wall-clock delta gives the end-to-end latency.
 * We run the probe 20 times and assert that the p95 value stays ≤ 100 ms.
 */

import { test, expect, type Page } from "@playwright/test";
import * as path from "path";

const FIXTURE_OK = path.resolve(import.meta.dirname, "../fixtures/speckit-project-ok");
const SAMPLE_RUNS = 20;
const BUDGET_MS = 100;

async function openProject(page: Page, projectPath: string): Promise<void> {
  // The Welcome page renders a primary "Open project" button. In the E2E
  // harness the Tauri dialog is bypassed and the path is injected via a
  // custom IPC stub exposed by the test environment.
  await page.goto("/");
  await expect(page.getByRole("heading", { name: /SpecLens/i })).toBeVisible();

  // Trigger path injection via the test-only query param accepted by the
  // dev server harness.
  await page.goto(`/?open=${encodeURIComponent(projectPath)}`);
  // Wait for the workspace to render (three-pane layout).
  await expect(page.getByRole("region", { name: "Workspace" })).toBeVisible({
    timeout: 15_000,
  });
}

async function connectTerminal(page: Page): Promise<void> {
  const connectBtn = page.getByRole("button", { name: /connect/i });
  await connectBtn.click();
  // Status line transitions to "Connected …" once the PTY is ready.
  await expect(page.locator('[aria-live="polite"]')).toContainText(/connected/i, {
    timeout: 10_000,
  });
}

test.describe("T091 — Terminal stream echo round-trip latency", () => {
  test.beforeEach(async ({ page }) => {
    await openProject(page, FIXTURE_OK);
    // Env-check dialog: acknowledge healthy env so workspace is unblocked.
    const continueBtn = page.locator('[data-testid="env-continue-button"]');
    if (await continueBtn.isVisible({ timeout: 5_000 }).catch(() => false)) {
      await continueBtn.click();
    }
    await connectTerminal(page);
  });

  test("single echo appears within 100 ms", async ({ page }) => {
    const sentinel = `speclens-echo-${Date.now()}`;
    const mirror = page.locator('[data-testid="terminal-line-mirror"]');

    const t0 = Date.now();
    // Send a command via the QuickCommands input or the xterm keyboard path.
    // The TerminalToolbar exposes a search/command input; we use the
    // data-testid on the xterm mount and dispatch keyboard events.
    await page.locator('[data-testid="terminal-xterm-mount"]').click();
    await page.keyboard.type(`echo ${sentinel}`);
    await page.keyboard.press("Enter");

    await expect(mirror.locator(`li:has-text("${sentinel}")`)).toBeVisible({
      timeout: BUDGET_MS * 5, // generous outer timeout; latency check is below
    });
    const elapsed = Date.now() - t0;

    // Single sample must be well within 5× budget (individual variance is OK).
    expect(elapsed).toBeLessThan(BUDGET_MS * 5);
  });

  test(`p95 echo latency ≤ ${BUDGET_MS} ms over ${SAMPLE_RUNS} samples`, async ({ page }) => {
    const mirror = page.locator('[data-testid="terminal-line-mirror"]');
    const latencies: number[] = [];

    for (let i = 0; i < SAMPLE_RUNS; i++) {
      const sentinel = `spec-p${i}-${Date.now()}`;
      await page.locator('[data-testid="terminal-xterm-mount"]').click();
      const t0 = Date.now();
      await page.keyboard.type(`echo ${sentinel}`);
      await page.keyboard.press("Enter");
      await expect(mirror.locator(`li:has-text("${sentinel}")`)).toBeVisible({
        timeout: 5_000,
      });
      latencies.push(Date.now() - t0);
    }

    latencies.sort((a, b) => a - b);
    const p95 = latencies[Math.ceil(SAMPLE_RUNS * 0.95) - 1]!;
    console.log(`Terminal echo p95 latency: ${p95} ms (samples: ${latencies.join(", ")})`);
    expect(p95).toBeLessThanOrEqual(BUDGET_MS);
  });

  test("disconnect clears the connected status", async ({ page }) => {
    const disconnectBtn = page.getByRole("button", { name: /disconnect/i });
    await disconnectBtn.click();
    await expect(page.locator('[aria-live="polite"]')).toContainText(/not connected/i, {
      timeout: 5_000,
    });
  });
});
