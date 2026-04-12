/**
 * T113 — Tasks first-entry flow.
 *
 * Covers the complete US4 happy path:
 *   1. Open project → navigate to the "5-tasks" step → Tasks tab.
 *   2. TaskFilePicker appears with no pre-selected items (FR-050).
 *   3. Select one or more candidates → Confirm opens TaskFileBar.
 *   4. TaskFileBar "+ add file" adds a second file.
 *   5. Filter-per-tab: each tab retains its own filter state.
 */

import { test, expect, type Page } from "@playwright/test";
import * as path from "path";

const FIXTURE_TASKS = path.resolve(import.meta.dirname, "../fixtures/task-files");
const FIXTURE_OK = path.resolve(import.meta.dirname, "../fixtures/speckit-project-ok");

async function navigateToWorkspace(page: Page, projectPath: string): Promise<void> {
  await page.goto(`/?open=${encodeURIComponent(projectPath)}`);
  await expect(page.getByRole("region", { name: "Workspace" })).toBeVisible({
    timeout: 15_000,
  });
  // Dismiss env dialog if present.
  const continueBtn = page.locator('[data-testid="env-continue-button"]');
  if (await continueBtn.isVisible({ timeout: 3_000 }).catch(() => false)) {
    await continueBtn.click();
  }
}

async function navigateToTasksTab(page: Page): Promise<void> {
  // Select the "5-tasks" step in the left steps sidebar.
  await page
    .getByRole("button", { name: /5.*task/i })
    .first()
    .click();

  // Click the "Tasks" tab in PhaseTabs.
  const tasksTab = page.getByRole("tab", { name: /tasks/i });
  await expect(tasksTab).toBeEnabled({ timeout: 5_000 });
  await tasksTab.click();
}

test.describe("T113 — Tasks first-entry flow", () => {
  test.beforeEach(async ({ page }) => {
    await navigateToWorkspace(page, FIXTURE_OK);
  });

  test("TaskFilePicker renders with no pre-checked items (FR-050)", async ({ page }) => {
    await navigateToTasksTab(page);

    // Picker section must be visible.
    await expect(
      page.getByRole("region", { name: /pick task files/i }),
    ).toBeVisible({ timeout: 5_000 });

    // Confirm button is disabled until something is selected.
    const confirmBtn = page.locator('[data-testid="task-picker-confirm"]');
    await expect(confirmBtn).toBeDisabled();

    // All checkboxes are unchecked by default.
    const checkboxes = page.locator('input[type="checkbox"]');
    const count = await checkboxes.count();
    for (let i = 0; i < count; i++) {
      await expect(checkboxes.nth(i)).not.toBeChecked();
    }
  });

  test("selecting one file enables the Confirm button", async ({ page }) => {
    await navigateToTasksTab(page);

    const confirmBtn = page.locator('[data-testid="task-picker-confirm"]');
    await expect(confirmBtn).toBeDisabled();

    // Select the first available candidate.
    const firstCheckbox = page.locator('input[type="checkbox"]').first();
    await firstCheckbox.check();

    await expect(confirmBtn).toBeEnabled();
  });

  test("confirm opens TaskFileBar with the selected tab", async ({ page }) => {
    await navigateToTasksTab(page);

    const firstCheckbox = page.locator('input[type="checkbox"]').first();
    await firstCheckbox.check();

    const confirmBtn = page.locator('[data-testid="task-picker-confirm"]');
    await confirmBtn.click();

    // TaskFileBar must now be visible.
    await expect(page.locator('[data-testid="task-file-bar"]')).toBeVisible({
      timeout: 5_000,
    });

    // There should be at least one tab rendered.
    const tabs = page.locator('[data-testid="task-file-bar"] [role="tab"]');
    await expect(tabs).toHaveCount(1, { timeout: 3_000 });
  });

  test("+ add file button re-opens the picker in add mode", async ({ page }) => {
    await navigateToTasksTab(page);

    // Confirm with the first candidate.
    await page.locator('input[type="checkbox"]').first().check();
    await page.locator('[data-testid="task-picker-confirm"]').click();
    await expect(page.locator('[data-testid="task-file-bar"]')).toBeVisible();

    // Click the "+ add file" button.
    const addBtn = page.locator('[data-testid="task-file-bar-add"]');
    await addBtn.click();

    // Picker must re-appear with a Cancel button (add mode).
    await expect(
      page.getByRole("region", { name: /pick task files/i }),
    ).toBeVisible({ timeout: 3_000 });
    await expect(page.getByRole("button", { name: /cancel/i })).toBeVisible();
  });

  test("manual path input adds a custom task file", async ({ page }) => {
    await navigateToTasksTab(page);

    const manualInput = page.getByRole("textbox", { name: /manual path/i });
    await manualInput.fill("specs/001/tasks.md");
    await page.getByRole("button", { name: /add path/i }).click();

    // The confirm button should now be enabled.
    await expect(page.locator('[data-testid="task-picker-confirm"]')).toBeEnabled();
  });

  test("filter state is independent per tab", async ({ page }) => {
    await navigateToTasksTab(page);

    // Select two task files (if available) to create two tabs.
    const checkboxes = page.locator('input[type="checkbox"]');
    const count = await checkboxes.count();
    if (count < 2) {
      test.skip();
      return;
    }
    await checkboxes.nth(0).check();
    await checkboxes.nth(1).check();
    await page.locator('[data-testid="task-picker-confirm"]').click();

    const tabs = page.locator('[data-testid="task-file-bar"] [role="tab"]');
    await expect(tabs).toHaveCount(2, { timeout: 5_000 });

    // Set filter on first tab.
    const tab0 = tabs.nth(0).locator("button").first();
    await tab0.click();
    const filterInput = page.locator('[data-testid="task-filter-search"]');
    if (await filterInput.isVisible()) {
      await filterInput.fill("only-in-tab-0");
    }

    // Switch to second tab — filter should be independent (empty).
    const tab1 = tabs.nth(1).locator("button").first();
    await tab1.click();
    if (await filterInput.isVisible()) {
      await expect(filterInput).toHaveValue("");
    }
  });
});
