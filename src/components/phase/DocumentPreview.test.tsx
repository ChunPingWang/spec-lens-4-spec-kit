/**
 * T070 — Vitest coverage for `DocumentPreview`. US2 currently renders
 * Markdown via `marked`; JSON pretty-print + syntax-highlighted code
 * branches land with US4 task files. We lock in the Markdown path,
 * truncation notice, empty state, and error state here.
 */
import { render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import "@/i18n";

import { DocumentPreview } from "./DocumentPreview";
import { usePhaseStore, type PhaseDocumentReadResponse } from "@/stores/phaseStore";

const projectId = "00000000-0000-0000-0000-000000000001";

function primeCache(path: string, entry: PhaseDocumentReadResponse) {
  usePhaseStore.setState((prev) => ({
    documentCache: { ...prev.documentCache, [path]: entry },
  }));
}

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
    readDocument: vi.fn(async () => {
      throw new Error("readDocument not stubbed");
    }),
  });
}

describe("DocumentPreview", () => {
  beforeEach(() => {
    resetPhaseStore();
  });

  it("renders the empty-state hint when no document is selected", () => {
    render(<DocumentPreview projectId={projectId} relativePath={null} />);
    expect(
      screen.getByText(/Select a document on the left to preview it\./i),
    ).toBeInTheDocument();
  });

  it("renders Markdown content from the store cache", async () => {
    const path = "specs/001-sample/plan.md";
    primeCache(path, {
      relativePath: path,
      content: "# Heading One\n\nBody paragraph text.",
      truncated: false,
      hash: null,
    });
    render(<DocumentPreview projectId={projectId} relativePath={path} />);
    await waitFor(() => {
      expect(screen.getByRole("heading", { level: 1, name: "Heading One" })).toBeInTheDocument();
    });
    expect(screen.getByText("Body paragraph text.")).toBeInTheDocument();
  });

  it("shows the truncation notice when the response is flagged truncated", async () => {
    const path = "specs/001-sample/big.md";
    primeCache(path, {
      relativePath: path,
      content: "# Big doc\n",
      truncated: true,
      hash: null,
    });
    render(<DocumentPreview projectId={projectId} relativePath={path} />);
    await waitFor(() => {
      expect(screen.getByText(/Preview truncated/i)).toBeInTheDocument();
    });
  });

  it("calls readDocument when the selected path is not cached", async () => {
    const path = "specs/001-sample/research.md";
    const response: PhaseDocumentReadResponse = {
      relativePath: path,
      content: "## Research\n\nFindings here.",
      truncated: false,
      hash: null,
    };
    const readDocument = vi.fn(async () => response);
    usePhaseStore.setState({ readDocument });
    render(<DocumentPreview projectId={projectId} relativePath={path} />);
    await waitFor(() => {
      expect(readDocument).toHaveBeenCalledWith(projectId, path);
    });
  });

  it("surfaces read errors inline", async () => {
    const readDocument = vi.fn(async () => {
      throw new Error("boom");
    });
    usePhaseStore.setState({ readDocument });
    render(<DocumentPreview projectId={projectId} relativePath="specs/001-sample/gone.md" />);
    await waitFor(() => {
      expect(screen.getByText("boom")).toBeInTheDocument();
    });
  });
});
