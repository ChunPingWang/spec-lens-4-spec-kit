import { useEffect, useState } from "react";

import { AppShell } from "@/components/layout/AppShell";
import { Welcome } from "@/pages/Welcome";
import { Workspace } from "@/pages/Workspace";
import { useProjectStore } from "@/stores";
import "@/i18n";

/**
 * Top-level router for a single window. A window either shows the Welcome page
 * (no project attached) or the Workspace (one project attached). Per FR-100,
 * opening a second project creates a new Tauri window rather than replacing
 * the current one, so this router intentionally supports only two states.
 */
export default function App() {
  const project = useProjectStore((s) => s.current);
  const loadRecent = useProjectStore((s) => s.loadRecent);
  const [ready, setReady] = useState(false);

  useEffect(() => {
    void loadRecent().finally(() => setReady(true));
  }, [loadRecent]);

  if (!ready) {
    return null;
  }

  return <AppShell>{project ? <Workspace /> : <Welcome />}</AppShell>;
}
