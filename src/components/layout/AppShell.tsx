import type { PropsWithChildren } from "react";

import { TopBar } from "./TopBar";
import { ErrorToast } from "@/components/ui/ErrorToast";

interface AppShellProps {
  headerSlot?: React.ReactNode;
}

/**
 * Top-level layout: fixed top bar, scrollable content area, global error
 * toast. Keeps the rest of the app agnostic of header/footer chrome.
 */
export function AppShell({ children, headerSlot }: PropsWithChildren<AppShellProps>) {
  return (
    <div className="flex min-h-screen flex-col bg-background text-foreground">
      <TopBar>{headerSlot}</TopBar>
      <main className="flex flex-1 flex-col overflow-hidden">{children}</main>
      <ErrorToast />
    </div>
  );
}
