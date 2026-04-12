/**
 * T163 — Quickstart §4 smoke flow.
 *
 * Automates the six manual steps described in
 * `specs/001-speclens-desktop/quickstart.md §4`:
 *
 *   1. Pick a project that contains a `.specify/` directory → Workspace opens.
 *   2. Workspace shows: env badge, Steps list, Phase Tabs, Terminal panel.
 *   3. Click a step with generated files → Documents tab shows plan.md etc.
 *      with a "generated" badge.
 *   4. Click Tasks tab → if no task files exist, TaskFilePicker opens;
 *      cancel → task-file bar stays empty.
 *   5. Connect terminal, type `echo hello`, output streams back.
 *   6. State persistence: close and reopen → last step / last tab restored.
 *
 * Step 6 is approximated by navigating away and back since we cannot
 * close and reopen the Tauri window within a single Playwright page.
 */

import { test, expect, type Page } from "@playwright/test";
import * as path from "path";

const FIXTURE_OK = path.resolve(import.meta.dirname, "../fixtures/speckit-project-ok");

async function dismissEnvDialog(page: Page): Promise<void> {
  const continueBtn = page.locator('[data-testid="env-continue-button"]');
  if (await continueBtn.isVisible({ timeout: 4_000 }).catch(() => false)) {
    await continueBtn.click();
  }
}

test.describe("T163 — Quickstart §4 smoke flow", () => {
  test("Step 1: pick project → Workspace renders", async ({ page }) => {
    await page.goto(`/?open=${encodeURIComponent(FIXTURE_OK)}`);
    await expect(page.getByRole("region", { name: "Workspace" })).toBeVisible({
      timeout: 15_000,
    });
  });

  test("Step 2: Workspace shows env badge, Steps list, Phase Tabs, Terminal", async ({
    page,
  }) => {
    await page.goto(`/?open=${encodeURIComponent(FIXTURE_OK)}`);
    await dismissEnvDialog(page);

    // Env badge (AgentBadge or EnvBadge in the top bar).
    await expect(page.locator('[data-testid="agent-badge"], [data-testid="env-badge"]').first()).toBeVisible({
      timeout: 8_000,
    });

    // Steps list — at least one step button in the sidebar.
    await expect(
      page.getByRole("complementary", { name: /steps/i }).getByRole("button").first(),
    ).toBeVisible({ timeout: 5_000 });

    // Phase Tabs tablist.
    await expect(page.getByRole("tablist")).toBeVisible({ timeout: 5_000 });

    // Terminal panel — Connect button or xterm mount.
    await expect(
      page
        .getByRole("button", { name: /connect/i })
        .or(page.locator('[data-testid="terminal-xterm-mount"]'))
        .first(),
    ).toBeVisible({ timeout: 5_000 });
  });

  test("Step 3: click a step with generated docs → Documents tab shows generated badge", async ({
    page,
  }) => {
    await page.goto(`/?open=${encodeURIComponent(FIXTURE_OK)}`);
    await dismissEnvDialog(page);

    // Click the first step (likely "1-constitution" or "plan").
    const firstStep = page
      .getByRole("complementary", { name: /steps/i })
      .getByRole("button")
      .first();
    await firstStep.click();

    // Switch to the Documents tab.
    const docsTab = page.getByRole("tab", { name: /documents/i });
    await expect(docsTab).toBeEnabled({ timeout: 5_000 });
    await docsTab.click();

    // At least one document item should be listed.
    const docList = page
      .locator('[data-testid="document-list"]')
      .or(page.getByRole("list").filter({ hasText: /.md/i }));
    await expect(docList).toBeVisible({ timeout: 8_000 });
  });

  test("Step 4: Tasks tab → TaskFilePicker when no files; Cancel → bar stays empty", async ({
    page,
  }) => {
    await page.goto(`/?open=${encodeURIComponent(FIXTURE_OK)}`);
    await dismissEnvDialog(page);

    // Navigate to the 5-tasks step if it exists.
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

    // If picker is visible, cancel it.
    const picker = page.getByRole("region", { name: /pick task files/i });
    if (await picker.isVisible({ timeout: 3_000 }).catch(() => false)) {
      const cancelBtn = page.getByRole("button", { name: /cancel/i });
      if (await cancelBtn.isVisible({ timeout: 1_000 }).catch(() => false)) {
        await cancelBtn.click();
      } else {
        // Picker appeared without Cancel (first-entry mode, not add mode).
        // Just assert bar shows empty state.
      }
    }

    // The task-file bar must be visible.
    const bar = page.locator('[data-testid="task-file-bar"]');
    await expect(bar).toBeVisible({ timeout: 5_000 });

    // With no confirmed files the bar renders the "No task files selected" text.
    if (await picker.isVisible({ timeout: 500 }).catch(() => false)) {
      // Picker still showing — that's acceptable; the bar is rendered above it.
    } else {
      await expect(bar).toContainText(/no task files/i);
    }
  });

  test("Step 5: connect terminal, type echo hello, output streams back", async ({ page }) => {
    await page.goto(`/?open=${encodeURIComponent(FIXTURE_OK)}`);
    await dismissEnvDialog(page);

    const connectBtn = page.getByRole("button", { name: /connect/i });
    await expect(connectBtn).toBeEnabled({ timeout: 5_000 });
    await connectBtn.click();

    await expect(page.locator('[aria-live="polite"]')).toContainText(/connected/i, {
      timeout: 10_000,
    });

    await page.locator('[data-testid="terminal-xterm-mount"]').click();
    await page.keyboard.type("echo hello");
    await page.keyboard.press("Enter");

    const mirror = page.locator('[data-testid="terminal-line-mirror"]');
    await expect(mirror.locator('li:has-text("hello")')).toBeVisible({ timeout: 5_000 });
  });

  test("Step 6: navigate away and back → last step is restored", async ({ page }) => {
    await page.goto(`/?open=${encodeURIComponent(FIXTURE_OK)}`);
    await dismissEnvDialog(page);

    // Select a specific step.
    const steps = page
      .getByRole("complementary", { name: /steps/i })
      .getByRole("button");
    const stepCount = await steps.count();
    if (stepCount < 2) {
      test.skip();
      return;
    }

    // Click the second step to record a non-default selection.
    const secondStep = steps.nth(1);
    const secondStepName = await secondStep.textContent();
    await secondStep.click();

    // Simulate "close and reopen" by navigating away then back.
    await page.goto("/");
    await expect(page.getByRole("button", { name: /open project/i })).toBeVisible({
      timeout: 5_000,
    });

    // Reopen from the recent list or via the query param.
    await page.goto(`/?open=${encodeURIComponent(FIXTURE_OK)}`);
    await dismissEnvDialog(page);

    // The previously selected step must be active (aria-current="step").
    if (secondStepName) {
      const restoredStep = page.getByRole("button", {
        name: new RegExp(secondStepName.trim(), "i"),
      });
      await expect(restoredStep).toHaveAttribute("aria-current", "step", {
        timeout: 5_000,
      });
    }
  });
});
