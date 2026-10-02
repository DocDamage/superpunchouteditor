import { describe, expect, it } from "vitest";
import {
  DETAIL_LEVELS,
  applyStampShape,
  boxBlur,
  cropRect,
  findFigureBox,
  guessHeadPlacement,
  posterize,
  simplifyPicture,
} from "../utils/photoStamp";

function solid(width: number, height: number, color: [number, number, number]): Uint8ClampedArray {
  const rgba = new Uint8ClampedArray(width * height * 4);
  for (let i = 0; i < width * height; i += 1) {
    rgba.set([...color, 255], i * 4);
  }
  return rgba;
}

describe("cropRect", () => {
  it("uses the largest centred square of a wide photo", () => {
    const rect = cropRect(400, 200, 1, { zoom: 1, centerX: 0.5, centerY: 0.5 });
    expect(rect).toEqual({ x: 100, y: 0, width: 200, height: 200 });
  });

  it("zooms in around the chosen centre", () => {
    const rect = cropRect(400, 400, 1, { zoom: 2, centerX: 0.5, centerY: 0.5 });
    expect(rect).toEqual({ x: 100, y: 100, width: 200, height: 200 });
  });

  it("never leaves the photo, however far it is slid", () => {
    for (const centerX of [-1, 0, 0.5, 1, 2]) {
      for (const zoom of [0.2, 1, 3]) {
        const rect = cropRect(300, 200, 1, { zoom, centerX, centerY: centerX });
        expect(rect.x).toBeGreaterThanOrEqual(0);
        expect(rect.y).toBeGreaterThanOrEqual(0);
        expect(rect.x + rect.width).toBeLessThanOrEqual(300 + 1e-9);
        expect(rect.y + rect.height).toBeLessThanOrEqual(200 + 1e-9);
        expect(rect.width).toBeCloseTo(rect.height);
      }
    }
  });
});

describe("simplifying a picture", () => {
  it("posterize reduces the number of different values", () => {
    const rgba = new Uint8ClampedArray(256 * 4);
    for (let i = 0; i < 256; i += 1) rgba.set([i, i, i, 255], i * 4);
    posterize(rgba, 3);
    const values = new Set<number>();
    for (let i = 0; i < 256; i += 1) values.add(rgba[i * 4]);
    expect([...values].sort((a, b) => a - b)).toEqual([0, 128, 255]);
  });

  it("posterize with fewer than two shades changes nothing", () => {
    const rgba = solid(2, 2, [37, 99, 201]);
    posterize(rgba, 0);
    expect([...rgba.slice(0, 3)]).toEqual([37, 99, 201]);
  });

  it("blur smooths a single bright pixel and keeps alpha", () => {
    const rgba = solid(3, 3, [0, 0, 0]);
    rgba.set([255, 255, 255, 255], 4 * 4);
    boxBlur(rgba, 3, 3, 1);
    expect(rgba[4 * 4]).toBeLessThan(255);
    expect(rgba[0]).toBeGreaterThan(0);
    expect(rgba[3]).toBe(255);
  });

  it("each level is at least as simple as the one before", () => {
    const countColors = (level: number): number => {
      const rgba = new Uint8ClampedArray(16 * 16 * 4);
      for (let i = 0; i < 256; i += 1) rgba.set([i, (i * 7) % 256, (i * 13) % 256, 255], i * 4);
      simplifyPicture(rgba, 16, 16, level);
      const seen = new Set<string>();
      for (let i = 0; i < 256; i += 1) seen.add(`${rgba[i * 4]},${rgba[i * 4 + 1]},${rgba[i * 4 + 2]}`);
      return seen.size;
    };
    const counts = DETAIL_LEVELS.map((_, level) => countColors(level));
    for (let level = 1; level < counts.length; level += 1) {
      expect(counts[level]).toBeLessThanOrEqual(counts[level - 1]);
    }
    expect(counts[counts.length - 1]).toBeLessThan(counts[0]);
  });
});

describe("applyStampShape", () => {
  it("makes an oval with see-through corners and a solid middle", () => {
    const rgba = solid(16, 16, [10, 20, 30]);
    applyStampShape(rgba, 16, 16, "oval");
    expect(rgba[3]).toBe(0); // top-left corner
    expect(rgba[(15 * 16 + 15) * 4 + 3]).toBe(0); // bottom-right corner
    expect(rgba[(8 * 16 + 8) * 4 + 3]).toBe(255); // centre
    // Only fully solid or fully see-through pixels.
    for (let i = 0; i < 256; i += 1) expect([0, 255]).toContain(rgba[i * 4 + 3]);
  });

  it("keeps every pixel for a square", () => {
    const rgba = solid(4, 4, [1, 2, 3]);
    applyStampShape(rgba, 4, 4, "square");
    for (let i = 0; i < 16; i += 1) expect(rgba[i * 4 + 3]).toBe(255);
  });
});

describe("finding the boxer in a pose", () => {
  it("boxes everything that is not the backdrop and guesses the head", () => {
    const width = 64;
    const height = 64;
    const rgba = solid(width, height, [8, 8, 12]);
    for (let y = 10; y <= 49; y += 1) {
      for (let x = 20; x <= 39; x += 1) rgba.set([200, 150, 100, 255], (y * width + x) * 4);
    }
    const box = findFigureBox(rgba, width, height);
    expect(box).toEqual({ left: 20, top: 10, right: 39, bottom: 49 });

    const head = guessHeadPlacement(box!);
    expect(head.centerX).toBe(30);
    expect(head.size).toBe(12);
    expect(head.centerY).toBe(16);
  });

  it("returns nothing for an empty pose", () => {
    expect(findFigureBox(solid(8, 8, [8, 8, 12]), 8, 8)).toBeNull();
  });
});
