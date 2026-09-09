import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { GuidedSidebar } from "../components/GuidedSidebar";
import type { ComponentProps } from "react";
vi.mock("../components/ThemeToggle", () => ({ ThemeToggle: () => <button>Theme</button> }));
vi.mock("../components/TesterPanel", () => ({ TesterPanel: () => null }));
function props(): ComponentProps<typeof GuidedSidebar> {
  return { tabItems: [{ key: "editor", label: "Edit" }, { key: "viewer", label: "Inspect" }, { key: "test", label: "Test Game" }, { key: "project", label: "Projects" }],
    currentTab: "editor", romSha1: null, boxers: [], boxerPortraits: {}, canUndo: false, canRedo: false, editCount: 0, pendingWritesCount: 0,
    isDesktopRuntime: true, runtimeError: "", error: null, onOpenRom: vi.fn(), onUndo: vi.fn(), onRedo: vi.fn(), onNavigate: vi.fn(),
    onSelectBoxer: vi.fn(), onOpenHelp: vi.fn(), onOpenKeyboardShortcuts: vi.fn(), onOpenEmulatorSettings: vi.fn(), onOpenExternalTools: vi.fn() };
}
beforeEach(() => localStorage.clear());
describe("sidebar usability safeguards", () => {
  it("explains and disables ROM-only destinations until a ROM is loaded", () => {
    render(<GuidedSidebar {...props()} />);
    expect(screen.getByRole("button", { name: /inspect/i })).toBeDisabled();
    expect(screen.getByRole("button", { name: /test game/i })).toBeDisabled();
    expect(screen.getByRole("button", { name: /projects/i })).toBeEnabled();
    expect(screen.getByRole("button", { name: /edit & export/i })).toBeEnabled();
  });
  it("keeps experimental labels visible after applying friendly navigation labels", () => {
    const values = props(); values.tabItems.push({ key: "compare", label: "Compare (Experimental)" });
    render(<GuidedSidebar {...values} romSha1="rom" />);
    expect(screen.getByRole("button", { name: /compare \(experimental\)/i })).toBeInTheDocument();
  });
  it("provides a searchable picker when editing a selected boxer", () => {
    render(<GuidedSidebar {...props()} romSha1="rom" selectedBoxerKey="gabby" boxers={[{ key: "gabby", name: "Gabby Jay" }]} />);
    expect(screen.getByRole("searchbox")).toBeVisible();
    expect(screen.getByRole("button", { name: /Gabby Jay/ })).toHaveAttribute("aria-pressed", "true");
  });
  it("locks session changes while a boxer is loading", () => {
    render(<GuidedSidebar {...props()} romSha1="rom" selectingBoxer />);
    expect(screen.getByRole("button", { name: /switch rom/i })).toBeDisabled();
    expect(screen.getByRole("button", { name: /projects/i })).toBeDisabled();
  });
  it("lets a handled error be dismissed without modifying the session", () => {
    const dismiss = vi.fn(); const values = props();
    render(<GuidedSidebar {...values} error="An import failed" onDismissError={dismiss} />);
    fireEvent.click(screen.getByRole("button", { name: "Dismiss message" }));
    expect(dismiss).toHaveBeenCalledOnce(); expect(values.onOpenRom).not.toHaveBeenCalled();
  });
});
