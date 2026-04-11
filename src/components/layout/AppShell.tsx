import type { PropsWithChildren } from "react";

import { TopBar } from "./TopBar";
import { ErrorToast } from "@/components/ui/ErrorToast";
import { useProjectStore } from "@/stores";

interface AppShellProps {
  headerSlot?: React.ReactNode;
}

/**
 * Top-level layout: fixed top bar, scrollable content area, global error
 * toast. Reads the active project from the store so the `TopBar` can show
 * project metadata without extra prop drilling.
 */
export function AppShell({ children, headerSlot }: PropsWithChildren<AppShellProps>) {
  const project = useProjectStore((s) => s.current);
  const headerProject = project
    ? {
        name: project.name,
        path: project.rootPath,
        lastOpenedAt: project.state.lastOpenedAt,
        agent: project.agent,
      }
    : undefined;
  return (
    <div className="flex min-h-screen flex-col bg-background text-foreground">
      <TopBar project={headerProject}>{headerSlot}</TopBar>
      <main className="flex flex-1 flex-col overflow-hidden">{children}</main>
      <ErrorToast />
    </div>
  );
}
