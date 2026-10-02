import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useStore } from "../../store/useStore";
import type { BoxerRecord, FighterMetadata, PoseInfo } from "../../store/useStore";
import { showToast } from "../ToastContainer";
import {
  DETAIL_LEVELS,
  applyStampShape,
  cropRect,
  findFigureBox,
  guessHeadPlacement,
  simplifyPicture,
} from "../../utils/photoStamp";

interface FaceStampProps {
  boxer: BoxerRecord;
  /** Changes whenever an edit, undo or redo happens, so the pose redraws. */
  revisionKey: string;
}

interface StampOutcome {
  pixels_written: number;
  pixels_skipped: number;
  tiles_changed: number;
  other_poses_changed: number[];
  also_affects: string[];
}

/** The pose canvas the game data is drawn on, in game pixels. */
const POSE_SIZE = 256;
/** How much the pose is enlarged on screen. */
const VIEW_SCALE = 2;
const TOO_DETAILED = "STAMP_TOO_DETAILED";

type Shape = "oval" | "square";

/**
 * Put a photo on the boxer as they appear in the ring. The user places the
 * photo over the boxer's head on a real pose; the editor writes it into the
 * boxer's graphics in that boxer's colours.
 */
export function FaceStamp({ boxer, revisionKey }: FaceStampProps): React.ReactElement {
  const { romSha1, refreshUndoState, refreshPendingWrites } = useStore();

  const [fighterId, setFighterId] = useState<number | null>(null);
  const [poseCount, setPoseCount] = useState(0);
  const [poseIndex, setPoseIndex] = useState(0);
  const [poseBitmap, setPoseBitmap] = useState<ImageBitmap | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);

  const [photo, setPhoto] = useState<HTMLImageElement | null>(null);
  const [photoName, setPhotoName] = useState("");
  const [zoom, setZoom] = useState(1);
  const [panX, setPanX] = useState(0.5);
  const [panY, setPanY] = useState(0.4);

  const [placement, setPlacement] = useState({ centerX: 128, centerY: 80, size: 32 });
  const [shape, setShape] = useState<Shape>("oval");
  const [busy, setBusy] = useState(false);
  const [result, setResult] = useState<{ kind: "ok" | "error"; text: string } | null>(null);

  const canvasRef = useRef<HTMLCanvasElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const dragRef = useRef<{ dx: number; dy: number } | null>(null);
  const photoUrlRef = useRef<string | null>(null);
  const autoPlacedRef = useRef<string | null>(null);

  // Find this boxer and how many poses they have.
  useEffect(() => {
    let cancelled = false;
    setFighterId(null);
    setPoseCount(0);
    setPoseIndex(0);
    setPoseBitmap(null);
    setLoadError(null);
    setResult(null);
    autoPlacedRef.current = null;
    if (!romSha1) return;

    void (async () => {
      try {
        const fighters = await invoke<FighterMetadata[]>("get_fighter_list");
        const fighter = fighters.find((candidate) => candidate.name.toLowerCase() === boxer.name.toLowerCase());
        if (!fighter) throw new Error(`${boxer.name} has no in-ring pictures the editor can read.`);
        const poses = await invoke<PoseInfo[]>("get_fighter_poses", { fighterId: fighter.id });
        if (cancelled) return;
        setFighterId(fighter.id);
        setPoseCount(poses.length);
      } catch (error) {
        if (!cancelled) setLoadError(String(error));
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [boxer.name, romSha1]);

  // Draw the current pose (and redraw it after every edit).
  useEffect(() => {
    if (fighterId === null || poseCount === 0) return;
    let cancelled = false;
    void (async () => {
      try {
        const bytes = await invoke<number[]>("render_fighter_pose", { fighterId, poseId: poseIndex });
        const bitmap = await createImageBitmap(new Blob([new Uint8Array(bytes)], { type: "image/png" }));
        if (cancelled) {
          bitmap.close();
          return;
        }
        setPoseBitmap((previous) => {
          previous?.close();
          return bitmap;
        });
        setLoadError(null);
      } catch (error) {
        if (!cancelled) setLoadError(`This pose could not be drawn. ${error}`);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [fighterId, poseCount, poseIndex, revisionKey]);

  // The first time a pose appears, put the stamp over the boxer's head.
  useEffect(() => {
    if (!poseBitmap) return;
    const key = `${boxer.key}:${poseIndex}`;
    if (autoPlacedRef.current === key) return;
    autoPlacedRef.current = key;

    const scratch = document.createElement("canvas");
    scratch.width = POSE_SIZE;
    scratch.height = POSE_SIZE;
    const ctx = scratch.getContext("2d");
    if (!ctx) return;
    ctx.drawImage(poseBitmap, 0, 0);
    const box = findFigureBox(ctx.getImageData(0, 0, POSE_SIZE, POSE_SIZE).data, POSE_SIZE, POSE_SIZE);
    if (box) setPlacement(guessHeadPlacement(box));
  }, [boxer.key, poseBitmap, poseIndex]);

  /** The photo, cropped and sized for the stamp, as raw pixels. */
  const buildStampPixels = useCallback(
    (detailLevel: number): ImageData | null => {
      if (!photo) return null;
      const size = Math.max(4, Math.round(placement.size));
      const scratch = document.createElement("canvas");
      scratch.width = size;
      scratch.height = size;
      const ctx = scratch.getContext("2d");
      if (!ctx) return null;
      const crop = cropRect(photo.naturalWidth, photo.naturalHeight, 1, { zoom, centerX: panX, centerY: panY });
      ctx.imageSmoothingEnabled = true;
      ctx.imageSmoothingQuality = "high";
      ctx.drawImage(photo, crop.x, crop.y, crop.width, crop.height, 0, 0, size, size);
      const pixels = ctx.getImageData(0, 0, size, size);
      simplifyPicture(pixels.data, size, size, detailLevel);
      applyStampShape(pixels.data, size, size, shape);
      return pixels;
    },
    [panX, panY, photo, placement.size, shape, zoom],
  );

  const stampOrigin = useMemo(() => {
    const size = Math.max(4, Math.round(placement.size));
    return {
      size,
      left: Math.round(placement.centerX - size / 2),
      top: Math.round(placement.centerY - size / 2),
    };
  }, [placement]);

  // Paint the pose with the photo on top.
  useEffect(() => {
    const canvas = canvasRef.current;
    const ctx = canvas?.getContext("2d");
    if (!canvas || !ctx) return;
    ctx.imageSmoothingEnabled = false;
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    if (poseBitmap) ctx.drawImage(poseBitmap, 0, 0, canvas.width, canvas.height);

    const pixels = buildStampPixels(0);
    if (pixels) {
      const scratch = document.createElement("canvas");
      scratch.width = pixels.width;
      scratch.height = pixels.height;
      scratch.getContext("2d")?.putImageData(pixels, 0, 0);
      ctx.drawImage(
        scratch,
        stampOrigin.left * VIEW_SCALE,
        stampOrigin.top * VIEW_SCALE,
        stampOrigin.size * VIEW_SCALE,
        stampOrigin.size * VIEW_SCALE,
      );
    }

    // A dashed outline shows where the photo goes, even before one is chosen.
    ctx.save();
    ctx.setLineDash([6, 4]);
    ctx.lineWidth = 2;
    ctx.strokeStyle = "#ffffff";
    ctx.beginPath();
    const cx = (stampOrigin.left + stampOrigin.size / 2) * VIEW_SCALE;
    const cy = (stampOrigin.top + stampOrigin.size / 2) * VIEW_SCALE;
    const radius = (stampOrigin.size / 2) * VIEW_SCALE;
    if (shape === "oval") ctx.arc(cx, cy, radius, 0, Math.PI * 2);
    else ctx.rect(cx - radius, cy - radius, radius * 2, radius * 2);
    ctx.stroke();
    ctx.restore();
  }, [buildStampPixels, poseBitmap, shape, stampOrigin]);

  useEffect(
    () => () => {
      if (photoUrlRef.current) URL.revokeObjectURL(photoUrlRef.current);
    },
    [],
  );

  const handlePhotoChosen = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    event.target.value = "";
    if (!file) return;
    const url = URL.createObjectURL(file);
    const image = new Image();
    image.onload = () => {
      if (photoUrlRef.current) URL.revokeObjectURL(photoUrlRef.current);
      photoUrlRef.current = url;
      setPhoto(image);
      setPhotoName(file.name);
      setZoom(1);
      setPanX(0.5);
      setPanY(0.4);
      setResult(null);
    };
    image.onerror = () => {
      URL.revokeObjectURL(url);
      showToast("That file is not a picture the editor can open. Try a PNG or JPG.", "error");
    };
    image.src = url;
  };

  const toPose = (event: React.PointerEvent<HTMLCanvasElement>) => {
    const rect = event.currentTarget.getBoundingClientRect();
    return {
      x: ((event.clientX - rect.left) / rect.width) * POSE_SIZE,
      y: ((event.clientY - rect.top) / rect.height) * POSE_SIZE,
    };
  };

  const handlePointerDown = (event: React.PointerEvent<HTMLCanvasElement>) => {
    const point = toPose(event);
    dragRef.current = { dx: point.x - placement.centerX, dy: point.y - placement.centerY };
    // Clicking away from the stamp jumps it there.
    if (Math.hypot(dragRef.current.dx, dragRef.current.dy) > placement.size / 2) {
      dragRef.current = { dx: 0, dy: 0 };
      setPlacement((current) => ({ ...current, centerX: point.x, centerY: point.y }));
    }
    event.currentTarget.setPointerCapture(event.pointerId);
  };

  const handlePointerMove = (event: React.PointerEvent<HTMLCanvasElement>) => {
    const drag = dragRef.current;
    if (!drag) return;
    const point = toPose(event);
    setPlacement((current) => ({
      ...current,
      centerX: Math.min(POSE_SIZE, Math.max(0, point.x - drag.dx)),
      centerY: Math.min(POSE_SIZE, Math.max(0, point.y - drag.dy)),
    }));
  };

  const handlePointerUp = () => {
    dragRef.current = null;
  };

  /** Build the game-sized overlay: see-through everywhere except the stamp. */
  const buildOverlayPng = async (detailLevel: number): Promise<number[] | null> => {
    const pixels = buildStampPixels(detailLevel);
    if (!pixels) return null;
    const overlay = document.createElement("canvas");
    overlay.width = POSE_SIZE;
    overlay.height = POSE_SIZE;
    const ctx = overlay.getContext("2d");
    if (!ctx) return null;
    ctx.putImageData(pixels, stampOrigin.left, stampOrigin.top);
    const blob = await new Promise<Blob | null>((resolve) => overlay.toBlob(resolve, "image/png"));
    if (!blob) return null;
    return Array.from(new Uint8Array(await blob.arrayBuffer()));
  };

  const handleStamp = async () => {
    if (fighterId === null || !photo) return;
    setBusy(true);
    setResult(null);
    try {
      let lastError = "";
      for (let level = 0; level < DETAIL_LEVELS.length; level += 1) {
        const overlayPng = await buildOverlayPng(level);
        if (!overlayPng) throw new Error("The photo could not be prepared.");
        try {
          const outcome = await invoke<StampOutcome>("stamp_photo_on_pose", {
            fighterId,
            poseId: poseIndex,
            overlayPng,
          });
          await Promise.all([refreshUndoState(), refreshPendingWrites()]);

          const parts = [`Your photo is on ${boxer.name}!`];
          if (level > 0) {
            parts.push(`It was made ${DETAIL_LEVELS[level].name} so it fits in the game.`);
          }
          const others = outcome.other_poses_changed.length;
          if (others > 0) {
            parts.push(`${others} other pose${others === 1 ? "" : "s"} use the same head, so they changed too.`);
          }
          if (outcome.also_affects.length > 0) {
            parts.push(`${outcome.also_affects.join(" and ")} share${outcome.also_affects.length === 1 ? "s" : ""} these pictures and changed as well.`);
          }
          parts.push("Not happy? Press Undo in the sidebar.");
          setResult({ kind: "ok", text: parts.join(" ") });
          return;
        } catch (stampError) {
          lastError = String(stampError);
          if (!lastError.includes(TOO_DETAILED)) throw stampError;
          // Too much detail for the space available: try a simpler version.
        }
      }
      setResult({
        kind: "error",
        text:
          "This photo has too much detail to fit in this boxer's pictures, even in cartoon style. " +
          "Try making the stamp smaller, or use a simpler picture with big flat areas.",
      });
    } catch (error) {
      setResult({ kind: "error", text: String(error).replace(`${TOO_DETAILED}: `, "") });
    } finally {
      setBusy(false);
    }
  };

  if (loadError && fighterId === null) {
    return (
      <div className="workshop-empty">
        <h3>Photos cannot go on this boxer yet</h3>
        <p>{loadError}</p>
      </div>
    );
  }

  return (
    <div className="face-stamp">
      <div className="face-stamp-stage">
        <div className="face-stamp-poses">
          <button
            type="button"
            className="secondary"
            onClick={() => setPoseIndex((index) => Math.max(0, index - 1))}
            disabled={busy || poseIndex <= 0}
          >
            ← Back
          </button>
          <span>
            Pose {poseCount === 0 ? 0 : poseIndex + 1} of {poseCount}
          </span>
          <button
            type="button"
            className="secondary"
            onClick={() => setPoseIndex((index) => Math.min(poseCount - 1, index + 1))}
            disabled={busy || poseIndex >= poseCount - 1}
          >
            Next →
          </button>
        </div>
        <canvas
          ref={canvasRef}
          className="face-stamp-canvas"
          width={POSE_SIZE * VIEW_SCALE}
          height={POSE_SIZE * VIEW_SCALE}
          role="img"
          aria-label={`${boxer.name} pose ${poseIndex + 1} with the photo placed on it`}
          onPointerDown={handlePointerDown}
          onPointerMove={handlePointerMove}
          onPointerUp={handlePointerUp}
          onPointerCancel={handlePointerUp}
        />
        <p className="face-stamp-hint">Drag the dashed shape onto the boxer&apos;s head.</p>
      </div>

      <div className="face-stamp-tools">
        <section className="magic-card">
          <header>
            <div>
              <h3>1. Pick a photo</h3>
              <p>A face photo works best. It stays on this computer.</p>
            </div>
          </header>
          <input
            ref={fileInputRef}
            type="file"
            accept="image/png,image/jpeg,image/webp,image/bmp,image/gif"
            className="sr-only"
            onChange={handlePhotoChosen}
            aria-label="Choose a photo"
          />
          <button type="button" className="btn-primary" onClick={() => fileInputRef.current?.click()} disabled={busy}>
            {photo ? "Pick a Different Photo" : "Pick a Photo"}
          </button>
          {photoName && <p className="face-stamp-file">{photoName}</p>}
        </section>

        <section className="magic-card">
          <header>
            <div>
              <h3>2. Line it up</h3>
              <p>Drag it on the picture, then fine-tune here.</p>
            </div>
          </header>
          <label className="face-stamp-slider">
            Size on the boxer
            <input
              type="range"
              min={10}
              max={96}
              value={placement.size}
              onChange={(event) => setPlacement((current) => ({ ...current, size: Number(event.target.value) }))}
            />
          </label>
          <label className="face-stamp-slider">
            Zoom into the photo
            <input
              type="range"
              min={1}
              max={5}
              step={0.05}
              value={zoom}
              disabled={!photo}
              onChange={(event) => setZoom(Number(event.target.value))}
            />
          </label>
          <label className="face-stamp-slider">
            Slide photo left / right
            <input
              type="range"
              min={0}
              max={1}
              step={0.01}
              value={panX}
              disabled={!photo}
              onChange={(event) => setPanX(Number(event.target.value))}
            />
          </label>
          <label className="face-stamp-slider">
            Slide photo up / down
            <input
              type="range"
              min={0}
              max={1}
              step={0.01}
              value={panY}
              disabled={!photo}
              onChange={(event) => setPanY(Number(event.target.value))}
            />
          </label>
          <div className="face-stamp-shapes" role="radiogroup" aria-label="Photo shape">
            {(["oval", "square"] as const).map((option) => (
              <button
                key={option}
                type="button"
                role="radio"
                aria-checked={shape === option}
                className={shape === option ? "btn-primary" : "secondary"}
                onClick={() => setShape(option)}
              >
                {option === "oval" ? "Round" : "Square"}
              </button>
            ))}
          </div>
        </section>

        <section className="magic-card">
          <header>
            <div>
              <h3>3. Stamp it</h3>
              <p>The game redraws your photo using {boxer.name}&apos;s own colors.</p>
            </div>
          </header>
          <button
            type="button"
            className="btn-primary face-stamp-go"
            onClick={() => void handleStamp()}
            disabled={busy || !photo || fighterId === null}
          >
            {busy ? "Stamping…" : "Stamp It!"}
          </button>
          {result && (
            <div className={`save-status ${result.kind === "error" ? "error" : ""}`} role="status">
              {result.text}
            </div>
          )}
        </section>
      </div>

      <div className="workshop-tip face-stamp-note">
        <strong>Good to know.</strong> A boxer uses different head pictures for different moves. One stamp covers
        every pose that shares the same head. Flip through the poses and stamp again on any that still show the
        old face. The big picture shown before a fight cannot be changed yet.
      </div>
    </div>
  );
}
