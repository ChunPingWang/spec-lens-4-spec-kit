/**
 * T138 — Vitest coverage for `AgentBadge`. Locks in FR-060 / FR-061 /
 * FR-087:
 *   - canonical brand display name (untranslated);
 *   - per-`detectedFrom` localized subscript label;
 *   - matching `AgentIcon` rendered alongside the badge.
 */
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import "@/i18n";

import { AgentBadge } from "./AgentBadge";
import type { AgentProfile } from "@/types/ipc";

function makeAgent(overrides: Partial<AgentProfile> = {}): AgentProfile {
  return {
    id: "claude-code",
    displayName: "Claude Code",
    iconKey: "agent-claude-code",
    detectedFrom: "project",
    highlightRuleSet: "universal",
    ...overrides,
  };
}

describe("AgentBadge", () => {
  it("renders the canonical brand name and the configured label for project source", () => {
    render(<AgentBadge agent={makeAgent({ detectedFrom: "project" })} />);
    const badge = screen.getByTestId("agent-badge");
    expect(badge).toHaveAttribute("data-agent-id", "claude-code");
    expect(badge).toHaveAttribute("data-detected-from", "project");
    expect(screen.getByText("Claude Code")).toBeInTheDocument();
    expect(screen.getByText(/configured/i)).toBeInTheDocument();
    expect(screen.getByTestId("agent-icon-claude-code")).toBeInTheDocument();
  });

  it("renders the auto-detected label when detection comes from a fingerprint", () => {
    render(
      <AgentBadge
        agent={makeAgent({
          id: "cursor",
          displayName: "Cursor",
          iconKey: "agent-cursor",
          detectedFrom: "path",
        })}
      />,
    );
    const badge = screen.getByTestId("agent-badge");
    expect(badge).toHaveAttribute("data-agent-id", "cursor");
    expect(badge).toHaveAttribute("data-detected-from", "path");
    expect(screen.getByText("Cursor")).toBeInTheDocument();
    expect(screen.getByText(/auto-detected/i)).toBeInTheDocument();
    expect(screen.getByTestId("agent-icon-cursor")).toBeInTheDocument();
  });

  it("renders the generic mode label and icon when nothing was detected", () => {
    render(
      <AgentBadge
        agent={makeAgent({
          id: "generic",
          displayName: "Generic",
          iconKey: "agent-generic",
          detectedFrom: "none",
        })}
      />,
    );
    const badge = screen.getByTestId("agent-badge");
    expect(badge).toHaveAttribute("data-agent-id", "generic");
    expect(badge).toHaveAttribute("data-detected-from", "none");
    expect(screen.getByText("Generic")).toBeInTheDocument();
    expect(screen.getByText(/generic mode/i)).toBeInTheDocument();
    expect(screen.getByTestId("agent-icon-generic")).toBeInTheDocument();
  });
});
