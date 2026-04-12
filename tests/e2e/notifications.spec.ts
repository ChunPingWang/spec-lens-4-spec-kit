/**
 * T147 — Native notification toggle scenarios.
 *
 * Covers:
 *   1. NotificationToggle renders with the correct initial state.
 *   2. Toggling from enabled → disabled reflects in the UI (aria-checked).
 *   3. Toggling from disabled → enabled reflects in the UI.
 *   4. When notifications are disabled, no OS notification is dispatched
 *      when a `steps_state_changed` event fires (FR-070 focus gate).
 *   5. When notifications are enabled and the window is NOT focused, a
 *      notification is expected to be dispatched.
 *
 * Note: OS-level notification delivery cannot be observed directly via
 * Playwright. Tests for items 4 & 5 rely on a stub counter exposed by
 * the IPC mock at `window.__speclensNotifyCount`.
 */

import { test, expect, type Page } from "@playwright/test";
import * as path from "path";

const FIXTURE_OK = path.resolve(import.meta.dirname, "../fixtures/speckit-project-ok");

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

async function openSettings(page: Page): Promise<void> {
  // Settings are reachable via a Settings button/link in the top bar or
  // a dedicated settings route.
  const settingsBtn = page
    .getByRole("button", { name: /settings/i })
    .or(page.getByRole("link", { name: /settings/i }))
    .first();
  if (await settingsBtn.isVisible({ timeout: 2_000 }).catch(() => false)) {
    await settingsBtn.click();
  } else {
    // Navigate directly if the button is not present.
    await page.goto("/settings");
  }
}

test.describe("T147 — Notification toggle: initial state", () => {
  test.beforeEach(async ({ page }) => {
    await openWorkspace(page);
    await openSettings(page);
  });

  test("NotificationToggle is visible and reflects backend config", async ({ page }) => {
    const toggle = page.locator('[data-testid="notification-toggle"]');
    await expect(toggle).toBeVisible({ timeout: 5_000 });

    // The switch element must carry aria-checked.
    const switchEl = toggle.getByRole("switch");
    await expect(switchEl).toBeVisible();
    const checked = await switchEl.getAttribute("aria-checked");
    expect(["true", "false"]).toContain(checked);
  });
});

test.describe("T147 — Notification toggle: enabled → disabled", () => {
  test("toggling off sets aria-checked to false and data-enabled to false", async ({
    page,
  }) => {
    await openWorkspace(page);
    await openSettings(page);

    const toggle = page.locator('[data-testid="notification-toggle"]');
    await expect(toggle).toBeVisible({ timeout: 5_000 });

    const switchEl = toggle.getByRole("switch");

    // Ensure enabled first.
    if ((await switchEl.getAttribute("aria-checked")) !== "true") {
      await switchEl.click();
      await expect(switchEl).toHaveAttribute("aria-checked", "true");
    }

    // Disable.
    await switchEl.click();

    await expect(switchEl).toHaveAttribute("aria-checked", "false", { timeout: 5_000 });
    await expect(toggle).toHaveAttribute("data-enabled", "false");
  });
});

test.describe("T147 — Notification toggle: disabled → enabled", () => {
  test("toggling on sets aria-checked to true and data-enabled to true", async ({ page }) => {
    await openWorkspace(page);
    await openSettings(page);

    const toggle = page.locator('[data-testid="notification-toggle"]');
    await expect(toggle).toBeVisible({ timeout: 5_000 });

    const switchEl = toggle.getByRole("switch");

    // Ensure disabled first.
    if ((await switchEl.getAttribute("aria-checked")) !== "false") {
      await switchEl.click();
      await expect(switchEl).toHaveAttribute("aria-checked", "false");
    }

    // Re-enable.
    await switchEl.click();
    await expect(switchEl).toHaveAttribute("aria-checked", "true", { timeout: 5_000 });
    await expect(toggle).toHaveAttribute("data-enabled", "true");
  });
});

test.describe("T147 — Notification toggle: focus-gating (FR-070)", () => {
  test("no notification dispatched when toggle is disabled", async ({ page }) => {
    await openWorkspace(page);
    await openSettings(page);

    const toggle = page.locator('[data-testid="notification-toggle"]');
    await expect(toggle).toBeVisible({ timeout: 5_000 });

    const switchEl = toggle.getByRole("switch");
    // Make sure disabled.
    if ((await switchEl.getAttribute("aria-checked")) !== "false") {
      await switchEl.click();
      await expect(switchEl).toHaveAttribute("aria-checked", "false");
    }

    // Reset the stub counter.
    await page.evaluate(() => {
      (window as unknown as Record<string, unknown>)["__speclensNotifyCount"] = 0;
    });

    // Simulate a steps_state_changed event via the IPC mock.
    await page.evaluate(() => {
      window.dispatchEvent(
        new CustomEvent("specklens:steps_state_changed", {
          detail: { stepId: "1-constitution", status: "done" },
        }),
      );
    });

    // Give the event handler time to fire (it should not trigger a notify call).
    await page.waitForTimeout(200);

    const count = await page.evaluate(
      () => (window as unknown as Record<string, unknown>)["__speclensNotifyCount"] ?? 0,
    );
    expect(count).toBe(0);
  });

  test("notification dispatched when toggle is enabled and window not focused", async ({
    page,
  }) => {
    await openWorkspace(page);
    await openSettings(page);

    const toggle = page.locator('[data-testid="notification-toggle"]');
    await expect(toggle).toBeVisible({ timeout: 5_000 });

    const switchEl = toggle.getByRole("switch");
    // Make sure enabled.
    if ((await switchEl.getAttribute("aria-checked")) !== "true") {
      await switchEl.click();
      await expect(switchEl).toHaveAttribute("aria-checked", "true");
    }

    // Reset stub counter.
    await page.evaluate(() => {
      (window as unknown as Record<string, unknown>)["__speclensNotifyCount"] = 0;
    });

    // Blur the window to simulate "not focused" (FR-070 gate).
    await page.evaluate(() => window.dispatchEvent(new Event("blur")));

    // Trigger a state-change event.
    await page.evaluate(() => {
      window.dispatchEvent(
        new CustomEvent("specklens:steps_state_changed", {
          detail: { stepId: "1-constitution", status: "done" },
        }),
      );
    });

    await page.waitForTimeout(300);

    const count = await page.evaluate(
      () => (window as unknown as Record<string, unknown>)["__speclensNotifyCount"] ?? 0,
    );
    // At least one notification call should have been made.
    expect(count).toBeGreaterThanOrEqual(1);
  });
});
