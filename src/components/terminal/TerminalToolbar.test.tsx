/**
 * T099 — Vitest coverage for `TerminalToolbar`. Exercises FR-042 (filters,
 * regex search) and FR-043 (copy, export, clear view, auto-scroll lock).
 */
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import "@/i18n";

import { TerminalToolbar } from "./TerminalToolbar";
import type { HighlightSeverity } from "@/types/ipc";

const noop = () => {};

const defaultProps = {
  connected: true,
  autoScroll: true,
  onAutoScrollChange: noop as (v: boolean) => void,
  onSearch: noop as (query: string, isRegex: boolean) => void,
  onClearSearch: noop as () => void,
  onClearView: noop as () => void,
  onCopy: noop as () => void,
  onExport: noop as (format: "log" | "txt") => void,
  severityFilter: null as HighlightSeverity | null,
  onSeverityFilterChange: noop as (s: HighlightSeverity | null) => void,
  searchResultCount: 0,
};

describe("TerminalToolbar", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("renders the search input", () => {
    render(<TerminalToolbar {...defaultProps} />);
    expect(screen.getByRole("searchbox")).toBeInTheDocument();
  });

  it("calls onSearch when user types and submits", async () => {
    const onSearch = vi.fn();
    const user = userEvent.setup();
    render(<TerminalToolbar {...defaultProps} onSearch={onSearch} />);

    const input = screen.getByRole("searchbox");
    await user.type(input, "hello{Enter}");
    expect(onSearch).toHaveBeenCalledWith("hello", false);
  });

  it("toggles regex mode and passes it to onSearch", async () => {
    const onSearch = vi.fn();
    const user = userEvent.setup();
    render(<TerminalToolbar {...defaultProps} onSearch={onSearch} />);

    const regexBtn = screen.getByRole("button", { name: /regex/i });
    await user.click(regexBtn);

    const input = screen.getByRole("searchbox");
    await user.type(input, "err.*{Enter}");
    expect(onSearch).toHaveBeenCalledWith("err.*", true);
  });

  it("shows search result count when > 0", () => {
    render(<TerminalToolbar {...defaultProps} searchResultCount={42} />);
    expect(screen.getByText("42")).toBeInTheDocument();
  });

  it("calls onClearSearch when the clear-search button is clicked", async () => {
    const onClearSearch = vi.fn();
    const user = userEvent.setup();
    render(
      <TerminalToolbar {...defaultProps} onClearSearch={onClearSearch} searchResultCount={5} />,
    );

    const clearBtn = screen.getByRole("button", { name: /clear search/i });
    await user.click(clearBtn);
    expect(onClearSearch).toHaveBeenCalledOnce();
  });

  it("renders severity filter buttons (info, warn, error)", () => {
    render(<TerminalToolbar {...defaultProps} />);
    expect(screen.getByRole("button", { name: /info/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /warn/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /error/i })).toBeInTheDocument();
  });

  it("calls onSeverityFilterChange with severity when clicked", async () => {
    const onChange = vi.fn();
    const user = userEvent.setup();
    render(<TerminalToolbar {...defaultProps} onSeverityFilterChange={onChange} />);

    await user.click(screen.getByRole("button", { name: /error/i }));
    expect(onChange).toHaveBeenCalledWith("error");
  });

  it("calls onSeverityFilterChange with null to deselect active filter", async () => {
    const onChange = vi.fn();
    const user = userEvent.setup();
    render(
      <TerminalToolbar {...defaultProps} severityFilter="error" onSeverityFilterChange={onChange} />,
    );

    await user.click(screen.getByRole("button", { name: /error/i }));
    expect(onChange).toHaveBeenCalledWith(null);
  });

  it("calls onCopy when copy button is clicked", async () => {
    const onCopy = vi.fn();
    const user = userEvent.setup();
    render(<TerminalToolbar {...defaultProps} onCopy={onCopy} />);

    await user.click(screen.getByRole("button", { name: /copy/i }));
    expect(onCopy).toHaveBeenCalledOnce();
  });

  it("calls onExport('log') from the export .log button", async () => {
    const onExport = vi.fn();
    const user = userEvent.setup();
    render(<TerminalToolbar {...defaultProps} onExport={onExport} />);

    await user.click(screen.getByRole("button", { name: /\.log/i }));
    expect(onExport).toHaveBeenCalledWith("log");
  });

  it("calls onExport('txt') from the export .txt button", async () => {
    const onExport = vi.fn();
    const user = userEvent.setup();
    render(<TerminalToolbar {...defaultProps} onExport={onExport} />);

    await user.click(screen.getByRole("button", { name: /\.txt/i }));
    expect(onExport).toHaveBeenCalledWith("txt");
  });

  it("calls onClearView when clear-view button is clicked", async () => {
    const onClearView = vi.fn();
    const user = userEvent.setup();
    render(<TerminalToolbar {...defaultProps} onClearView={onClearView} />);

    await user.click(screen.getByRole("button", { name: /clear view/i }));
    expect(onClearView).toHaveBeenCalledOnce();
  });

  it("toggles auto-scroll via the checkbox", async () => {
    const onAutoScrollChange = vi.fn();
    const user = userEvent.setup();
    render(
      <TerminalToolbar {...defaultProps} autoScroll={true} onAutoScrollChange={onAutoScrollChange} />,
    );

    const toggle = screen.getByRole("checkbox", { name: /auto-scroll/i });
    await user.click(toggle);
    expect(onAutoScrollChange).toHaveBeenCalledWith(false);
  });

  it("disables interactive controls when not connected", () => {
    render(<TerminalToolbar {...defaultProps} connected={false} />);
    expect(screen.getByRole("searchbox")).toBeDisabled();
    expect(screen.getByRole("button", { name: /copy/i })).toBeDisabled();
  });
});
