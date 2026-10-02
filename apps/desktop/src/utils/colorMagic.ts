/**
 * One-tap recolouring helpers for boxer palettes.
 *
 * Every function is pure: it takes a palette and returns a new one. Colours are
 * always snapped to what the console can really display (5 bits per channel),
 * so what the editor shows is exactly what the game will show.
 */

export interface Rgb {
  r: number;
  g: number;
  b: number;
}

export interface Hsl {
  /** Hue in degrees, 0–360. */
  h: number;
  /** Saturation, 0–1. */
  s: number;
  /** Lightness, 0–1. */
  l: number;
}

export interface RecolorOptions {
  /** Leave skin-like colours alone so only the outfit changes. */
  keepSkin?: boolean;
}

/** Colours per palette row. The first colour of each row is the see-through one. */
export const PALETTE_ROW_SIZE = 16;

const clamp01 = (value: number): number => Math.min(1, Math.max(0, value));

/** Snap one 0–255 channel to the nearest value the console can display. */
export function snapChannel(value: number): number {
  const five = Math.min(31, Math.max(0, Math.round((value / 255) * 31)));
  return (five << 3) | (five >> 2);
}

/** Snap a colour to the nearest one the console can display. */
export function snapToConsole(color: Rgb): Rgb {
  return { r: snapChannel(color.r), g: snapChannel(color.g), b: snapChannel(color.b) };
}

export function rgbToHsl({ r, g, b }: Rgb): Hsl {
  const rn = r / 255;
  const gn = g / 255;
  const bn = b / 255;
  const max = Math.max(rn, gn, bn);
  const min = Math.min(rn, gn, bn);
  const l = (max + min) / 2;
  const delta = max - min;

  if (delta === 0) return { h: 0, s: 0, l };

  const s = delta / (1 - Math.abs(2 * l - 1));
  let h: number;
  if (max === rn) h = ((gn - bn) / delta) % 6;
  else if (max === gn) h = (bn - rn) / delta + 2;
  else h = (rn - gn) / delta + 4;

  return { h: (h * 60 + 360) % 360, s: clamp01(s), l };
}

export function hslToRgb({ h, s, l }: Hsl): Rgb {
  const hue = ((h % 360) + 360) % 360;
  const chroma = (1 - Math.abs(2 * l - 1)) * clamp01(s);
  const x = chroma * (1 - Math.abs(((hue / 60) % 2) - 1));
  const m = l - chroma / 2;

  let rgb: [number, number, number];
  if (hue < 60) rgb = [chroma, x, 0];
  else if (hue < 120) rgb = [x, chroma, 0];
  else if (hue < 180) rgb = [0, chroma, x];
  else if (hue < 240) rgb = [0, x, chroma];
  else if (hue < 300) rgb = [x, 0, chroma];
  else rgb = [chroma, 0, x];

  return {
    r: Math.round((rgb[0] + m) * 255),
    g: Math.round((rgb[1] + m) * 255),
    b: Math.round((rgb[2] + m) * 255),
  };
}

export function rgbToHex({ r, g, b }: Rgb): string {
  return `#${[r, g, b].map((channel) => channel.toString(16).padStart(2, "0")).join("")}`;
}

export function hexToRgb(hex: string): Rgb | null {
  const match = /^#?([0-9a-f]{6})$/i.exec(hex.trim());
  if (!match) return null;
  const value = parseInt(match[1], 16);
  return { r: (value >> 16) & 0xff, g: (value >> 8) & 0xff, b: value & 0xff };
}

/** Grey, black and white have no hue to change. */
export function isNeutral(color: Rgb): boolean {
  const { s, l } = rgbToHsl(color);
  return s < 0.12 || l < 0.06 || l > 0.96;
}

/**
 * A best guess at whether a colour is skin. Skin shades sit in the
 * orange-brown range with moderate saturation. This is a guess, which is why
 * the interface always labels it as one.
 */
export function isSkinTone(color: Rgb): boolean {
  const max = Math.max(color.r, color.g, color.b);
  const min = Math.min(color.r, color.g, color.b);
  if (max === 0) return false;

  const { h } = rgbToHsl(color);
  const brightness = max / 255;
  const vividness = (max - min) / max;

  if (h < 8 || h > 45) return false;

  // Light skin: soft, not vivid. A vivid light orange is clothing.
  if (brightness >= 0.6) return vividness >= 0.12 && vividness <= 0.62;

  // Shaded and dark skin: browns. Deep red-oranges below this hue are clothing.
  return h >= 16 && h <= 42 && vividness >= 0.3 && brightness >= 0.12;
}

function isLocked(index: number, color: Rgb, options: RecolorOptions): boolean {
  if (index % PALETTE_ROW_SIZE === 0) return true; // see-through slot
  if (options.keepSkin && isSkinTone(color)) return true;
  return false;
}

function mapPalette(
  palette: readonly Rgb[],
  options: RecolorOptions,
  transform: (hsl: Hsl, color: Rgb) => Hsl | null,
): Rgb[] {
  return palette.map((color, index) => {
    if (isLocked(index, color, options)) return { ...color };
    const next = transform(rgbToHsl(color), color);
    return next ? snapToConsole(hslToRgb(next)) : { ...color };
  });
}

/** Paint every colourful shade with one hue, keeping light and dark shading. */
export function paintWithHue(palette: readonly Rgb[], hue: number, options: RecolorOptions = {}): Rgb[] {
  return mapPalette(palette, options, (hsl, color) =>
    isNeutral(color) ? null : { h: hue, s: Math.max(hsl.s, 0.55), l: hsl.l },
  );
}

/** Rotate every hue by the same amount. */
export function shiftHue(palette: readonly Rgb[], degrees: number, options: RecolorOptions = {}): Rgb[] {
  return mapPalette(palette, options, (hsl, color) =>
    isNeutral(color) ? null : { ...hsl, h: hsl.h + degrees },
  );
}

/** Shiny gold: warm yellow hues with strong shine. */
export function makeGold(palette: readonly Rgb[], options: RecolorOptions = {}): Rgb[] {
  return mapPalette(palette, options, (hsl) => ({
    h: 44,
    s: 0.85,
    l: clamp01(0.12 + hsl.l * 0.78),
  }));
}

/** Frozen: pale blues. */
export function makeIce(palette: readonly Rgb[], options: RecolorOptions = {}): Rgb[] {
  return mapPalette(palette, options, (hsl) => ({
    h: 198,
    s: 0.6,
    l: clamp01(0.2 + hsl.l * 0.75),
  }));
}

/** Brighter, punchier colours. */
export function makeNeon(palette: readonly Rgb[], options: RecolorOptions = {}): Rgb[] {
  return mapPalette(palette, options, (hsl, color) =>
    isNeutral(color) ? null : { h: hsl.h, s: 1, l: clamp01(0.5 + (hsl.l - 0.5) * 0.7) },
  );
}

/** Darker and moodier. */
export function makeShadow(palette: readonly Rgb[], options: RecolorOptions = {}): Rgb[] {
  return mapPalette(palette, options, (hsl) => ({ h: hsl.h, s: hsl.s * 0.7, l: hsl.l * 0.55 }));
}

/** Old black-and-white film. */
export function makeGrayscale(palette: readonly Rgb[], options: RecolorOptions = {}): Rgb[] {
  return mapPalette(palette, options, (hsl) => ({ h: 0, s: 0, l: hsl.l }));
}

export function palettesEqual(a: readonly Rgb[], b: readonly Rgb[]): boolean {
  return a.length === b.length && a.every((color, index) => {
    const other = b[index];
    return color.r === other.r && color.g === other.g && color.b === other.b;
  });
}
