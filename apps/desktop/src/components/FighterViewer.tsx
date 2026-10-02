import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { sortBoxersInGameOrder, useStore } from '../store/useStore';
import './FighterViewer.css';

/** Poses per second while flipping through a boxer's poses. */
const FLIP_SPEED = 5;

/**
 * Look Around: browse every boxer and every pose. Nothing here changes the
 * game; it is a safe place to explore.
 */
export const FighterViewer: React.FC = () => {
  const { fighters, loadFighterList, selectedFighterId, selectFighter, poses, renderPose } = useStore();

  const [poseIndex, setPoseIndex] = useState(0);
  const [imageSrc, setImageSrc] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [failed, setFailed] = useState(false);
  const [flipping, setFlipping] = useState(false);
  const urlRef = useRef<string | null>(null);
  const requestRef = useRef(0);
  const poseIndexRef = useRef(0);

  const orderedFighters = useMemo(() => sortBoxersInGameOrder(fighters), [fighters]);
  const selectedFighter = orderedFighters.find((fighter) => fighter.id === selectedFighterId) ?? null;

  useEffect(() => {
    void loadFighterList();
  }, [loadFighterList]);

  // Start with the first boxer so the page is never empty.
  useEffect(() => {
    if (selectedFighterId === null && orderedFighters.length > 0) {
      void selectFighter(orderedFighters[0].id);
    }
  }, [orderedFighters, selectFighter, selectedFighterId]);

  const showPose = useCallback(
    async (fighterId: number, index: number, quiet = false) => {
      const request = ++requestRef.current;
      if (!quiet) setLoading(true);
      try {
        const bytes = await renderPose(fighterId, index);
        if (request !== requestRef.current) return;
        const url = URL.createObjectURL(new Blob([new Uint8Array(bytes)], { type: 'image/png' }));
        if (urlRef.current) URL.revokeObjectURL(urlRef.current);
        urlRef.current = url;
        setImageSrc(url);
        setPoseIndex(index);
        poseIndexRef.current = index;
        setFailed(false);
      } catch (poseError) {
        if (request !== requestRef.current) return;
        console.error('Could not draw pose:', poseError);
        // The store records the error for its banner; this page shows its own message.
        useStore.getState().setError(null);
        setFailed(true);
        setFlipping(false);
      } finally {
        if (request === requestRef.current) setLoading(false);
      }
    },
    [renderPose],
  );

  // Show the first pose whenever another boxer is picked.
  useEffect(() => {
    setFlipping(false);
    if (selectedFighterId !== null && poses.length > 0) {
      void showPose(selectedFighterId, 0);
    } else {
      setImageSrc(null);
    }
  }, [poses, selectedFighterId, showPose]);

  // Flip through the poses like a flip book.
  useEffect(() => {
    if (!flipping || selectedFighterId === null || poses.length < 2) return;
    const timer = window.setInterval(() => {
      const next = (poseIndexRef.current + 1) % poses.length;
      void showPose(selectedFighterId, next, true);
    }, 1000 / FLIP_SPEED);
    return () => window.clearInterval(timer);
  }, [flipping, poses.length, selectedFighterId, showPose]);

  useEffect(
    () => () => {
      if (urlRef.current) URL.revokeObjectURL(urlRef.current);
    },
    [],
  );

  const go = (index: number) => {
    if (selectedFighterId === null || index < 0 || index >= poses.length) return;
    setFlipping(false);
    void showPose(selectedFighterId, index);
  };

  return (
    <div className="workspace-page look-page">
      <header className="look-header">
        <p className="eyebrow">Look Around</p>
        <h2>See every boxer and pose</h2>
        <p>Explore as much as you like. Nothing on this page changes your game.</p>
      </header>

      <div className="look-boxers" role="group" aria-label="Choose a boxer to look at">
        {orderedFighters.map((fighter) => (
          <button
            key={fighter.id}
            type="button"
            className={`look-boxer ${fighter.id === selectedFighterId ? 'active' : ''}`}
            aria-pressed={fighter.id === selectedFighterId}
            onClick={() => void selectFighter(fighter.id)}
          >
            {fighter.name}
          </button>
        ))}
      </div>

      <section className="look-stage">
        <div className="look-stage-top">
          <h3>{selectedFighter ? selectedFighter.name : 'Pick a boxer'}</h3>
          {poses.length > 0 && (
            <span className="look-count">
              Pose {poseIndex + 1} of {poses.length}
            </span>
          )}
        </div>

        <div className="look-canvas" aria-busy={loading}>
          {imageSrc && !failed && (
            <img
              src={imageSrc}
              alt={selectedFighter ? `${selectedFighter.name} pose ${poseIndex + 1}` : 'Boxer pose'}
              width={512}
              height={512}
            />
          )}
          {!imageSrc && !failed && (
            <span className="look-note">{loading ? 'Drawing the boxer…' : 'Pick a boxer above to see their poses.'}</span>
          )}
          {failed && <span className="look-note">This pose could not be drawn. Try another one.</span>}
        </div>

        {poses.length > 0 && (
          <div className="look-controls">
            <button type="button" className="secondary" onClick={() => go(poseIndex - 1)} disabled={poseIndex <= 0}>
              ← Back
            </button>
            <input
              type="range"
              min={0}
              max={poses.length - 1}
              value={poseIndex}
              aria-label="Pose"
              onChange={(event) => go(Number(event.target.value))}
            />
            <button
              type="button"
              className="secondary"
              onClick={() => go(poseIndex + 1)}
              disabled={poseIndex >= poses.length - 1}
            >
              Next →
            </button>
            <button
              type="button"
              className="btn-primary"
              onClick={() => setFlipping((value) => !value)}
              disabled={poses.length < 2}
            >
              {flipping ? 'Stop' : 'Flip Through'}
            </button>
          </div>
        )}
      </section>
    </div>
  );
};
