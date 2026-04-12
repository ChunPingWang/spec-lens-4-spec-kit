/**
 * T139 — Generic agent mode: all features remain available.
 *
 * When SpecLens cannot detect a specific AI agent (no .specify/
 * init-options.json, no recognised binary on PATH), it falls back to
 * "generic" mode. This spec asserts:
 *
 *   - AgentBadge renders with data-agent-id="generic" and
 *     data-detected-from="none".
 *   - All workspace panes (steps, phase tabs, terminal) are still usable.
 *   - No feature gate blocks the Tasks, Documents, or Terminal panels.
 *   - The badge label reads the localised "generic" source string.
 */

import { test, expect, type Page } from "@playwright/test";
import * as path from "path";

const FIXTURE_OK = path.resolve(import.meta.dirname, "../fixtures/speckit-project-ok");

async function openWorkspaceGenericAgent(page: Page): Promise<void> {
  // The `agent=none` query param tells the IPC mock to return a generic
  // profile (id:"generic", detectedFrom:"none").
  await page.goto(`/?open=${encodeURIComponent(FIXTURE_OK)}&agent=none`);
  await expect(page.getByRole("region", { name: "Workspace" })).toBeVisible({
    timeout: 15_000,
  });
  // Dismiss env dialog if present.
  const continueBtn = page.locator('[data-testid="env-continue-button"]');
  if (await continueBtn.isVisible({ timeout: 3_000 }).catch(() => false)) {
    await continueBtn.click();
  }
}

test.describe("T139 — Generic agent mode", () => {
  test.beforeEach(async ({ page }) => {
    await openWorkspaceGenericAgent(page);
  });

  test("AgentBadge shows generic agent with 'none' detected-from", async ({ page }) => {
    const badge = page.locator('[data-testid="agent-badge"]');
    await expect(badge).toBeVisible({ timeout: 5_000 });
    await expect(badge).toHaveAttribute("data-agent-id", "generic");
    await expect(badge).toHaveAttribute("data-detected-from", "none");
  });

  test("AgentBadge label contains the generic source string", async ({ page }) => {
    const badge = page.locator('[data-testid="agent-badge"]');
    // The subscript inside the badge renders the localised "agent.source.generic" key.
    await expect(badge).toContainText(/generic/i);
  });

  test("steps sidebar is accessible in generic mode", async ({ page }) => {
    const stepsAside = page.getByRole("complementary", { name: /steps/i });
    await expect(stepsAside).toBeVisible();
    // At least one step button must be rendered.
    await expect(stepsAside.getByRole("button").first()).toBeVisible();
  });

  test("PhaseTabs overview panel is accessible in generic mode", async ({ page }) => {
    const overviewTab = page.getByRole("tab", { name: /overview/i });
    await expect(overviewTab).toBeVisible();
    await overviewTab.click();
    // Overview panel renders step details.
    await expect(page.locator('[data-testid="overview-panel"]').or(
      page.getByRole("region", { name: /overview/i })
    )).toBeVisible({ timeout: 5_000 });
  });

  test("Documents tab is accessible in generic mode", async ({ page }) => {
    const docsTab = page.getByRole("tab", { name: /documents/i });
    await expect(docsTab).toBeEnabled();
    await docsTab.click();
    // Document list renders in the left column of the documents layout.
    await expect(page.locator('[data-testid="document-list"]').or(
      page.getByRole("list")
    )).toBeVisible({ timeout: 5_000 });
  });

  test("Terminal Connect button is accessible in generic mode", async ({ page }) => {
    const connectBtn = page.getByRole("button", { name: /connect/i });
    await expect(connectBtn).toBeVisible();
    await expect(connectBtn).toBeEnabled();
  });

  test("Tasks tab is unlocked when step is '5-tasks' in generic mode", async ({ page }) => {
    // Select the tasks step if present.
    const taskStepBtn = page.getByRole("button", { name: /5.*task/i }).first();
    if (await taskStepBtn.isVisible({ timeout: 3_000 }).catch(() => false)) {
      await taskStepBtn.click();
      const tasksTab = page.getByRole("tab", { name: /tasks/i });
      await expect(tasksTab).toBeEnabled({ timeout: 5_000 });
    } else {
      // Fixture has no tasks step — verify the tab is at least rendered.
      await expect(page.getByRole("tablist")).toBeVisible();
    }
  });

  test("no feature-gate overlay is shown in generic mode", async ({ page }) => {
    // There should be no "feature unavailable" or "agent required" banner.
    await expect(page.getByText(/feature unavailable/i)).not.toBeVisible();
    await expect(page.getByText(/agent required/i)).not.toBeVisible();
  });
});
