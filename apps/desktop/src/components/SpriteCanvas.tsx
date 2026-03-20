import React, { useRef, useEffect, useState, useCallback } from 'react';
import { FrameData, SpriteEntry, EditorTool, CanvasPoint } from '../types/frame';

interface SpriteCanvasProps {
  frame: FrameData | null;
  previewUrl: string | null;
  selectedSprites: number[];
  currentTool: EditorTool;
  zoom: number;
  showGrid: boolean;
  gridSize: number;
  snapToGrid: boolean;
  canvasWidth: number;
  canvasHeight: number;
  onSelectSprite: (index: number, additive: boolean) => void;
  onMoveSprite: (index: number, x: number, y: number) => void;
  onAddSprite: (tileId: number, x: number, y: number) => void;
  onCanvasDrag?: (deltaX: number, deltaY: number) => void;
  selectedTileId: number | null;
}

export const SpriteCanvas: React.FC<SpriteCanvasProps> = ({
  frame,
  previewUrl,
  selectedSprites,
  currentTool,
  zoom,
  showGrid,
  gridSize,
  snapToGrid,
  canvasWidth,
  canvasHeight,
  onSelectSprite,
  onMoveSprite,
  onAddSprite,
  onCanvasDrag,
  selectedTileId,
}) => {
  const canvasRef = useRef<HTMLDivElement>(null);
  const [isDragging, setIsDragging] = useState(false);
  const [dragStart, setDragStart] = useState<CanvasPoint>({ x: 0, y: 0 });
  const [dragSprite, setDragSprite] = useState<number | null>(null);
  const [dragOffset, setDragOffset] = useState<CanvasPoint>({ x: 0, y: 0 });
  const [isPanning, setIsPanning] = useState(false);
  const [panStart, setPanStart] = useState<CanvasPoint>({ x: 0, y: 0 });

  const centerX = canvasWidth / 2;
  const centerY = canvasHeight / 2;

  // Convert screen coordinates to sprite space
  const screenToSprite = useCallback((screenX: number, screenY: number): CanvasPoint => {
    return {
      x: Math.round((screenX - centerX) / zoom),
      y: Math.round((screenY - centerY) / zoom),
    };
  }, [centerX, centerY, zoom]);

  // Convert sprite coordinates to screen space
  const spriteToScreen = useCallback((spriteX: number, spriteY: number): CanvasPoint => {
    return {
      x: centerX + spriteX * zoom,
      y: centerY + spriteY * zoom,
    };
  }, [centerX, centerY, zoom]);

  // Snap to grid
  const snapToGridFn = useCallback((value: number): number => {
    if (!snapToGrid) return value;
    return Math.round(value / gridSize) * gridSize;
  }, [snapToGrid, gridSize]);

  // Hit test - find sprite at position
  const hitTest = useCallback((screenX: number, screenY: number): number | null => {
    if (!frame) return null;
    
    const spritePos = screenToSprite(screenX, screenY);
    
    // Check sprites in reverse order (top to bottom)
    for (let i = frame.sprites.length - 1; i >= 0; i--) {
      const sprite = frame.sprites[i];
      if (
        spritePos.x >= sprite.x &&
        spritePos.x < sprite.x + 8 &&
        spritePos.y >= sprite.y &&
        spritePos.y < sprite.y + 8
      ) {
        return i;
      }
    }
    return null;
  }, [frame, screenToSprite]);

  // Handle mouse down
  const handleMouseDown = (e: React.MouseEvent) => {
    e.preventDefault();
    const rect = canvasRef.current?.getBoundingClientRect();
    if (!rect) return;

    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    // Space + drag = pan
    if (e.button === 1 || (e.button === 0 && e.shiftKey)) {
      setIsPanning(true);
      setPanStart({ x, y });
      return;
    }

    if (currentTool === 'select') {
      const hitSprite = hitTest(x, y);
      
      if (hitSprite !== null) {
        // Start dragging sprite
        const sprite = frame?.sprites[hitSprite];
        if (sprite) {
          const screenPos = spriteToScreen(sprite.x, sprite.y);
          setDragOffset({
            x: x - screenPos.x,
            y: y - screenPos.y,
          });
        }
        setDragSprite(hitSprite);
        setIsDragging(true);
        onSelectSprite(hitSprite, e.ctrlKey || e.metaKey);
      } else {
        // Clicked empty space - deselect
        if (!e.ctrlKey && !e.metaKey) {
          onSelectSprite(-1, false);
        }
      }
    } else if (currentTool === 'move' && selectedTileId !== null) {
      // Add new sprite at position
      const spritePos = screenToSprite(x, y);
      const snappedX = snapToGridFn(spritePos.x);
      const snappedY = snapToGridFn(spritePos.y);
      onAddSprite(selectedTileId, snappedX, snappedY);
    }
  };

  // Handle mouse move
  const handleMouseMove = (e: React.MouseEvent) => {
    const rect = canvasRef.current?.getBoundingClientRect();
    if (!rect) return;

    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    if (isPanning && onCanvasDrag) {
      const deltaX = x - panStart.x;
      const deltaY = y - panStart.y;
      onCanvasDrag(deltaX, deltaY);
      setPanStart({ x, y });
      return;
    }

    if (isDragging && dragSprite !== null && frame) {
      const newScreenX = x - dragOffset.x;
      const newScreenY = y - dragOffset.y;
      const newSpritePos = screenToSprite(newScreenX, newScreenY);
      
      const snappedX = snapToGridFn(newSpritePos.x);
      const snappedY = snapToGridFn(newSpritePos.y);
      
      onMoveSprite(dragSprite, snappedX, snappedY);
    }
  };

  // Handle mouse up
  const handleMouseUp = () => {
    setIsDragging(false);
    setDragSprite(null);
    setIsPanning(false);
  };

  // Handle context menu (right click to delete)
  const handleContextMenu = (e: React.MouseEvent) => {
    e.preventDefault();
    const rect = canvasRef.current?.getBoundingClientRect();
    if (!rect) return;

    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;
    const hitSprite = hitTest(x, y);

    if (hitSprite !== null && frame) {
      onSelectSprite(hitSprite, false);
      // The parent component will handle deletion
    }
  };

  // Handle keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (!frame) return;

      const nudgeAmount = e.shiftKey ? gridSize : 1;

      switch (e.key) {
        case 'ArrowUp':
          e.preventDefault();
          selectedSprites.forEach(idx => {
            const sprite = frame.sprites[idx];
            if (sprite) {
              onMoveSprite(idx, sprite.x, sprite.y - nudgeAmount);
            }
          });
          break;
        case 'ArrowDown':
          e.preventDefault();
          selectedSprites.forEach(idx => {
            const sprite = frame.sprites[idx];
            if (sprite) {
              onMoveSprite(idx, sprite.x, sprite.y + nudgeAmount);
            }
          });
          break;
        case 'ArrowLeft':
          e.preventDefault();
          selectedSprites.forEach(idx => {
            const sprite = frame.sprites[idx];
            if (sprite) {
              onMoveSprite(idx, sprite.x - nudgeAmount, sprite.y);
            }
          });
          break;
        case 'ArrowRight':
          e.preventDefault();
          selectedSprites.forEach(idx => {
            const sprite = frame.sprites[idx];
            if (sprite) {
              onMoveSprite(idx, sprite.x + nudgeAmount, sprite.y);
            }
          });
          break;
        case 'Delete':
        case 'Backspace':
          e.preventDefault();
          // Parent handles deletion
          break;
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [frame, selectedSprites, gridSize, onMoveSprite]);

  return (
    <div
      ref={canvasRef}
      className="sprite-canvas"
      style={{
        width: canvasWidth,
        height: canvasHeight,
        backgroundColor: '#1a1a2e',
        position: 'relative',
        overflow: 'hidden',
        cursor: isPanning ? 'grabbing' : currentTool === 'move' ? 'crosshair' : 'default',
        imageRendering: 'pixelated',
      }}
      onMouseDown={handleMouseDown}
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
      onMouseLeave={handleMouseUp}
      onContextMenu={handleContextMenu}
    >
      {/* Grid overlay */}
      {showGrid && (
        <svg
          style={{
            position: 'absolute',
            top: 0,
            left: 0,
            width: '100%',
            height: '100%',
            pointerEvents: 'none',
          }}
        >
          <defs>
            <pattern
              id="grid"
              width={gridSize * zoom}
              height={gridSize * zoom}
              patternUnits="userSpaceOnUse"
            >
              <path
                d={`M ${gridSize * zoom} 0 L 0 0 0 ${gridSize * zoom}`}
                fill="none"
                stroke="rgba(100, 100, 120, 0.3)"
                strokeWidth={1}
              />
            </pattern>
          </defs>
          <rect width="100%" height="100%" fill="url(#grid)" />
          
          {/* Center crosshair */}
          <line
            x1={centerX}
            y1={0}
            x2={centerX}
            y2={canvasHeight}
            stroke="rgba(100, 100, 120, 0.5)"
            strokeWidth={1}
          />
          <line
            x1={0}
            y1={centerY}
            x2={canvasWidth}
            y2={centerY}
            stroke="rgba(100, 100, 120, 0.5)"
            strokeWidth={1}
          />
        </svg>
      )}

      {/* Frame preview image */}
      {previewUrl && (
        <img
          src={previewUrl}
          alt="Frame Preview"
          style={{
            position: 'absolute',
            left: centerX - 128 * zoom,
            top: centerY - 128 * zoom,
            width: 256 * zoom,
            height: 256 * zoom,
            imageRendering: 'pixelated',
            pointerEvents: 'none',
            opacity: 0.9,
          }}
        />
      )}

      {/* Selection overlays */}
      {frame && selectedSprites.map(idx => {
        const sprite = frame.sprites[idx];
        if (!sprite) return null;
        
        const screenPos = spriteToScreen(sprite.x, sprite.y);
        
        return (
          <div
            key={idx}
            style={{
              position: 'absolute',
              left: screenPos.x,
              top: screenPos.y,
              width: 8 * zoom,
              height: 8 * zoom,
              border: '2px solid #00a8ff',
              boxSizing: 'border-box',
              pointerEvents: 'none',
              zIndex: 100,
            }}
          >
            {/* Selection handles */}
            <div style={{
              position: 'absolute',
              top: -4,
              left: -4,
              width: 8,
              height: 8,
              backgroundColor: '#00a8ff',
              borderRadius: '50%',
            }} />
            <div style={{
              position: 'absolute',
              top: -4,
              right: -4,
              width: 8,
              height: 8,
              backgroundColor: '#00a8ff',
              borderRadius: '50%',
            }} />
            <div style={{
              position: 'absolute',
              bottom: -4,
              left: -4,
              width: 8,
              height: 8,
              backgroundColor: '#00a8ff',
              borderRadius: '50%',
            }} />
            <div style={{
              position: 'absolute',
              bottom: -4,
              right: -4,
              width: 8,
              height: 8,
              backgroundColor: '#00a8ff',
              borderRadius: '50%',
            }} />
          </div>
        );
      })}

      {/* Info overlay */}
      <div
        style={{
          position: 'absolute',
          bottom: 8,
          left: 8,
          padding: '4px 8px',
          backgroundColor: 'rgba(0, 0, 0, 0.7)',
          color: '#aaa',
          fontSize: '11px',
          borderRadius: 4,
          pointerEvents: 'none',
        }}
      >
        {frame ? `${frame.sprites.length} sprites` : 'No frame loaded'} 
        {selectedSprites.length > 0 && ` • ${selectedSprites.length} selected`}
        {snapToGrid && ` • Snap: ${gridSize}px`}
      </div>
    </div>
  );
};
