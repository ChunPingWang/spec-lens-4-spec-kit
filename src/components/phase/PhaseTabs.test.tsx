/**
 * T068 — Vitest coverage for `PhaseTabs`, asserting the Tasks sub-page
 * is only enabled when the selected step is `5-tasks` (FR-030 / FR-032).
 */
import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import "@/i18n";

import { PhaseTabs } from "./PhaseTabs";
import { usePhaseStore } from "@/stores";

vi.mock("./DocumentList", () => ({
  DocumentList: () => <div data-testid="document-list" />,
}));
vi.mock("./DocumentPreview", () => ({
  DocumentPreview: () => <div data-testid="document-preview" />,
}));
vi.mock("./OverviewPanel", () => ({
  OverviewPanel: () => <div data-testid="overview-panel" />,
}));

const projectId = "00000000-0000-0000-0000-000000000001";

function resetPhaseStore() {
  usePhaseStore.setState({
    activeStepId: null,
    activeTab: "overview",
    documents: [],
    overview: null,
    documentCache: {},
    loadingDocuments: false,
    loadingOverview: false,
    error: null,
  });
}

describe("PhaseTabs", () => {
  beforeEach(() => {
    resetPhaseStore();
    // loadDocuments triggers an IPC call on mount; stub it out so nothing
    // actually invokes the backend during the render phase.
    usePhaseStore.setState({
      loadDocuments: vi.fn(async () => {}),
    });
  });

  it("disables the Tasks tab for non-tasks steps", () => {
    render(<PhaseTabs projectId={projectId} stepId="4-plan" />);
    const tasksTab = screen.getByRole("tab", { name: /tasks/i });
    expect(tasksTab).toBeDisabled();
  });

  it("enables the Tasks tab when the step is 5-tasks", () => {
    render(<PhaseTabs projectId={projectId} stepId="5-tasks" />);
    const tasksTab = screen.getByRole("tab", { name: /tasks/i });
    expect(tasksTab).not.toBeDisabled();
  });

  it("renders the overview panel by default", () => {
    render(<PhaseTabs projectId={projectId} stepId="4-plan" />);
    expect(screen.getByTestId("overview-panel")).toBeInTheDocument();
  });

  it("exposes three tabs via role=tab", () => {
    render(<PhaseTabs projectId={projectId} stepId="4-plan" />);
    expect(screen.getAllByRole("tab")).toHaveLength(3);
  });
});
