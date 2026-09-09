import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { EditorSections } from "../components/EditorSections";
const panels = { colors: <input aria-label="Color draft" defaultValue="original" />, sprites: <p>Sprite workspace</p>, assets: <p>Asset workspace</p>, export: <p>Export workspace</p> };

describe("focused editing sections", () => {
  it("starts with Colors and one accessible tab panel", () => {
    render(<EditorSections panels={panels} />);
    expect(screen.getByRole("tab", { name: /colors/i })).toHaveAttribute("aria-selected", "true");
    expect(screen.getAllByRole("tabpanel")).toHaveLength(1);
    expect(screen.queryByText("Sprite workspace")).not.toBeInTheDocument();
  });
  it("keeps local input drafts when moving away and back", () => {
    render(<EditorSections panels={panels} />);
    fireEvent.change(screen.getByLabelText("Color draft"), { target: { value: "my draft" } });
    fireEvent.click(screen.getByRole("tab", { name: /sprites/i }));
    expect(screen.getByText("Sprite workspace")).toBeVisible();
    expect(screen.getByLabelText("Color draft")).not.toBeVisible();
    fireEvent.click(screen.getByRole("tab", { name: /colors/i }));
    expect(screen.getByLabelText("Color draft")).toHaveValue("my draft");
  });
  it("moves focus with arrows and Home/End without silently changing tools", () => {
    render(<EditorSections panels={panels} />);
    const colors = screen.getByRole("tab", { name: /colors/i });
    const sprites = screen.getByRole("tab", { name: /sprites/i });
    const exportTab = screen.getByRole("tab", { name: /save a playable copy/i });
    colors.focus(); fireEvent.keyDown(colors, { key: "ArrowRight" });
    expect(sprites).toHaveFocus(); expect(colors).toHaveAttribute("aria-selected", "true");
    fireEvent.keyDown(sprites, { key: "End" }); expect(exportTab).toHaveFocus();
    fireEvent.keyDown(exportTab, { key: "ArrowRight" }); expect(colors).toHaveFocus();
    fireEvent.keyDown(colors, { key: "ArrowLeft" }); expect(exportTab).toHaveFocus();
    fireEvent.keyDown(exportTab, { key: "Home" }); expect(colors).toHaveFocus();
    fireEvent.click(sprites); expect(sprites).toHaveAttribute("aria-selected", "true");
  });
  it("links each tab to a labelled panel even before lazy mounting its contents", () => {
    render(<EditorSections panels={panels} />);
    for (const tab of screen.getAllByRole("tab")) {
      const panel = document.getElementById(tab.getAttribute("aria-controls")!);
      expect(panel).toHaveAttribute("aria-labelledby", tab.id);
    }
  });
});
