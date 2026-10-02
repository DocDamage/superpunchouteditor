import { useCallback, useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useStore } from '../store/useStore';
import type { BoxerRecord, FighterMetadata, PoseInfo } from '../store/useStore';

interface AssembledPosePreviewProps {
  boxer: BoxerRecord;
  /** When this value changes, the current pose is redrawn from the latest edits. */
  refreshKey?: string | number;
  /** Plain-language labels for the guided boxer workshop. */
  friendly?: boolean;
}

/**
 * Shows a complete in-game pose assembled from the fighter's tile banks and
 * metasprite/OAM data. This is deliberately separate from the raw tile-bank
 * reference sheet below it.
 */
export const AssembledPosePreview = ({ boxer, refreshKey, friendly = false }: AssembledPosePreviewProps) => {
  const { romSha1 } = useStore();
  const [fighterId, setFighterId] = useState<number | null>(null);
  const [poses, setPoses] = useState<PoseInfo[]>([]);
  const [poseIndex, setPoseIndex] = useState(0);
  const [imageSrc, setImageSrc] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const imageUrlRef = useRef<string | null>(null);
  const requestIdRef = useRef(0);
  const poseIndexRef = useRef(0);

  const renderPose = useCallback(async (id: number, index: number, quiet = false) => {
    const requestId = ++requestIdRef.current;
    // A quiet redraw keeps the current picture on screen until the new one is ready.
    if (!quiet) setLoading(true);
    setError(null);

    try {
      const bytes = await invoke<number[]>('render_fighter_pose', {
        fighterId: id,
        poseId: index,
      });

      if (requestId !== requestIdRef.current) return;

      if (imageUrlRef.current) {
        URL.revokeObjectURL(imageUrlRef.current);
      }
      const blob = new Blob([new Uint8Array(bytes)], { type: 'image/png' });
      const url = URL.createObjectURL(blob);
      imageUrlRef.current = url;
      setImageSrc(url);
      setPoseIndex(index);
      poseIndexRef.current = index;
    } catch (renderError) {
      if (requestId === requestIdRef.current) {
        setError(String(renderError));
        if (imageUrlRef.current) {
          URL.revokeObjectURL(imageUrlRef.current);
          imageUrlRef.current = null;
        }
        setImageSrc(null);
      }
    } finally {
      if (requestId === requestIdRef.current) {
        setLoading(false);
      }
    }
  }, []);

  useEffect(() => {
    let cancelled = false;
    const requestId = ++requestIdRef.current;

    setFighterId(null);
    setPoses([]);
    setPoseIndex(0);
    if (imageUrlRef.current) {
      URL.revokeObjectURL(imageUrlRef.current);
      imageUrlRef.current = null;
    }
    setImageSrc(null);
    setError(null);

    if (!romSha1) {
      return () => {
        cancelled = true;
        if (requestIdRef.current === requestId) requestIdRef.current += 1;
      };
    }

    const load = async () => {
      try {
        const fighters = await invoke<FighterMetadata[]>('get_fighter_list');
        const fighter = fighters.find(
          candidate => candidate.name.toLowerCase() === boxer.name.toLowerCase(),
        );

        if (!fighter) {
          throw new Error(`No ROM fighter entry found for ${boxer.name}`);
        }

        const poseList = await invoke<PoseInfo[]>('get_fighter_poses', {
          fighterId: fighter.id,
        });

        if (cancelled) return;
        setFighterId(fighter.id);
        setPoses(poseList);

        if (poseList.length > 0) {
          await renderPose(fighter.id, 0);
        }
      } catch (loadError) {
        if (!cancelled) {
          setError(String(loadError));
          setLoading(false);
        }
      }
    };

    void load();

    return () => {
      cancelled = true;
      if (requestIdRef.current === requestId) requestIdRef.current += 1;
    };
  }, [boxer.name, renderPose, romSha1]);

  useEffect(() => {
    return () => {
      if (imageUrlRef.current) {
        URL.revokeObjectURL(imageUrlRef.current);
      }
    };
  }, []);

  // Redraw the pose in place after an edit, undo or redo.
  const firstRefreshRef = useRef(true);
  useEffect(() => {
    if (firstRefreshRef.current) {
      firstRefreshRef.current = false;
      return;
    }
    if (fighterId === null || refreshKey === undefined) return;
    void renderPose(fighterId, poseIndexRef.current, true);
    // Only the refresh key should trigger a redraw here.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [refreshKey]);

  const changePose = (nextIndex: number) => {
    if (fighterId === null || nextIndex < 0 || nextIndex >= poses.length) return;
    void renderPose(fighterId, nextIndex);
  };

  return (
    <div
      className="pose-preview"
      style={friendly ? undefined : {
        paddingBottom: '1.5rem',
        borderBottom: '1px solid var(--border)',
      }}
    >
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'flex-start',
          gap: '1rem',
          flexWrap: 'wrap',
          marginBottom: '0.75rem',
        }}
      >
        <div>
          <h3 style={{ margin: 0 }}>{friendly ? 'Preview' : 'Assembled Pose Preview'}</h3>
          <p style={{ margin: '4px 0 0', color: 'var(--text-dim)', fontSize: '0.85rem' }}>
            {friendly
              ? `This is how ${boxer.name} looks in the game. It updates as you make changes.`
              : 'Uses the game\u2019s pose data to place the 8×8 tiles into a complete boxer image.'}
          </p>
        </div>

        {poses.length > 0 && (
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', flexWrap: 'wrap' }}>
            <button
              type="button"
              onClick={() => changePose(poseIndex - 1)}
              disabled={loading || poseIndex <= 0}
              style={{ padding: '5px 10px' }}
            >
              {friendly ? '← Back' : '← Prev'}
            </button>
            <select
              aria-label={`${boxer.name} pose`}
              value={poseIndex}
              onChange={event => changePose(Number(event.target.value))}
              disabled={loading}
              style={{ padding: '5px 8px', borderRadius: '4px', background: 'var(--glass)' }}
            >
              {poses.map((pose, index) => (
                <option key={pose.index ?? index} value={index}>
                  {friendly
                    ? `Pose ${index + 1} of ${poses.length}`
                    : `Pose ${index} · $${pose.data_addr.toString(16).toUpperCase().padStart(4, '0')}`}
                </option>
              ))}
            </select>
            <button
              type="button"
              onClick={() => changePose(poseIndex + 1)}
              disabled={loading || poseIndex >= poses.length - 1}
              style={{ padding: '5px 10px' }}
            >
              Next →
            </button>
          </div>
        )}
      </div>

      <div
        style={{
          minHeight: '280px',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          background: 'var(--canvas-bg)',
          border: '1px solid var(--border)',
          borderRadius: '10px',
          padding: '1rem',
          position: 'relative',
          overflow: 'auto',
        }}
      >
        {loading && (
          <div style={{ color: 'var(--text-dim)', fontSize: '0.875rem' }}>
            {friendly ? 'Drawing the boxer…' : 'Assembling pose…'}
          </div>
        )}

        {!loading && imageSrc && (
          <img
            src={imageSrc}
            alt={`${boxer.name} assembled pose ${poseIndex}`}
            width={friendly ? 384 : 512}
            height={friendly ? 384 : 512}
            style={{ imageRendering: 'pixelated', display: 'block', maxWidth: '100%', height: 'auto' }}
          />
        )}

        {!loading && !imageSrc && !error && (
          <div style={{ color: 'var(--text-dim)', fontSize: '0.875rem' }}>
            No pose data found for this boxer.
          </div>
        )}

        {!loading && error && (
          <div style={{ color: 'var(--error)', fontSize: '0.875rem', textAlign: 'center' }}>
            {friendly ? 'This pose could not be drawn' : 'Could not assemble this pose'}: {error}
          </div>
        )}
      </div>
    </div>
  );
};
