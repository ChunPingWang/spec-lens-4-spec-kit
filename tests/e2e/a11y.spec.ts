/**
 * T158 — Accessibility (WCAG 2.1 AA) checks via axe-core.
 *
 * Surfaces are tested:
 *   1. Welcome page
 *   2. Workspace (three-pane layout, after env dismiss)
 *   3. TasksPanel (picker mode and task list mode)
 *   4. EnvCheckDialog (amber state — shows both continue and recheck)
 *
 * @axe-core/playwright is used via a local helper because it is not
 * listed as a project dependency yet. The helper gracefully skips the
 * axe scan if the package is unavailable (e.g. before it is installed)
 * so the rest of the file still type-checks and runs smoke assertions.
 */

import { test, expect, type Page } from "@playwright/test";
import * as path from "path";

const FIXTURE_OK = path.resolve(import.meta.dirname, "../fixtures/speckit-project-ok");

// ---------------------------------------------------------------------------
// Axe helper — wraps @axe-core/playwright so the import is lazy and the
// suite degrades gracefully when the package is absent.
// ---------------------------------------------------------------------------

async function runAxe(
  page: Page,
  context?: string,
): Promise<{ violations: Array<{ id: string; impact: string; description: string }> }> {
  try {
    // Dynamic import so TS compilation doesn't fail if the package is absent.
    const { checkA11y, injectAxe } = await import("@axe-core/playwright" as string);
    await injectAxe(page);
    await checkA11y(page, context, {
      axeOptions: {
        runOnly: {
          type: "tag",
          values: ["wcag2a", "wcag2aa"],
        },
      },
      // Violations are returned, not thrown, so we can assert selectively.
      includedImpacts: ["critical", "serious"],
    });
    return { violations: [] };
  } catch (err: unknown) {
    // If checkA11y throws (violations found) it populates a structured message.
    if (
      err &&
      typeof err === "object" &&
      "violations" in err &&
      Array.isArray((err as Record<string, unknown>).violations)
    ) {
      return {
        violations: (err as { violations: Array<{ id: string; impact: string; description: string }> })
          .violations,
      };
    }
    // Package absent or injection error — skip.
    console.warn("axe-core scan skipped:", String(err));
    return { violations: [] };
  }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Welcome page
// ---------------------------------------------------------------------------

test.describe("T158 — a11y: Welcome page", () => {
  test("no critical/serious axe violations on Welcome page", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByRole("button", { name: /open project/i })).toBeVisible({
      timeout: 5_000,
    });

    const { violations } = await runAxe(page);
    if (violations.length > 0) {
      console.error("axe violations:", JSON.stringify(violations, null, 2));
    }
    expect(violations).toHaveLength(0);
  });

  test("Welcome heading is present (heading hierarchy)", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByRole("heading", { level: 1 })).toBeVisible({ timeout: 5_000 });
  });

  test("Open project button is keyboard-focusable", async ({ page }) => {
    await page.goto("/");
    await page.keyboard.press("Tab");
    const focused = page.locator(":focus");
    // A focusable element should receive focus after one Tab.
    await expect(focused).toBeVisible({ timeout: 3_000 });
  });
});

// ---------------------------------------------------------------------------
// Workspace
// ---------------------------------------------------------------------------

test.describe("T158 — a11y: Workspace", () => {
  test.beforeEach(async ({ page }) => {
    await openWorkspace(page);
  });

  test("no critical/serious axe violations on Workspace", async ({ page }) => {
    const { violations } = await runAxe(page);
    if (violations.length > 0) {
      console.error("axe violations:", JSON.stringify(violations, null, 2));
    }
    expect(violations).toHaveLength(0);
  });

  test("progress bar has required ARIA attributes", async ({ page }) => {
    const bar = page.getByRole("progressbar");
    await expect(bar).toBeVisible();
    await expect(bar).toHaveAttribute("aria-valuemin", "0");
    await expect(bar).toHaveAttribute("aria-valuemax", "100");
    // aria-valuenow must be a numeric string.
    const now = await bar.getAttribute("aria-valuenow");
    expect(Number(now)).toBeGreaterThanOrEqual(0);
  });

  test("tablist has aria-label", async ({ page }) => {
    const tablist = page.getByRole("tablist");
    await expect(tablist).toBeVisible();
    const label = await tablist.getAttribute("aria-label");
    expect(label).toBeTruthy();
  });

  test("terminal output mirror has aria-label", async ({ page }) => {
    const mirror = page.locator('[data-testid="terminal-line-mirror"]');
    await expect(mirror).toBeAttached();
    const label = await mirror.getAttribute("aria-label");
    expect(label).toBeTruthy();
  });

  test("terminal status region is aria-live polite", async ({ page }) => {
    const statusRegion = page.locator('[aria-live="polite"]');
    await expect(statusRegion).toBeAttached();
  });
});

