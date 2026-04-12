/**
 * T129 — Env-check dialog scenarios.
 *
 * Three environment states are exercised:
 *
 *   1. Healthy (green) — dialog auto-dismisses; workspace is accessible.
 *   2. Warning (amber) — "Continue anyway" button is shown; workspace
 *      becomes accessible after acknowledgement; UpdateBanner persists.
 *   3. Error (red) — dialog is modal and blocks the workspace; no
 *      dismiss button; only Re-check is offered (FR-012).
 *
 * The test harness injects the environment status via query params that
 * the dev server recognises and forwards to the IPC mock layer.
 */

import { test, expect, type Page } from "@playwright/test";
import * as path from "path";

const FIXTURE_OK = path.resolve(import.meta.dirname, "../fixtures/speckit-project-ok");

async function openProjectWithEnvState(
  page: Page,
  projectPath: string,
  envState: "ok" | "warn" | "error",
): Promise<void> {
  await page.goto(
    `/?open=${encodeURIComponent(projectPath)}&env=${envState}`,
  );
  await page.waitForLoadState("domcontentloaded");
}

test.describe("T129 — Env-check dialog: healthy environment", () => {
  test("green badge: dialog auto-dismisses and workspace is shown", async ({ page }) => {
    await openProjectWithEnvState(page, FIXTURE_OK, "ok");

    // Dialog should not be present (green auto-acknowledges).
    const dialog = page.locator('[data-testid="env-check-dialog"]');
    await expect(dialog).not.toBeVisible({ timeout: 8_000 });

    // Three-pane workspace must be accessible.
    await expect(page.getByRole("region", { name: "Workspace" })).toBeVisible({
      timeout: 10_000,
    });
  });

  test("green badge is not shown in workspace once acknowledged", async ({ page }) => {
    await openProjectWithEnvState(page, FIXTURE_OK, "ok");
    // EnvCheckDialog must be gone.
    await expect(page.locator('[data-testid="env-check-dialog"]')).not.toBeVisible({
      timeout: 8_000,
    });
    // UpdateBanner must not appear for a healthy env.
    await expect(page.locator('[data-testid="update-banner"]')).not.toBeVisible();
  });
});

test.describe("T129 — Env-check dialog: warning (amber) environment", () => {
  test("amber badge renders 'Continue anyway' and does not block on click", async ({
    page,
  }) => {
    await openProjectWithEnvState(page, FIXTURE_OK, "warn");

    const dialog = page.locator('[data-testid="env-check-dialog"]');
    await expect(dialog).toBeVisible({ timeout: 10_000 });

    // Badge colour must be amber.
    await expect(page.locator('[data-testid="env-badge"]')).toHaveAttribute(
      "data-color",
      "amber",
    );

    // "Continue anyway" must be present.
    const continueBtn = page.locator('[data-testid="env-continue-button"]');
    await expect(continueBtn).toBeVisible();
    await expect(continueBtn).toContainText(/continue anyway/i);

    await continueBtn.click();

    // After acknowledgement the dialog closes.
    await expect(dialog).not.toBeVisible({ timeout: 5_000 });

    // Workspace is now usable.
    await expect(page.getByRole("region", { name: "Workspace" })).toBeVisible();
  });

  test("amber: UpdateBanner persists after acknowledgement", async ({ page }) => {
    await openProjectWithEnvState(page, FIXTURE_OK, "warn");

    const continueBtn = page.locator('[data-testid="env-continue-button"]');
    await expect(continueBtn).toBeVisible({ timeout: 8_000 });
    await continueBtn.click();

    // A persistent amber banner must remain visible.
    await expect(page.locator('[data-testid="update-banner"]')).toBeVisible({
      timeout: 5_000,
    });
  });

  test("amber: Re-check button is available", async ({ page }) => {
    await openProjectWithEnvState(page, FIXTURE_OK, "warn");

    const dialog = page.locator('[data-testid="env-check-dialog"]');
    await expect(dialog).toBeVisible({ timeout: 8_000 });

    await expect(page.getByRole("button", { name: /re-?check/i })).toBeVisible();
  });
});

test.describe("T129 — Env-check dialog: error (red) environment (FR-012)", () => {
  test("red badge renders a modal that cannot be dismissed", async ({ page }) => {
    await openProjectWithEnvState(page, FIXTURE_OK, "error");

    const dialog = page.locator('[data-testid="env-check-dialog"]');
    await expect(dialog).toBeVisible({ timeout: 10_000 });
    await expect(dialog).toHaveAttribute("aria-modal", "true");

    // Badge must be red.
    await expect(page.locator('[data-testid="env-badge"]')).toHaveAttribute(
      "data-color",
      "red",
    );
  });

  test("red badge: no 'Continue' button — only Re-check", async ({ page }) => {
    await openProjectWithEnvState(page, FIXTURE_OK, "error");

    const dialog = page.locator('[data-testid="env-check-dialog"]');
    await expect(dialog).toBeVisible({ timeout: 10_000 });

    // "Continue" / "Continue anyway" must NOT be present for red state.
    await expect(page.locator('[data-testid="env-continue-button"]')).not.toBeVisible();

    // Re-check button must be present.
    await expect(page.getByRole("button", { name: /re-?check/i })).toBeVisible();
  });

  test("red badge: install guide is shown", async ({ page }) => {
    await openProjectWithEnvState(page, FIXTURE_OK, "error");

    const dialog = page.locator('[data-testid="env-check-dialog"]');
    await expect(dialog).toBeVisible({ timeout: 10_000 });

    // InstallGuideView content should be visible (or loading indicator).
    const guideOrLoader = page
      .locator('[data-testid="install-guide"]')
      .or(page.getByText(/loading guide/i));
    await expect(guideOrLoader).toBeVisible({ timeout: 8_000 });
  });

  test("red badge blocks workspace behind backdrop", async ({ page }) => {
    await openProjectWithEnvState(page, FIXTURE_OK, "error");

    const dialog = page.locator('[data-testid="env-check-dialog"]');
    await expect(dialog).toBeVisible({ timeout: 10_000 });

    // The workspace section should be obscured — clicking outside the
    // dialog must not close it.
    await page.keyboard.press("Escape");
    await expect(dialog).toBeVisible(); // still visible
  });
});
