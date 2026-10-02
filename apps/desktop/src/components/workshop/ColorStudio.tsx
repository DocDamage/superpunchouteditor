import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useStore } from "../../store/useStore";
import type { BoxerRecord, Color } from "../../store/useStore";
import { AssembledPosePreview } from "../AssembledPosePreview";
import { showToast } from "../ToastContainer";
import { StarIcon } from "../icons";
import {
  hexToRgb,
  makeGold,
  makeGrayscale,
  makeIce,
  makeNeon,
  makeShadow,
  paintWithHue,
  palettesEqual,
  PALETTE_ROW_SIZE,
  rgbToHex,
  shiftHue,
  snapToConsole,
} from "../../utils/colorMagic";

interface ColorStudioProps {
  boxer: BoxerRecord;
  /** Changes whenever an edit, undo or redo happens, so the preview redraws. */
  revisionKey: string;
}

const PAINT_COLORS: Array<{ name: string; hue: number; swatch: string }> = [
  { name: "Red", hue: 0, swatch: "#e5121d" },
  { name: "Orange", hue: 26, swatch: "#f57c00" },
  { name: "Yellow", hue: 50, swatch: "#f6c40c" },
  { name: "Green", hue: 130, swatch: "#12a150" },
  { name: "Teal", hue: 176, swatch: "#0fa3a3" },
  { name: "Blue", hue: 220, swatch: "#2a63e0" },
  { name: "Purple", hue: 272, swatch: "#7b3fe4" },
  { name: "Pink", hue: 325, swatch: "#e83e9c" },
];

type Effect = {
  key: string;
  name: string;
  hint: string;
  apply: (palette: Color[], keepSkin: boolean) => Color[];
};

const EFFECTS: Effect[] = [
  { key: "gold", name: "Gold", hint: "Shiny champion gold", apply: (p, keepSkin) => makeGold(p, { keepSkin }) },
  { key: "ice", name: "Ice", hint: "Frozen blue", apply: (p, keepSkin) => makeIce(p, { keepSkin }) },
  { key: "neon", name: "Neon", hint: "Extra bright", apply: (p, keepSkin) => makeNeon(p, { keepSkin }) },
  { key: "shadow", name: "Shadow", hint: "Dark and moody", apply: (p, keepSkin) => makeShadow(p, { keepSkin }) },
  { key: "gray", name: "Old Movie", hint: "Black and white", apply: (p, keepSkin) => makeGrayscale(p, { keepSkin }) },
];

