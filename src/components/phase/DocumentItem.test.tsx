/**
 * T069 — Vitest coverage for `DocumentItem`, asserting each DocStatus
 * badge renders with the right a11y label and that `missing` documents
 * are non-interactive. See `specs/001-speclens-desktop/spec.md §FR-027`.
 */
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import "@/i18n";

import { DocumentItem } from "./DocumentItem";
import type { DocStatus, PhaseDocument } from "@/types/ipc";

function makeDoc(status: DocStatus): PhaseDocument {
  return {
    relativePath: `specs/001-sample/${status}.md`,
    displayName: `${status}.md`,
    kind: "spec",
    hash: null,
    status,
    size: 128,
    modifiedAt: "2026-04-11T00:00:00Z",
  };
}

describe("DocumentItem", () => {
  it.each<[DocStatus, string]>([
    ["generated", "Generated"],
    ["modified", "Modified"],
    ["missing", "Missing"],
    ["unverified", "Unverified"],
  ])("renders the %s badge with its i18n label", (status, label) => {
    render(
      <DocumentItem document={makeDoc(status)} active={false} onSelect={() => {}} />,
    );
    expect(screen.getByLabelText(`status-${status}`)).toHaveTextContent(label);
  });

  it("disables selection for missing documents", async () => {
    const onSelect = vi.fn();
    render(<DocumentItem document={makeDoc("missing")} active={false} onSelect={onSelect} />);
    const button = screen.getByRole("button");
    expect(button).toBeDisabled();
    await userEvent.click(button);
    expect(onSelect).not.toHaveBeenCalled();
  });

  it("invokes onSelect with relativePath when clicked", async () => {
    const onSelect = vi.fn();
    const doc = makeDoc("generated");
    render(<DocumentItem document={doc} active={false} onSelect={onSelect} />);
    await userEvent.click(screen.getByRole("button"));
    expect(onSelect).toHaveBeenCalledWith(doc.relativePath);
  });

  it("exposes relativePath via the title tooltip", () => {
    const doc = makeDoc("modified");
    render(<DocumentItem document={doc} active={true} onSelect={() => {}} />);
    expect(screen.getByRole("button")).toHaveAttribute("title", doc.relativePath);
  });
});
