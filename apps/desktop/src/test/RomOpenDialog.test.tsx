import { fireEvent, render, screen } from "@testing-library/react";
import { afterAll, beforeAll, describe, expect, it, vi } from "vitest";
import { RomOpenDialog } from "../components/RomOpenDialog";
vi.mock("../components/RegionSelector", () => ({ RegionSelector: () => <button>Confirm selected ROM</button> }));
const originalShow = Object.getOwnPropertyDescriptor(HTMLDialogElement.prototype, "showModal");
const originalClose = Object.getOwnPropertyDescriptor(HTMLDialogElement.prototype, "close");
beforeAll(() => {
  Object.defineProperty(HTMLDialogElement.prototype, "showModal", { configurable: true, value: function(this: HTMLDialogElement) { this.open = true; } });
  Object.defineProperty(HTMLDialogElement.prototype, "close", { configurable: true, value: function(this: HTMLDialogElement) { this.open = false; } });
});
afterAll(() => {
  if (originalShow) Object.defineProperty(HTMLDialogElement.prototype, "showModal", originalShow); else Reflect.deleteProperty(HTMLDialogElement.prototype, "showModal");
  if (originalClose) Object.defineProperty(HTMLDialogElement.prototype, "close", originalClose); else Reflect.deleteProperty(HTMLDialogElement.prototype, "close");
});
const props = () => ({ path: "C:/test.sfc", busy: false, error: null, onCancel: vi.fn(), onDetected: vi.fn(), onConfirm: vi.fn().mockResolvedValue(undefined) });
describe("ROM confirmation dialog", () => {
  it("has a named modal and returns focus to its opener on unmount", () => {
    const opener = document.createElement("button"); document.body.appendChild(opener); opener.focus();
    const { unmount } = render(<RomOpenDialog {...props()} />);
    expect(screen.getByRole("dialog", { name: "Confirm ROM Region" })).toHaveAttribute("open");
    unmount(); expect(opener).toHaveFocus(); opener.remove();
  });
  it("supports Escape cancellation before loading", () => {
    const values = props(); render(<RomOpenDialog {...values} />);
    fireEvent(screen.getByRole("dialog"), new Event("cancel", { cancelable: true }));
    expect(values.onCancel).toHaveBeenCalledOnce();
  });
  it("prevents cancellation and duplicate confirmation during an open", () => {
    const values = props(); render(<RomOpenDialog {...values} busy />);
    expect(screen.getByRole("button", { name: "Cancel ROM selection" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Confirm selected ROM" })).toBeDisabled();
    fireEvent(screen.getByRole("dialog"), new Event("cancel", { cancelable: true }));
    expect(values.onCancel).not.toHaveBeenCalled(); expect(screen.getByRole("status")).toHaveTextContent("Opening your ROM");
  });
});
