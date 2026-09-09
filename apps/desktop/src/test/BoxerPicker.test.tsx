import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { BoxerPicker } from "../components/BoxerPicker";
const boxers = [{ key: "gabby", name: "Gabby Jay" }, { key: "bear", name: "Bear Hugger" }];

describe("boxer character selection", () => {
  it("filters names without case or surrounding-space sensitivity", () => {
    render(<BoxerPicker boxers={boxers} portraits={{}} onSelect={vi.fn()} />);
    fireEvent.change(screen.getByRole("searchbox"), { target: { value: " BEAR " } });
    expect(screen.getByRole("button", { name: "Bear Hugger" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Gabby Jay" })).not.toBeInTheDocument();
    expect(screen.getByRole("status")).toHaveTextContent("1 of 2 boxers");
  });
  it("offers recovery from an empty search", () => {
    render(<BoxerPicker boxers={boxers} portraits={{}} onSelect={vi.fn()} />);
    fireEvent.change(screen.getByRole("searchbox"), { target: { value: "unknown" } });
    expect(screen.getByText(/no boxers match/i)).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Clear boxer search" }));
    expect(screen.getByRole("button", { name: "Gabby Jay" })).toBeInTheDocument();
  });
  it("identifies the selected fighter without relying on color", () => {
    const select = vi.fn(); render(<BoxerPicker boxers={boxers} portraits={{}} selectedKey="gabby" onSelect={select} />);
    expect(screen.getByRole("button", { name: /Gabby Jay/ })).toHaveAttribute("aria-pressed", "true");
    expect(screen.getByText("Selected")).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Bear Hugger" }));
    expect(select).toHaveBeenCalledWith("bear");
  });
  it("prevents overlapping selections while a boxer is loading", () => {
    const select = vi.fn(); render(<BoxerPicker boxers={boxers} portraits={{}} onSelect={select} busy />);
    const button = screen.getByRole("button", { name: "Bear Hugger" });
    expect(button).toBeDisabled(); fireEvent.click(button); expect(select).not.toHaveBeenCalled();
  });
});
