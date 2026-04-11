import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import { DocumentList } from "./DocumentList";
import { DocumentPreview } from "./DocumentPreview";
import { OverviewPanel } from "./OverviewPanel";
import { usePhaseStore } from "@/stores";
import type { PhaseTab, Uuid } from "@/types/ipc";

interface PhaseTabsProps {
  projectId: Uuid;
  stepId: string;
}

const TABS: PhaseTab[] = ["overview", "documents", "tasks"];

/**
 * Phase detail container: switches between Overview, Documents, Tasks.
 * Tasks sub-page is only enabled for the `5-tasks` step (FR-030/FR-032).
 */
export function PhaseTabs({ projectId, stepId }: PhaseTabsProps) {
  const { t } = useTranslation();
  const activeTab = usePhaseStore((s) => s.activeTab);
  const setActiveTab = usePhaseStore((s) => s.setActiveTab);
  const documents = usePhaseStore((s) => s.documents);
  const loadDocuments = usePhaseStore((s) => s.loadDocuments);
  const [activeDocPath, setActiveDocPath] = useState<string | null>(null);

  // When the selected step changes, reload its documents and reset preview.
  useEffect(() => {
    void loadDocuments(projectId, stepId);
    setActiveDocPath(null);
  }, [projectId, stepId, loadDocuments]);

  const tasksLocked = stepId !== "5-tasks";

  return (
    <div className="flex h-full flex-col">
      <div
        role="tablist"
        aria-label={t("phase.tabsLabel", { defaultValue: "Phase sub-pages" })}
        className="flex items-center gap-1 border-b border-border px-2"
      >
        {TABS.map((tab) => {
          const disabled = tab === "tasks" && tasksLocked;
          return (
            <button
              key={tab}
              type="button"
              role="tab"
              aria-selected={activeTab === tab}
              disabled={disabled}
              onClick={() => setActiveTab(tab)}
              className={`border-b-2 px-3 py-2 text-xs font-medium capitalize transition-colors ${
                activeTab === tab
                  ? "border-primary text-foreground"
                  : "border-transparent text-muted-foreground hover:text-foreground"
              } disabled:cursor-not-allowed disabled:opacity-40`}
            >
              {t(`phase.tabs.${tab}`, { defaultValue: tab })}
            </button>
          );
        })}
      </div>

      <div className="flex-1 overflow-auto">
        {activeTab === "overview" && <OverviewPanel projectId={projectId} stepId={stepId} />}
        {activeTab === "documents" && (
          <div className="grid h-full grid-cols-[260px_1fr]">
            <div className="overflow-auto border-r border-border p-2">
              <DocumentList
                documents={documents}
                activePath={activeDocPath}
                onSelect={setActiveDocPath}
              />
            </div>
            <div className="overflow-auto">
              <DocumentPreview projectId={projectId} relativePath={activeDocPath} />
            </div>
          </div>
        )}
        {activeTab === "tasks" && !tasksLocked && (
          <p className="p-4 text-sm text-muted-foreground">
            {t("phase.tasksPlaceholder", {
              defaultValue: "Task file picker lands in Phase 6 (US4).",
            })}
          </p>
        )}
      </div>
    </div>
  );
}
