import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { PaletteEditor } from "../components/PaletteEditor";
const fixture = vi.hoisted(() => ({ currentPalette: [{ r: 8, g: 16, b: 24 }, { r: 80, g: 88, b: 96 }] as Array<{ r: number; g: number; b: number }> | null, updateColor: vi.fn() }));
vi.mock("../store/useStore", () => ({ useStore: () => fixture }));
beforeEach(() => { fixture.currentPalette = [{ r: 8, g: 16, b: 24 }, { r: 80, g: 88, b: 96 }]; fixture.updateColor.mockReset(); });
describe("accessible palette editing", () => {
  it.each([null, []])("provides guidance when there are no palette colors", (palette) => {
    fixture.currentPalette = palette; render(<PaletteEditor />);
    expect(screen.getByText("No palette loaded for this boxer.")).toBeVisible();
    expect(screen.queryByRole("slider")).not.toBeInTheDocument();
  });
  it("uses named native buttons and selects without changing ROM data", () => {
    render(<PaletteEditor />);
    const swatch = screen.getByRole("button", { name: "Color 2, #505860" });
    expect(swatch).toHaveAttribute("type", "button"); fireEvent.click(swatch);
    expect(swatch).toHaveAttribute("aria-pressed", "true");
    expect(screen.getByRole("heading", { name: "Color 2" })).toBeVisible();
    expect(fixture.updateColor).not.toHaveBeenCalled();
  });
  it("names all three sliders without relying on color", () => {
    render(<PaletteEditor />); fireEvent.click(screen.getByRole("button", { name: /Color 1,/ }));
    expect(screen.getByRole("slider", { name: "Red" })).toHaveValue("8");
    expect(screen.getByRole("slider", { name: "Green" })).toHaveValue("16");
    expect(screen.getByRole("slider", { name: "Blue" })).toHaveValue("24");
  });
  it("updates the selected palette slot through the existing store action", () => {
    render(<PaletteEditor />); fireEvent.click(screen.getByRole("button", { name: /Color 2,/ }));
    fireEvent.change(screen.getByRole("slider", { name: "Green" }), { target: { value: "128" } });
    expect(fixture.updateColor).toHaveBeenCalledTimes(1);
    expect(fixture.updateColor).toHaveBeenCalledWith(1, { r: 80, g: 128, b: 96 });
  });
});