export function ColorStudio({ boxer, revisionKey }: ColorStudioProps): React.ReactElement {
  const { currentPalette, applyPalette, restoreOriginalPalette } = useStore();
  const [keepSkin, setKeepSkin] = useState(true);
  const [busy, setBusy] = useState(false);
  const [selectedIndex, setSelectedIndex] = useState<number | null>(null);
  const [draft, setDraft] = useState<Color | null>(null);
  const colorInputRef = useRef<HTMLInputElement>(null);

  // Forget the selected swatch when another boxer is opened.
  useEffect(() => {
    setSelectedIndex(null);
    setDraft(null);
  }, [boxer.key]);

  const run = useCallback(
    async (label: string, next: Color[]) => {
      if (!currentPalette || busy) return;
      if (palettesEqual(currentPalette, next)) {
        showToast("That look is already on. Try a different one!", "info", 2500);
        return;
      }
      setBusy(true);
      try {
        await applyPalette(next, `${label} for ${boxer.name}`);
      } finally {
        setBusy(false);
      }
    },
    [applyPalette, boxer.name, busy, currentPalette],
  );

  const selectedColor = useMemo(
    () => (selectedIndex !== null && currentPalette ? currentPalette[selectedIndex] ?? null : null),
    [currentPalette, selectedIndex],
  );
  const shownColor = draft ?? selectedColor;

  const commitDraft = useCallback(
    async (color: Color | null) => {
      if (!color || selectedIndex === null || !currentPalette) return;
      const snapped = snapToConsole(color);
      setDraft(null);
      const current = currentPalette[selectedIndex];
      if (current.r === snapped.r && current.g === snapped.g && current.b === snapped.b) return;
      const next = currentPalette.map((existing, index) => (index === selectedIndex ? snapped : existing));
      await run(`Color ${selectedIndex + 1}`, next);
    },
    [currentPalette, run, selectedIndex],
  );

  // The colour picker fires its native "change" event once, when the user has
  // finished choosing. Committing there keeps Undo to one step per choice.
  useEffect(() => {
    const input = colorInputRef.current;
    if (!input) return;
    const onChange = () => {
      const rgb = hexToRgb(input.value);
      if (rgb) void commitDraft(rgb);
    };
    input.addEventListener("change", onChange);
    return () => input.removeEventListener("change", onChange);
  }, [commitDraft, selectedIndex]);

  if (!currentPalette) {
    return (
      <div className="workshop-empty">
        <h3>No colors to change here</h3>
        <p>This boxer does not have a color set the editor can change yet. Try another boxer.</p>
      </div>
    );
  }

  const rows: Color[][] = [];
  for (let start = 0; start < currentPalette.length; start += PALETTE_ROW_SIZE) {
    rows.push(currentPalette.slice(start, start + PALETTE_ROW_SIZE));
  }

  return (
    <div className="color-studio">
      <div className="color-studio-stage">
        <AssembledPosePreview boxer={boxer} refreshKey={revisionKey} friendly />
      </div>

      <div className="color-studio-tools">
        <section className="magic-card">
          <header>
            <span className="magic-card-icon pad-yellow" aria-hidden="true"><StarIcon size={18} /></span>
            <div>
              <h3>Magic Paint</h3>
              <p>Tap a color to repaint {boxer.name} in one go.</p>
            </div>
          </header>

          <div className="paint-grid" role="group" aria-label="Paint colors">
            {PAINT_COLORS.map((paint) => (
              <button
                key={paint.name}
                type="button"
                className="paint-chip"
                style={{ ["--chip" as string]: paint.swatch }}
                disabled={busy}
                onClick={() => void run(`${paint.name} paint`, paintWithHue(currentPalette, paint.hue, { keepSkin }))}
              >
                <span className="paint-chip-dot" aria-hidden="true" />
                {paint.name}
              </button>
            ))}
          </div>

          <label className="magic-toggle">
            <input type="checkbox" checked={keepSkin} onChange={(event) => setKeepSkin(event.target.checked)} />
            <span>
              Keep skin color
              <small>The editor guesses which colors are skin. Turn this off to paint everything.</small>
            </span>
          </label>
        </section>

        <section className="magic-card">
          <header>
            <div>
              <h3>Special Looks</h3>
              <p>Fun styles for the whole boxer.</p>
            </div>
          </header>
          <div className="effect-grid">
            {EFFECTS.map((effect) => (
              <button
                key={effect.key}
                type="button"
                className="secondary effect-button"
                disabled={busy}
                title={effect.hint}
                onClick={() => void run(`${effect.name} look`, effect.apply(currentPalette, keepSkin))}
              >
                {effect.name}
                <small>{effect.hint}</small>
              </button>
            ))}
            <button
              type="button"
              className="secondary effect-button surprise"
              disabled={busy}
              onClick={() => {
                const degrees = 40 + Math.floor(Math.random() * 280);
                void run("Surprise colors", shiftHue(currentPalette, degrees, { keepSkin }));
              }}
            >
              Surprise Me!
              <small>Random new colors</small>
            </button>
          </div>

          <button
            type="button"
            className="quiet-button magic-reset"
            disabled={busy}
            onClick={async () => {
              setBusy(true);
              try {
                const changed = await restoreOriginalPalette();
                showToast(
                  changed ? `${boxer.name} has the original colors again.` : "These are already the original colors.",
                  changed ? "success" : "info",
                  3000,
                );
              } finally {
                setBusy(false);
              }
            }}
          >
            Put the original colors back
          </button>
        </section>
      </div>

      <details className="fine-colors">
        <summary>
          <span>Change one color at a time</span>
          <small>For careful work. Tap a square, then pick its new color.</small>
        </summary>

        <div className="fine-colors-body">
          <div className="swatch-rows">
            {rows.map((row, rowIndex) => (
              <div key={rowIndex} className="swatch-row" role="group" aria-label={`Color row ${rowIndex + 1}`}>
                {row.map((color, columnIndex) => {
                  const index = rowIndex * PALETTE_ROW_SIZE + columnIndex;
                  const isClear = columnIndex === 0;
                  return (
                    <button
                      key={index}
                      type="button"
                      className={`swatch ${selectedIndex === index ? "selected" : ""} ${isClear ? "clear" : ""}`}
                      style={{ backgroundColor: `rgb(${color.r}, ${color.g}, ${color.b})` }}
                      aria-label={isClear ? `Row ${rowIndex + 1} see-through color` : `Color ${index + 1}`}
                      title={isClear ? "See-through. The game does not draw this one." : `Color ${index + 1}`}
                      aria-pressed={selectedIndex === index}
                      onClick={() => {
                        setDraft(null);
                        setSelectedIndex(index);
                      }}
                    />
                  );
                })}
              </div>
            ))}
          </div>

          {shownColor && selectedIndex !== null ? (
            <div className="swatch-editor">
              <div
                className="swatch-editor-preview"
                style={{ backgroundColor: `rgb(${shownColor.r}, ${shownColor.g}, ${shownColor.b})` }}
              />
              <div className="swatch-editor-controls">
                <label className="swatch-picker">
                  Pick a new color
                  <input
                    ref={colorInputRef}
                    type="color"
                    value={rgbToHex(shownColor)}
                    disabled={busy}
                    onInput={(event) => {
                      const rgb = hexToRgb((event.target as HTMLInputElement).value);
                      if (rgb) setDraft(rgb);
                    }}
                    onChange={() => undefined}
                  />
                </label>
                {(["r", "g", "b"] as const).map((channel) => (
                  <div className="slider-row" key={channel}>
                    <label htmlFor={`swatch-${channel}`} className={`channel-${channel}`}>
                      {channel.toUpperCase()}
                    </label>
                    <input
                      id={`swatch-${channel}`}
                      type="range"
                      min={0}
                      max={255}
                      step={8}
                      value={shownColor[channel]}
                      disabled={busy}
                      onChange={(event) => setDraft({ ...shownColor, [channel]: Number(event.target.value) })}
                      onPointerUp={() => void commitDraft(draft)}
                      onKeyUp={() => void commitDraft(draft)}
                      onBlur={() => void commitDraft(draft)}
                    />
                    <span>{shownColor[channel]}</span>
                  </div>
                ))}
                <p className="swatch-note">
                  Colors snap to the nearest one the game can show, so what you see here is what you get.
                </p>
              </div>
            </div>
          ) : (
            <p className="swatch-note">Tap a color square above to change it.</p>
          )}
        </div>
      </details>
    </div>
  );
}