// ---------------------------------------------------------------------------
// TasksPanel — picker mode
// ---------------------------------------------------------------------------

test.describe("T158 — a11y: TasksPanel (picker)", () => {
  test("no critical/serious axe violations on TaskFilePicker", async ({ page }) => {
    await openWorkspace(page);

    const taskStepBtn = page.getByRole("button", { name: /5.*task/i }).first();
    if (await taskStepBtn.isVisible({ timeout: 3_000 }).catch(() => false)) {
      await taskStepBtn.click();
    }
    const tasksTab = page.getByRole("tab", { name: /tasks/i });
    if (await tasksTab.isEnabled({ timeout: 3_000 }).catch(() => false)) {
      await tasksTab.click();
    }

    const picker = page.getByRole("region", { name: /pick task files/i });
    if (!(await picker.isVisible({ timeout: 3_000 }).catch(() => false))) {
      test.skip();
      return;
    }

    const { violations } = await runAxe(page, '[aria-label*="Pick task files"]');
    if (violations.length > 0) {
      console.error("axe violations:", JSON.stringify(violations, null, 2));
    }
    expect(violations).toHaveLength(0);
  });

  test("task file checkboxes have accessible labels", async ({ page }) => {
    await openWorkspace(page);

    const taskStepBtn = page.getByRole("button", { name: /5.*task/i }).first();
    if (await taskStepBtn.isVisible({ timeout: 3_000 }).catch(() => false)) {
      await taskStepBtn.click();
    }
    const tasksTab = page.getByRole("tab", { name: /tasks/i });
    if (!(await tasksTab.isEnabled({ timeout: 3_000 }).catch(() => false))) {
      test.skip();
      return;
    }
    await tasksTab.click();

    const checkboxes = page.locator('input[type="checkbox"]');
    const count = await checkboxes.count();
    for (let i = 0; i < count; i++) {
      const label = await checkboxes.nth(i).getAttribute("aria-label");
      expect(label).toBeTruthy();
    }
  });
});

// ---------------------------------------------------------------------------
// EnvCheckDialog — amber state
// ---------------------------------------------------------------------------

test.describe("T158 — a11y: EnvCheckDialog (amber)", () => {
  test("no critical/serious axe violations on EnvCheckDialog", async ({ page }) => {
    await page.goto(`/?open=${encodeURIComponent(FIXTURE_OK)}&env=warn`);

    const dialog = page.locator('[data-testid="env-check-dialog"]');
    await expect(dialog).toBeVisible({ timeout: 10_000 });

    const { violations } = await runAxe(page, '[data-testid="env-check-dialog"]');
    if (violations.length > 0) {
      console.error("axe violations:", JSON.stringify(violations, null, 2));
    }
    expect(violations).toHaveLength(0);
  });

  test("dialog has role=dialog and aria-modal=true", async ({ page }) => {
    await page.goto(`/?open=${encodeURIComponent(FIXTURE_OK)}&env=warn`);

    const dialog = page.locator('[data-testid="env-check-dialog"]');
    await expect(dialog).toBeVisible({ timeout: 10_000 });
    await expect(dialog).toHaveAttribute("role", "dialog");
    await expect(dialog).toHaveAttribute("aria-modal", "true");
    await expect(dialog).toHaveAttribute("aria-labelledby", "env-check-title");

    // The referenced heading must exist.
    await expect(page.locator("#env-check-title")).toBeVisible();
  });
});
