import type { PropsWithChildren } from "react";
import { useTranslation } from "react-i18next";

export function TopBar({ children }: PropsWithChildren) {
  const { t } = useTranslation();
  return (
    <header
      role="banner"
      className="flex h-12 shrink-0 items-center justify-between border-b border-border bg-card px-4"
    >
      <div className="flex items-center gap-2">
        <span className="text-sm font-semibold tracking-tight">{t("app.name")}</span>
        <span className="text-xs text-muted-foreground">{t("app.tagline")}</span>
      </div>
      <div className="flex items-center gap-2">{children}</div>
    </header>
  );
}
