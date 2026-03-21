/**
 * EmulatorCanvas Component
 * 
 * Renders the emulator output to a canvas with proper scaling and aspect ratio.
 * Handles resize events and provides pixel-perfect rendering options.
 */

import React, { useRef, useEffect, useCallback, useState } from 'react';

export interface EmulatorCanvasProps {
  /** Canvas width in pixels (SNES resolution: 256) */
  width?: number;
  /** Canvas height in pixels (SNES resolution: 224) */
  height?: number;
  /** Maximum scale factor for integer scaling */
  maxScale?: number;
  /** Whether to use integer scaling */
  integerScaling?: boolean;
  /** Scaling mode */
  scalingMode?: 'pixel-perfect' | 'smooth' | 'stretch';
  /** Whether to show scanlines effect */
  showScanlines?: boolean;
  /** Whether to show CRT effect */
  showCrtEffect?: boolean;
  /** Callback when canvas is ready */
  onCanvasReady?: (canvas: HTMLCanvasElement) => void;
  /** Optional className */
  className?: string;
  /** Optional style */
  style?: React.CSSProperties;
  /** Optional external canvas ref */
  canvasRef?: React.RefObject<HTMLCanvasElement | null>;
}

// SNES resolution constants
const SNES_WIDTH = 256;
const SNES_HEIGHT = 224;
const SNES_ASPECT_RATIO = SNES_WIDTH / SNES_HEIGHT;

export const EmulatorCanvas: React.FC<EmulatorCanvasProps> = ({
  width = SNES_WIDTH,
  height = SNES_HEIGHT,
  maxScale = 4,
  integerScaling = true,
  scalingMode = 'pixel-perfect',
  showScanlines = false,
  showCrtEffect = false,
  onCanvasReady,
  className = '',
  style = {},
  canvasRef: externalCanvasRef,
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const internalCanvasRef = useRef<HTMLCanvasElement>(null);
  const canvasRef = externalCanvasRef ?? internalCanvasRef;
  const [scale, setScale] = useState(1);
  const [containerSize, setContainerSize] = useState({ width: 0, height: 0 });

  // Calculate optimal scale based on container size
  const calculateScale = useCallback((containerWidth: number, containerHeight: number): number => {
    const availableWidth = containerWidth;
    const availableHeight = containerHeight;
    
    // Calculate scale based on aspect ratio
    const scaleX = availableWidth / width;
    const scaleY = availableHeight / height;
    let newScale = Math.min(scaleX, scaleY);
    
    if (integerScaling) {
      // Round down to nearest integer
      newScale = Math.max(1, Math.floor(newScale));
    }
    
    // Cap at max scale
    return Math.min(newScale, maxScale);
  }, [width, height, integerScaling, maxScale]);

  // Handle resize
  useEffect(() => {
    if (!containerRef.current) return;

    const resizeObserver = new ResizeObserver((entries) => {
      for (const entry of entries) {
        const { width: containerWidth, height: containerHeight } = entry.contentRect;
        setContainerSize({ width: containerWidth, height: containerHeight });
        const newScale = calculateScale(containerWidth, containerHeight);
        setScale(newScale);
      }
    });

    resizeObserver.observe(containerRef.current);

    return () => {
      resizeObserver.disconnect();
    };
  }, [calculateScale]);

  // Initialize canvas
  useEffect(() => {
    if (canvasRef.current && onCanvasReady) {
      onCanvasReady(canvasRef.current);
    }
  }, [onCanvasReady]);

  // Get CSS for image rendering based on scaling mode
  const getImageRendering = (): string => {
    switch (scalingMode) {
      case 'pixel-perfect':
        return 'pixelated';
      case 'smooth':
        return 'auto';
      case 'stretch':
        return 'auto';
      default:
        return 'pixelated';
    }
  };

  // Calculate canvas display size
  const displayWidth = scalingMode === 'stretch' ? containerSize.width : width * scale;
  const displayHeight = scalingMode === 'stretch' ? containerSize.height : height * scale;

  return (
    <div
      ref={containerRef}
      className={`emulator-canvas-container ${className}`}
      style={{
        position: 'relative',
        width: '100%',
        height: '100%',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        backgroundColor: 'var(--bg-primary, #0f172a)',
        overflow: 'hidden',
        ...style,
      }}
    >
      <canvas
        ref={canvasRef}
        width={width}
        height={height}
        className="emulator-canvas"
        style={{
          width: displayWidth,
          height: displayHeight,
          imageRendering: getImageRendering() as 'pixelated' | 'auto',
          objectFit: 'contain',
        }}
      />

      {/* Scanlines overlay */}
      {showScanlines && (
        <div
          className="scanlines-overlay"
          style={{
            position: 'absolute',
            top: '50%',
            left: '50%',
            transform: 'translate(-50%, -50%)',
            width: displayWidth,
            height: displayHeight,
            pointerEvents: 'none',
            backgroundImage: `
              repeating-linear-gradient(
                0deg,
                transparent,
                transparent 2px,
                rgba(0, 0, 0, 0.3) 2px,
                rgba(0, 0, 0, 0.3) 4px
              )
            `,
            opacity: 0.5,
          }}
        />
      )}

      {/* CRT effect overlay */}
      {showCrtEffect && (
        <div
          className="crt-overlay"
          style={{
            position: 'absolute',
            top: '50%',
            left: '50%',
            transform: 'translate(-50%, -50%)',
            width: displayWidth,
            height: displayHeight,
            pointerEvents: 'none',
            background: `
              radial-gradient(
                ellipse at center,
                transparent 0%,
                rgba(0, 0, 0, 0.2) 90%,
                rgba(0, 0, 0, 0.4) 100%
              )
            `,
            boxShadow: 'inset 0 0 50px rgba(0, 0, 0, 0.5)',
          }}
        />
      )}

      {/* Center crosshair for debugging */}
      {process.env.NODE_ENV === 'development' && (
        <div
          className="center-crosshair"
          style={{
            position: 'absolute',
            top: '50%',
            left: '50%',
            transform: 'translate(-50%, -50%)',
            width: displayWidth,
            height: displayHeight,
            pointerEvents: 'none',
          }}
        >
          <div
            style={{
              position: 'absolute',
              top: '50%',
              left: 0,
              right: 0,
              height: 1,
              backgroundColor: 'rgba(255, 0, 0, 0.5)',
            }}
          />
          <div
            style={{
              position: 'absolute',
              left: '50%',
              top: 0,
              bottom: 0,
              width: 1,
              backgroundColor: 'rgba(255, 0, 0, 0.5)',
            }}
          />
        </div>
      )}
    </div>
  );
};

export default EmulatorCanvas;
