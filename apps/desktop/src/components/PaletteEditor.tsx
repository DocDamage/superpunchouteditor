import { useId, useState } from "react";
import { useStore, type Color } from "../store/useStore";
import "./PaletteEditor.css";

const CHANNELS: Array<{ key: keyof Color; label: string }> = [
  { key: "r", label: "Red" }, { key: "g", label: "Green" }, { key: "b", label: "Blue" },
];
const hex = (color: Color) => `#${[color.r, color.g, color.b].map((value) => value.toString(16).padStart(2, "0")).join("").toUpperCase()}`;

export const PaletteEditor = () => {
  const { currentPalette, updateColor } = useStore();
  const [selectedIndex, setSelectedIndex] = useState<number | null>(null);
  const id = useId();
  if (!currentPalette?.length) return (
    <div className="empty-state"><p>No palette loaded for this boxer.</p><p>Choose another boxer or reopen your ROM to try again.</p></div>
  );
  const selectedColor = selectedIndex !== null ? currentPalette[selectedIndex] : null;
  const updateChannel = (channel: keyof Color, value: number) => {
    if (selectedIndex === null || !selectedColor || !Number.isFinite(value)) return;
    void updateColor(selectedIndex, { ...selectedColor, [channel]: Math.max(0, Math.min(255, Math.round(value))) });
  };
  return (
    <div className="palette-editor club-palette">
      <h3 id={`${id}-title`}>Palette Editor</h3>
      <p id={`${id}-instructions`} className="club-palette-hint">Pick a numbered color to edit. Use Tab and Enter or Space with a keyboard.</p>
      <div className="palette-grid" role="group" aria-labelledby={`${id}-title`} aria-describedby={`${id}-instructions`}>
        {currentPalette.map((color, index) => (
          <button type="button" key={index} className={`palette-swatch ${selectedIndex === index ? "selected" : ""}`}
            style={{ backgroundColor: `rgb(${color.r}, ${color.g}, ${color.b})` }}
            aria-label={`Color ${index + 1}, ${hex(color)}`} aria-pressed={selectedIndex === index}
            title={`Color ${index + 1}: ${hex(color)}`} onClick={() => setSelectedIndex(index)}>
            <span className="club-swatch-label" aria-hidden="true">{index + 1}{selectedIndex === index ? " ✓" : ""}</span>
          </button>
        ))}
      </div>
      {selectedColor && selectedIndex !== null ? (
        <section className="color-picker-panel" aria-labelledby={`${id}-selected`}>
          <div className="club-selected-color">
            <div className="color-preview-large" aria-hidden="true"
              style={{ backgroundColor: `rgb(${selectedColor.r}, ${selectedColor.g}, ${selectedColor.b})` }} />
            <h4 id={`${id}-selected`}>Color {selectedIndex + 1}</h4><code>{hex(selectedColor)}</code>
          </div>
          <div className="slider-group">
            {CHANNELS.map(({ key, label }) => (
              <div className="slider-row" key={key}>
                <label htmlFor={`${id}-${key}`}>{label}</label>
                <input id={`${id}-${key}`} type="range" min="0" max="255" step="8" value={selectedColor[key]}
                  aria-valuetext={`${selectedColor[key]} out of 255`}
                  onChange={(event) => updateChannel(key, Number(event.target.value))} />
                <output htmlFor={`${id}-${key}`}>{selectedColor[key]}</output>
              </div>
            ))}
            <p className="club-palette-hint">Changes use the editor’s existing ROM edit history. Undo restores the previous edit.</p>
          </div>
        </section>
      ) : <p className="club-palette-hint">Select a color above to show its controls.</p>}
    </div>
  );
};
