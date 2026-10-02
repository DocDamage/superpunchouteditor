import { describe, expect, it } from "vitest";
import { sortBoxersInGameOrder } from "../store/useStore";
import {
  hexToRgb,
  hslToRgb,
  isSkinTone,
  makeGold,
  makeGrayscale,
  paintWithHue,
  palettesEqual,
  rgbToHex,
  rgbToHsl,
  shiftHue,
  snapChannel,
  snapToConsole,
  type Rgb,
} from "../utils/colorMagic";

const CLEAR: Rgb = { r: 0, g: 0, b: 132 };
const LIGHT_SKIN: Rgb = { r: 247, g: 198, b: 156 };
const DARK_SKIN: Rgb = { r: 132, g: 82, b: 41 };
const RED_TRUNKS: Rgb = { r: 222, g: 41, b: 0 };
const TRUNKS_SHADE: Rgb = { r: 148, g: 33, b: 0 };
const WHITE: Rgb = { r: 255, g: 255, b: 255 };
const GREY: Rgb = { r: 132, g: 132, b: 132 };

const palette: Rgb[] = [CLEAR, LIGHT_SKIN, DARK_SKIN, RED_TRUNKS, TRUNKS_SHADE, WHITE, GREY];

describe("colorMagic", () => {
  it("snaps channels to values the console can display", () => {
    expect(snapChannel(0)).toBe(0);
    expect(snapChannel(255)).toBe(255);
    expect(snapChannel(250)).toBe(247);
    for (let value = 0; value <= 255; value += 1) {
      const snapped = snapChannel(value);
      // A snapped value survives a 5-bit round trip unchanged.
      const five = snapped >> 3;
      expect((five << 3) | (five >> 2)).toBe(snapped);
      expect(snapChannel(snapped)).toBe(snapped);
    }
    expect(snapToConsole({ r: 1, g: 128, b: 254 })).toEqual({ r: 0, g: 132, b: 255 });
  });

  it("round-trips colours through HSL and hex", () => {
    for (const color of [RED_TRUNKS, LIGHT_SKIN, GREY, { r: 0, g: 132, b: 255 }]) {
      const back = hslToRgb(rgbToHsl(color));
      expect(Math.abs(back.r - color.r)).toBeLessThanOrEqual(1);
      expect(Math.abs(back.g - color.g)).toBeLessThanOrEqual(1);
      expect(Math.abs(back.b - color.b)).toBeLessThanOrEqual(1);
      expect(hexToRgb(rgbToHex(color))).toEqual(color);
    }
    expect(hexToRgb("not a color")).toBeNull();
  });

  it("guesses skin without mistaking vivid clothing for it", () => {
    expect(isSkinTone(LIGHT_SKIN)).toBe(true);
    expect(isSkinTone(DARK_SKIN)).toBe(true);
    expect(isSkinTone(RED_TRUNKS)).toBe(false);
    expect(isSkinTone(TRUNKS_SHADE)).toBe(false);
    expect(isSkinTone({ r: 255, g: 132, b: 0 })).toBe(false);
    expect(isSkinTone({ r: 0, g: 0, b: 0 })).toBe(false);
    expect(isSkinTone({ r: 41, g: 99, b: 222 })).toBe(false);
  });

  it("paints clothing but keeps skin, greys and the see-through slot", () => {
    const blue = paintWithHue(palette, 220, { keepSkin: true });
    expect(blue[0]).toEqual(CLEAR);
    expect(blue[1]).toEqual(LIGHT_SKIN);
    expect(blue[2]).toEqual(DARK_SKIN);
    expect(blue[5]).toEqual(WHITE);
    expect(blue[6]).toEqual(GREY);
    expect(blue[3].b).toBeGreaterThan(blue[3].r);
    expect(blue[4].b).toBeGreaterThan(blue[4].r);
    // Shading is kept: the shade stays darker than the main colour.
    expect(rgbToHsl(blue[4]).l).toBeLessThan(rgbToHsl(blue[3]).l);
  });

  it("paints skin too when asked", () => {
    const green = paintWithHue(palette, 130, { keepSkin: false });
    expect(green[1].g).toBeGreaterThan(green[1].r);
    expect(green[0]).toEqual(CLEAR);
  });

  it("only ever produces displayable colours", () => {
    const results = [
      paintWithHue(palette, 300),
      shiftHue(palette, 77),
      makeGold(palette),
      makeGrayscale(palette),
    ];
    for (const result of results) {
      expect(result).toHaveLength(palette.length);
      for (const color of result.slice(1)) {
        expect(snapToConsole(color)).toEqual(color);
      }
    }
  });

  it("never mutates the palette it was given", () => {
    const copy = palette.map((color) => ({ ...color }));
    makeGold(palette);
    shiftHue(palette, 120);
    expect(palettesEqual(palette, copy)).toBe(true);
  });

  it("makes greys in the old-movie look", () => {
    const grey = makeGrayscale(palette);
    expect(grey[3].r).toBe(grey[3].g);
    expect(grey[3].g).toBe(grey[3].b);
  });
});

describe("sortBoxersInGameOrder", () => {
  it("lists boxers circuit by circuit and puts unknown ones last", () => {
    const names = ["Little Mac", "Zed Custom", "Mr. Sandman", "Gabby Jay", "Alpha Custom", "Bear Hugger"];
    const sorted = sortBoxersInGameOrder(names.map((name) => ({ name }))).map((boxer) => boxer.name);
    expect(sorted).toEqual(["Gabby Jay", "Bear Hugger", "Mr. Sandman", "Little Mac", "Alpha Custom", "Zed Custom"]);
  });
});
