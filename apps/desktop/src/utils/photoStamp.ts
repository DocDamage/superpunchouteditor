/**
 * Helpers for putting a photo onto a boxer.
 *
 * Everything here is pure pixel maths so it can be tested without a browser
 * canvas.
 */

export interface CropView {
  /** 1 shows as much of the photo as fits; larger values zoom in. */
  zoom: number;
  /** Centre of the crop, as a 0–1 fraction of the photo's width and height. */
  centerX: number;
  centerY: number;
}

export interface CropRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

/**
 * The part of a photo to use, as a rectangle in photo pixels. The rectangle
 * always has the target's shape and always stays inside the photo, so the
 * result never has empty bars.
 */
export function cropRect(
  photoWidth: number,
  photoHeight: number,
  targetAspect: number,
  view: CropView,
): CropRect {
  const zoom = Math.max(1, view.zoom);
  let width = photoWidth;
  let height = photoWidth / targetAspect;
  if (height > photoHeight) {
    height = photoHeight;
    width = photoHeight * targetAspect;
  }
  width /= zoom;
  height /= zoom;

  const x = Math.min(Math.max(view.centerX * photoWidth - width / 2, 0), photoWidth - width);
  const y = Math.min(Math.max(view.centerY * photoHeight - height / 2, 0), photoHeight - height);
  return { x, y, width, height };
}

/**
 * How much to simplify a picture. The boxer's graphics are stored packed, and
 * busy pictures need more room than flat ones, so when a stamp does not fit
 * the editor retries with the next level.
 */
export const DETAIL_LEVELS = [
  { name: "full detail", shades: 0, blur: 0 },
  { name: "a little simpler", shades: 6, blur: 0 },
  { name: "simpler", shades: 4, blur: 1 },
  { name: "cartoon style", shades: 3, blur: 2 },
] as const;

/** Average each pixel with its neighbours, `passes` times. Alpha is kept. */
export function boxBlur(rgba: Uint8ClampedArray, width: number, height: number, passes: number): void {
  for (let pass = 0; pass < passes; pass += 1) {
    const source = new Uint8ClampedArray(rgba);
    for (let y = 0; y < height; y += 1) {
      for (let x = 0; x < width; x += 1) {
        let r = 0;
        let g = 0;
        let b = 0;
        let count = 0;
        for (let dy = -1; dy <= 1; dy += 1) {
          for (let dx = -1; dx <= 1; dx += 1) {
            const nx = x + dx;
            const ny = y + dy;
            if (nx < 0 || ny < 0 || nx >= width || ny >= height) continue;
            const offset = (ny * width + nx) * 4;
            r += source[offset];
            g += source[offset + 1];
            b += source[offset + 2];
            count += 1;
          }
        }
        const offset = (y * width + x) * 4;
        rgba[offset] = r / count;
        rgba[offset + 1] = g / count;
        rgba[offset + 2] = b / count;
      }
    }
  }
}

/** Reduce each colour channel to `shades` evenly spaced values. 0 or 1 leaves it alone. */
export function posterize(rgba: Uint8ClampedArray, shades: number): void {
  if (shades < 2) return;
  const step = 255 / (shades - 1);
  for (let offset = 0; offset < rgba.length; offset += 4) {
    rgba[offset] = Math.round(Math.round(rgba[offset] / step) * step);
    rgba[offset + 1] = Math.round(Math.round(rgba[offset + 1] / step) * step);
    rgba[offset + 2] = Math.round(Math.round(rgba[offset + 2] / step) * step);
  }
}

/** Simplify a picture in place to the given detail level. */
export function simplifyPicture(
  rgba: Uint8ClampedArray,
  width: number,
  height: number,
  level: number,
): void {
  const detail = DETAIL_LEVELS[Math.min(Math.max(level, 0), DETAIL_LEVELS.length - 1)];
  boxBlur(rgba, width, height, detail.blur);
  posterize(rgba, detail.shades);
}

/**
 * Cut a picture to an oval (or leave it square): pixels outside the shape
 * become fully see-through, pixels inside fully solid. The game has no
 * partly see-through pixels.
 */
export function applyStampShape(
  rgba: Uint8ClampedArray,
  width: number,
  height: number,
  shape: "oval" | "square",
): void {
  for (let y = 0; y < height; y += 1) {
    for (let x = 0; x < width; x += 1) {
      const offset = (y * width + x) * 4;
      let inside = true;
      if (shape === "oval") {
        const fx = (x + 0.5) / width - 0.5;
        const fy = (y + 0.5) / height - 0.5;
        inside = fx * fx + fy * fy <= 0.25;
      }
      rgba[offset + 3] = inside ? 255 : 0;
    }
  }
}

export interface Box {
  left: number;
  top: number;
  right: number;
  bottom: number;
}

/**
 * Find the boxer in a drawn pose: the box around every pixel that is not the
 * plain backdrop colour (taken from the top-left corner).
 */
export function findFigureBox(rgba: Uint8ClampedArray, width: number, height: number): Box | null {
  const [br, bg, bb] = [rgba[0], rgba[1], rgba[2]];
  let left = width;
  let top = height;
  let right = -1;
  let bottom = -1;
  for (let y = 0; y < height; y += 1) {
    for (let x = 0; x < width; x += 1) {
      const offset = (y * width + x) * 4;
      if (rgba[offset] !== br || rgba[offset + 1] !== bg || rgba[offset + 2] !== bb) {
        if (x < left) left = x;
        if (x > right) right = x;
        if (y < top) top = y;
        if (y > bottom) bottom = y;
      }
    }
  }
  return right < 0 ? null : { left, top, right, bottom };
}

/** A sensible first guess for where the head is: top middle of the boxer. */
export function guessHeadPlacement(box: Box): { centerX: number; centerY: number; size: number } {
  const height = box.bottom - box.top + 1;
  const size = Math.max(12, Math.round(height / 5));
  return {
    centerX: Math.round((box.left + box.right) / 2),
    centerY: box.top + Math.round(size / 2),
    size,
  };
}
