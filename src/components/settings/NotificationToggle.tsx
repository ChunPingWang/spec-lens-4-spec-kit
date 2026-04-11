/**
 * Phase 9 US7 — `NotificationToggle` (T150 / FR-070).
 *
 * Two-state switch wired straight to `configStore.setNotificationsEnabled`.
 * Lives in the settings surface and (eventually) the system tray menu so
 * the user can flip OS notifications on/off without leaving SpecLens.
 */
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import { useConfigStore } from "@/stores";

export function NotificationToggle() {
  const { t } = useTranslation();
  const config = useConfigStore((s) => s.config);
  const load = useConfigStore((s) => s.load);
  const setNotificationsEnabled = useConfigStore((s) => s.setNotificationsEnabled);

  // The toggle is purely backend-backed, but we keep a local "saving"
  // flag so the user gets a disabled state during the round-trip rather
  // than an unresponsive switch.
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    if (!config) {
      void load();
    }
  }, [config, load]);

  const enabled = config?.notificationsEnabled ?? true;

  async function handleToggle(): Promise<void> {
    setSaving(true);
    try {
      await setNotificationsEnabled(!enabled);
    } finally {
      setSaving(false);
    }
  }

  return (
    <div
      data-testid="notification-toggle"
      data-enabled={enabled}
      className="flex items-center justify-between gap-3 rounded-md border border-border bg-card px-3 py-2"
    >
      <div className="flex flex-col">
        <span className="text-sm font-medium">{t("notifications.toggleLabel")}</span>
        <span className="text-xs text-muted-foreground">
          {t("notifications.toggleHint")}
        </span>
      </div>
      <button
        type="button"
        role="switch"
        aria-checked={enabled}
        aria-label={t("notifications.toggleLabel") ?? "Toggle notifications"}
        disabled={saving}
        onClick={() => {
          void handleToggle();
        }}
        className={`relative inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full transition-colors ${
          enabled ? "bg-primary" : "bg-muted"
        } ${saving ? "opacity-60" : ""}`}
      >
        <span
          className={`inline-block h-4 w-4 transform rounded-full bg-background shadow transition-transform ${
            enabled ? "translate-x-4" : "translate-x-0.5"
          }`}
        />
      </button>
    </div>
  );
}
