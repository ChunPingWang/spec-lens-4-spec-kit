/**
 * T100 — Vitest coverage for `QuickCommands`. The component is gated behind
 * the `allowTerminalForwarding` setting (FR-044) and disabled by default.
 * When enabled it renders quick-command buttons for common Spec-Kit CLI
 * invocations that forward text to the PTY via `onSend`.
 */
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import "@/i18n";

import { QuickCommands } from "./QuickCommands";

const onSend = vi.fn();

describe("QuickCommands", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("renders nothing when allowTerminalForwarding is false", () => {
    const { container } = render(
      <QuickCommands allowTerminalForwarding={false} connected={true} onSend={onSend} />,
    );
    expect(container.innerHTML).toBe("");
  });

  it("renders quick-command buttons when enabled and connected", () => {
    render(<QuickCommands allowTerminalForwarding={true} connected={true} onSend={onSend} />);
    expect(screen.getByRole("button", { name: /spec-kit status/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /spec-kit run/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /spec-kit validate/i })).toBeInTheDocument();
  });

  it("sends the correct command when a button is clicked", async () => {
    const user = userEvent.setup();
    render(<QuickCommands allowTerminalForwarding={true} connected={true} onSend={onSend} />);

    await user.click(screen.getByRole("button", { name: /spec-kit status/i }));
    expect(onSend).toHaveBeenCalledWith("spec-kit status\n");
  });

  it("disables buttons when not connected", () => {
    render(<QuickCommands allowTerminalForwarding={true} connected={false} onSend={onSend} />);
    const buttons = screen.getAllByRole("button");
    for (const btn of buttons) {
      expect(btn).toBeDisabled();
    }
  });

  it("renders a group label for accessibility", () => {
    render(<QuickCommands allowTerminalForwarding={true} connected={true} onSend={onSend} />);
    expect(screen.getByRole("group", { name: /quick commands/i })).toBeInTheDocument();
  });
});
