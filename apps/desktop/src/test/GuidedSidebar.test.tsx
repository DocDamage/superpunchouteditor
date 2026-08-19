import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { GuidedSidebar } from "../components/GuidedSidebar";

vi.mock("../components/ThemeToggle", () => ({
  ThemeToggle: () => <button type="button">Theme</button>,
}));

vi.mock("../components/TesterPanel", () => ({
  TesterPanel: () => null,
}));

const noop = () => {};

describe("GuidedSidebar", () => {
  it("shows a short stable workflow and keeps advanced tools out of the primary path", () => {
    render(
      <GuidedSidebar
        tabItems={[
          { key: "editor", label: "Edit" },
          { key: "viewer", label: "Inspect" },
          { key: "test", label: "Test Game" },
          { key: "project", label: "Projects" },
          { key: "settings", label: "Settings" },
        ]}
        currentTab="editor"
        romSha1={null}
        boxers={[]}
        boxerPortraits={{}}
        canUndo={false}
        canRedo={false}
        editCount={0}
        pendingWritesCount={0}
        isDesktopRuntime={true}
        runtimeError=""
        error={null}
        onOpenRom={noop}
        onUndo={noop}
        onRedo={noop}
        onNavigate={noop}
        onSelectBoxer={noop}
        onOpenHelp={noop}
        onOpenKeyboardShortcuts={noop}
        onOpenEmulatorSettings={noop}
        onOpenExternalTools={noop}
      />
    );

    expect(screen.getByRole("button", { name: /open rom/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /edit & export/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /inspect/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /test game/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /projects/i })).toBeInTheDocument();
    expect(screen.queryByText(/advanced tools/i)).not.toBeInTheDocument();
  });

  it("renders visible experimental tools behind an advanced disclosure", () => {
    render(
      <GuidedSidebar
        tabItems={[
          { key: "editor", label: "Edit" },
          { key: "viewer", label: "Inspect" },
          { key: "test", label: "Test Game" },
          { key: "project", label: "Projects" },
          { key: "scripts", label: "Scripts (Experimental)" },
          { key: "settings", label: "Settings" },
        ]}
        currentTab="editor"
        romSha1="0123456789abcdef"
        boxers={[]}
        boxerPortraits={{}}
        canUndo={false}
        canRedo={false}
        editCount={0}
        pendingWritesCount={0}
        isDesktopRuntime={true}
        runtimeError=""
        error={null}
        onOpenRom={noop}
        onUndo={noop}
        onRedo={noop}
        onNavigate={noop}
        onSelectBoxer={noop}
        onOpenHelp={noop}
        onOpenKeyboardShortcuts={noop}
        onOpenEmulatorSettings={noop}
        onOpenExternalTools={noop}
      />
    );

    const disclosure = screen.getByRole("button", { name: /advanced tools/i });
    expect(disclosure).toHaveAttribute("aria-expanded", "false");
    expect(screen.queryByRole("button", { name: /scripts/i })).not.toBeInTheDocument();

    disclosure.click();

    expect(screen.getByRole("button", { name: /scripts/i })).toBeInTheDocument();
  });
});
