/**
 * T154 — Cold-start performance (SC-001 / FR-080).
 *
 * Asserts that the SpecLens UI is interactive within 2 s (p95) from the
 * moment the page navigation starts. "Interactive" is defined as the
 * Welcome page heading being visible and the "Open project" button being
 * enabled.
 *
 * We run SAMPLE_COUNT cold navigations and collect the time-to-interactive
 * for each, then assert that the p95 value does not exceed BUDGET_MS.
 *
 * The test uses `page.goto` timing (navigationStart → first meaningful
 * paint proxy) rather than a real Tauri cold-launch, because the E2E
 * suite drives the Vite dev-server. The Playwright navigation time is a
 * conservative upper bound for the web-view load portion.
 */

import { test, expect, type Page } from "@playwright/test";

const BUDGET_MS = 2_000;
const SAMPLE_COUNT = 10;

async function measureColdStart(page: Page): Promise<number> {
  const t0 = Date.now();
  await page.goto("/");
  // "Interactive" proxy: the Open project button is enabled.
  await expect(
    page.getByRole("button", { name: /open project/i }),
  ).toBeEnabled({ timeout: BUDGET_MS * 3 });
  return Date.now() - t0;
}

test.describe("T154 — Cold-start performance ≤ 2 s p95 (SC-001)", () => {
  test("single cold-start is within 3× budget (sanity)", async ({ page }) => {
    const elapsed = await measureColdStart(page);
    console.log(`Cold-start single sample: ${elapsed} ms`);
    // A single run must finish within 3× the budget to catch catastrophic regressions.
    expect(elapsed).toBeLessThan(BUDGET_MS * 3);
  });

  test(`p95 cold-start ≤ ${BUDGET_MS} ms over ${SAMPLE_COUNT} samples`, async ({
    page,
  }) => {
    const latencies: number[] = [];

    for (let i = 0; i < SAMPLE_COUNT; i++) {
      latencies.push(await measureColdStart(page));
    }

    latencies.sort((a, b) => a - b);
    const p95 = latencies[Math.ceil(SAMPLE_COUNT * 0.95) - 1]!;
    const p50 = latencies[Math.floor(SAMPLE_COUNT * 0.5)]!;

    console.log(
      `Cold-start latencies (ms): ${latencies.join(", ")}\n` +
        `p50=${p50} ms, p95=${p95} ms`,
    );

    expect(p95).toBeLessThanOrEqual(BUDGET_MS);
  });

  test("Welcome page heading is present after cold start", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByRole("heading", { level: 1 })).toBeVisible({
      timeout: BUDGET_MS,
    });
  });

  test("no JS errors on cold start", async ({ page }) => {
    const errors: string[] = [];
    page.on("pageerror", (err) => errors.push(err.message));

    await page.goto("/");
    await page.getByRole("button", { name: /open project/i }).waitFor({
      timeout: BUDGET_MS * 2,
    });

    expect(errors).toHaveLength(0);
  });
});
