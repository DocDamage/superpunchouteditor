import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { useStore } from "../../store/useStore";
import type { AssetFile, BoxerRecord } from "../../store/useStore";
import { showToast } from "../ToastContainer";

interface PicturesPanelProps {
  boxer: BoxerRecord;
  /** Changes whenever an edit, undo or redo happens, so pictures redraw. */
  revisionKey: string;
}

type PictureKind = "icon" | "portrait" | "large_portrait";

interface Picture extends AssetFile {
  kind: PictureKind;
  index: number;
}

const KIND_INFO: Record<PictureKind, { title: string; where: string }> = {
  icon: { title: "Small face", where: "The little face shown in menus." },
  portrait: { title: "Portrait", where: "The picture shown before a fight." },
  large_portrait: { title: "Big portrait", where: "The large picture shown before a fight." },
};

const widthTilesFor = (picture: Picture): number => (picture.kind === "icon" ? 4 : 16);

const fileNameOf = (path: string): string => path.split(/[\\/]/).pop() ?? path;

function PictureCard({
  boxer,
  picture,
  revisionKey,
}: {
  boxer: BoxerRecord;
  picture: Picture;
  revisionKey: string;
}): React.ReactElement {
  const { exportAsset, importAsset, refreshUndoState, refreshPendingWrites } = useStore();
  const [imageUrl, setImageUrl] = useState<string | null>(null);
  // null while the first drawing is in progress.
  const [readable, setReadable] = useState<boolean | null>(null);
  const [busy, setBusy] = useState(false);
  const urlRef = useRef<string | null>(null);
  const palette = boxer.palette_files[0];
  const info = KIND_INFO[picture.kind];
  const title = picture.index > 0 ? `${info.title} ${picture.index + 1}` : info.title;
  const others = (picture.shared_with ?? []).filter(
    (name) => name.toLowerCase() !== boxer.name.toLowerCase(),
  );

  const draw = useCallback(async () => {
    if (!palette) return;
    try {
      const bytes = await invoke<number[]>("render_asset_preview", {
        pcOffset: picture.start_pc,
        size: picture.size,
        category: picture.category,
        palettePcOffset: palette.start_pc,
        paletteSize: palette.size,
        widthTiles: widthTilesFor(picture),
      });
      const url = URL.createObjectURL(new Blob([new Uint8Array(bytes)], { type: "image/png" }));
      if (urlRef.current) URL.revokeObjectURL(urlRef.current);
      urlRef.current = url;
      setImageUrl(url);
      setReadable(true);
    } catch (drawError) {
      // The game stores some pictures in a format the editor cannot read yet.
      console.warn("Could not draw picture preview:", drawError);
      setImageUrl(null);
      setReadable(false);
    }
  }, [palette, picture]);

  useEffect(() => {
    void draw();
  }, [draw, revisionKey]);

  useEffect(
    () => () => {
      if (urlRef.current) URL.revokeObjectURL(urlRef.current);
    },
    [],
  );

  const handleSave = async () => {
    if (!palette) return;
    const path = await save({
      filters: [{ name: "PNG picture", extensions: ["png"] }],
      defaultPath: `${boxer.name} ${title}.png`,
    });
    if (!path) return;
    setBusy(true);
    try {
      await exportAsset(picture, palette, path);
      if (!useStore.getState().error) {
        showToast(`Saved ${fileNameOf(path)}. Open it in any drawing app.`, "success");
      }
    } finally {
      setBusy(false);
    }
  };

  const handleUseMine = async () => {
    if (!palette) return;
    if (others.length > 0) {
      const confirmed = window.confirm(
        `${others.join(" and ")} use${others.length === 1 ? "s" : ""} this same picture.\n\n` +
          "Changing it here changes it for them too. Keep going?",
      );
      if (!confirmed) return;
    }

    const path = await open({
      multiple: false,
      filters: [{ name: "Pictures", extensions: ["png", "jpg", "jpeg", "bmp", "gif", "webp"] }],
    });
    if (typeof path !== "string") return;

    setBusy(true);
    try {
      const result = await importAsset(picture, palette, path);
      if (!result) {
        const reason = useStore.getState().error;
        showToast(reason ?? "That picture could not be used. Try a different one.", "error", 8000);
        useStore.getState().setError(null);
        return;
      }
      await Promise.all([refreshUndoState(), refreshPendingWrites()]);
      const [targetWidth, targetHeight] = result.targetSize;
      showToast(
        result.autoResized
          ? `Done! Your picture was fitted to ${targetWidth}×${targetHeight} and matched to ${boxer.name}'s colors.`
          : `Done! Your picture was matched to ${boxer.name}'s colors.`,
        "success",
        6000,
      );
    } finally {
      setBusy(false);
    }
  };

  return (
    <article className={`picture-card ${readable === false ? "unsupported" : ""}`}>
      <div className="picture-card-frame">
        {imageUrl ? (
          <img src={imageUrl} alt={`${boxer.name} ${title}`} />
        ) : (
          <span className="picture-card-missing">{readable === false ? "Not ready yet" : "Drawing…"}</span>
        )}
      </div>
      <div className="picture-card-body">
        <h3>{title}</h3>
        <p>{info.where}</p>
        {others.length > 0 && (
          <p className="picture-card-shared">Also used by {others.join(", ")}.</p>
        )}
        {readable === false ? (
          <p className="picture-card-shared">
            The editor cannot open this picture yet, so it cannot be changed here.
          </p>
        ) : (
          <div className="picture-card-actions">
            <button
              type="button"
              className="btn-primary"
              onClick={() => void handleUseMine()}
              disabled={busy || !palette || readable !== true}
            >
              Use My Picture
            </button>
            <button
              type="button"
              className="secondary"
              onClick={() => void handleSave()}
              disabled={busy || !palette || readable !== true}
            >
              Save as File
            </button>
          </div>
        )}
      </div>
    </article>
  );
}

export function PicturesPanel({ boxer, revisionKey }: PicturesPanelProps): React.ReactElement {
  const pictures: Picture[] = [
    ...(boxer.icon_files ?? []).map((asset, index) => ({ ...asset, kind: "icon" as const, index })),
    ...(boxer.portrait_files ?? []).map((asset, index) => ({ ...asset, kind: "portrait" as const, index })),
    ...(boxer.large_portrait_files ?? []).map((asset, index) => ({
      ...asset,
      kind: "large_portrait" as const,
      index,
    })),
  ];

  if (pictures.length === 0) {
    return (
      <div className="workshop-empty">
        <h3>No pictures to swap</h3>
        <p>{boxer.name} does not have a face picture the editor can change yet.</p>
      </div>
    );
  }

  return (
    <div className="pictures-panel">
      <div className="workshop-tip">
        <strong>Any picture works.</strong> Pick a photo or drawing of any size. The editor fits it to the right
        size, trims the edges if it is too wide or tall, and swaps its colors for the closest ones {boxer.name}{" "}
        already uses. Simple pictures with few colors look best.
      </div>
      <div className="picture-grid">
        {pictures.map((picture) => (
          <PictureCard
            key={`${picture.kind}-${picture.start_pc}`}
            boxer={boxer}
            picture={picture}
            revisionKey={revisionKey}
          />
        ))}
      </div>
    </div>
  );
}
