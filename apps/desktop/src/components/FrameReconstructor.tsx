import React, { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useStore } from '../store/useStore';
import { SpriteCanvas } from './SpriteCanvas';
import { TilePalette } from './TilePalette';
import { SpriteProperties } from './SpriteProperties';
import { AnnotationPanel } from './AnnotationPanel';
import { FrameTagger } from './FrameTagger';
import { FrameData, FrameSummary, EditorTool, SpriteEntry } from '../types/frame';
import { FrameAnnotation, FrameTag } from '../types/frameTags';

export const FrameReconstructor: React.FC = () => {
  const { fighters, selectedFighterId, selectFighter, renderPose, selectedBoxer } = useStore();

  // State
  const [frames, setFrames] = useState<FrameSummary[]>([]);
  const [currentFrameIndex, setCurrentFrameIndex] = useState<number>(0);
  const [currentFrame, setCurrentFrame] = useState<FrameData | null>(null);
  const [previewUrl, setPreviewUrl] = useState<string | null>(null);
  const [selectedSprites, setSelectedSprites] = useState<number[]>([]);
  const [selectedTileId, setSelectedTileId] = useState<number | null>(null);
  const [currentTool, setCurrentTool] = useState<EditorTool>('select');
  const [zoom, setZoom] = useState<number>(2);
  const [showGrid, setShowGrid] = useState<boolean>(true);
  const [snapToGrid, setSnapToGrid] = useState<boolean>(true);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [hasChanges, setHasChanges] = useState(false);
  const [showAnnotationPanel, setShowAnnotationPanel] = useState(false);
  const [frameAnnotations, setFrameAnnotations] = useState<Record<number, FrameAnnotation>>({});
  const [frameTags, setFrameTags] = useState<FrameTag[]>([]);

  // Canvas settings
  const CANVAS_WIDTH = 512;
  const CANVAS_HEIGHT = 512;

  // Load frames when fighter changes
  useEffect(() => {
    if (selectedFighterId === null) return;

    const loadFrames = async () => {
      setLoading(true);
      setError(null);
      try {
        const frameList = await invoke<FrameSummary[]>('get_fighter_frames', {
          fighterId: selectedFighterId,
        });
        setFrames(frameList);
        
        // Load first frame
        if (frameList.length > 0) {
          await loadFrameDetail(0);
        }
        
        // Load annotations for this fighter
        await loadFrameAnnotations();
        await loadFrameTags();
      } catch (e) {
        console.error('Failed to load frames:', e);
        setError('Failed to load frames');
      } finally {
        setLoading(false);
      }
    };

    loadFrames();
  }, [selectedFighterId]);
  
  // Load all frame annotations for current fighter
  const loadFrameAnnotations = async () => {
    if (selectedFighterId === null) return;
    try {
      const result = await invoke<Record<string, FrameAnnotation> | null>('get_fighter_annotations', {
        fighterId: selectedFighterId.toString(),
      });
      if (result) {
        const annotations: Record<number, FrameAnnotation> = {};
        Object.entries(result).forEach(([key, value]) => {
          const frameIndex = parseInt(key, 10);
          if (!isNaN(frameIndex)) {
            annotations[frameIndex] = value;
          }
        });
        setFrameAnnotations(annotations);
      }
    } catch (e) {
      console.error('Failed to load frame annotations:', e);
    }
  };
  
  // Load all frame tags
  const loadFrameTags = async () => {
    try {
      const result = await invoke<FrameTag[]>('get_frame_tags');
      setFrameTags(result);
    } catch (e) {
      console.error('Failed to load frame tags:', e);
    }
  };

  // Load frame detail and preview
  const loadFrameDetail = async (frameIndex: number) => {
    if (selectedFighterId === null) return;

    setLoading(true);
    try {
      // Get frame data
      const frameData = await invoke<FrameData>('get_frame_detail', {
        fighterId: selectedFighterId,
        frameIndex,
      });
      setCurrentFrame(frameData);
      setCurrentFrameIndex(frameIndex);
      setSelectedSprites([]);
      setHasChanges(false);

      // Get preview
      const previewBytes = await invoke<number[]>('render_frame_preview', {
        fighterId: selectedFighterId,
        frameIndex,
      });
      const blob = new Blob([new Uint8Array(previewBytes)], { type: 'image/png' });
      const url = URL.createObjectURL(blob);
      setPreviewUrl(url);
    } catch (e) {
      console.error('Failed to load frame:', e);
      setError('Failed to load frame');
    } finally {
      setLoading(false);
    }
  };

  // Refresh preview with current frame data
  const refreshPreview = async () => {
    if (!currentFrame || selectedFighterId === null) return;

    try {
      const previewBytes = await invoke<number[]>('render_frame_data_preview', {
        fighterId: selectedFighterId,
        frame: currentFrame,
      });
      const blob = new Blob([new Uint8Array(previewBytes)], { type: 'image/png' });
      
      // Revoke old URL
      if (previewUrl) {
        URL.revokeObjectURL(previewUrl);
      }
      
      const url = URL.createObjectURL(blob);
      setPreviewUrl(url);
    } catch (e) {
      console.error('Failed to refresh preview:', e);
    }
  };

  // Update current frame in backend and refresh preview
  const updateFrame = async (updatedFrame: FrameData) => {
    setCurrentFrame(updatedFrame);
    setHasChanges(true);
    
    // Debounce preview refresh
    setTimeout(() => refreshPreview(), 50);
  };

  // Handle sprite selection
  const handleSelectSprite = useCallback((index: number, additive: boolean) => {
    if (index === -1) {
      setSelectedSprites([]);
      return;
    }

    if (additive) {
      setSelectedSprites(prev => 
        prev.includes(index) 
          ? prev.filter(i => i !== index)
          : [...prev, index]
      );
    } else {
      setSelectedSprites([index]);
    }
  }, []);

  // Handle sprite move
  const handleMoveSprite = useCallback(async (index: number, x: number, y: number) => {
    if (!currentFrame || selectedFighterId === null) return;

    try {
      const updatedFrame = await invoke<FrameData>('move_sprite', {
        fighterId: selectedFighterId,
        frameIndex: currentFrameIndex,
        spriteIndex: index,
        x,
        y,
      });
      await updateFrame(updatedFrame);
    } catch (e) {
      console.error('Failed to move sprite:', e);
    }
  }, [currentFrame, selectedFighterId, currentFrameIndex]);

  // Handle add sprite
  const handleAddSprite = useCallback(async (tileId: number, x: number, y: number) => {
    if (!currentFrame || selectedFighterId === null) return;

    try {
      const updatedFrame = await invoke<FrameData>('add_sprite_to_frame', {
        fighterId: selectedFighterId,
        frameIndex: currentFrameIndex,
        tileId,
        x,
        y,
      });
      await updateFrame(updatedFrame);
      
      // Select the newly added sprite (it's always the last one)
      setSelectedSprites([updatedFrame.sprites.length - 1]);
      
      // Switch back to select tool
      setCurrentTool('select');
    } catch (e) {
      console.error('Failed to add sprite:', e);
    }
  }, [currentFrame, selectedFighterId, currentFrameIndex]);

  // Handle remove sprite
  const handleRemoveSprite = useCallback(async (index: number) => {
    if (!currentFrame || selectedFighterId === null) return;

    try {
      const updatedFrame = await invoke<FrameData>('remove_sprite', {
        fighterId: selectedFighterId,
        frameIndex: currentFrameIndex,
        spriteIndex: index,
      });
      await updateFrame(updatedFrame);
      setSelectedSprites([]);
    } catch (e) {
      console.error('Failed to remove sprite:', e);
    }
  }, [currentFrame, selectedFighterId, currentFrameIndex]);

  // Handle duplicate sprite
  const handleDuplicateSprite = useCallback(async (index: number) => {
    if (!currentFrame || selectedFighterId === null) return;

    try {
      const updatedFrame = await invoke<FrameData>('duplicate_sprite', {
        fighterId: selectedFighterId,
        frameIndex: currentFrameIndex,
        spriteIndex: index,
      });
      await updateFrame(updatedFrame);
      
      // Select the duplicated sprite (it's always the last one)
      setSelectedSprites([updatedFrame.sprites.length - 1]);
    } catch (e) {
      console.error('Failed to duplicate sprite:', e);
    }
  }, [currentFrame, selectedFighterId, currentFrameIndex]);

  // Handle update sprite properties
  const handleUpdateSprite = useCallback(async (index: number, updates: Partial<SpriteEntry>) => {
    if (!currentFrame || selectedFighterId === null) return;

    const sprite = currentFrame.sprites[index];
    if (!sprite) return;

    try {
      const updatedFrame = await invoke<FrameData>('update_sprite_flags', {
        fighterId: selectedFighterId,
        frameIndex: currentFrameIndex,
        spriteIndex: index,
        hFlip: updates.h_flip ?? sprite.h_flip,
        vFlip: updates.v_flip ?? sprite.v_flip,
        palette: updates.palette ?? sprite.palette,
      });
      await updateFrame(updatedFrame);
    } catch (e) {
      console.error('Failed to update sprite:', e);
    }
  }, [currentFrame, selectedFighterId, currentFrameIndex]);

  // Handle save frame
  const handleSaveFrame = async () => {
    if (!currentFrame || selectedFighterId === null) return;

    try {
      await invoke('save_frame', {
        fighterId: selectedFighterId,
        frameIndex: currentFrameIndex,
        frame: currentFrame,
      });
      setHasChanges(false);
    } catch (e) {
      console.error('Failed to save frame:', e);
      setError('Failed to save frame');
    }
  };

  // Handle keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Tool shortcuts
      if (e.key === 'v' || e.key === 'V') {
        setCurrentTool('select');
      } else if (e.key === 'm' || e.key === 'M') {
        setCurrentTool('move');
      } else if (e.key === 'z' || e.key === 'Z') {
        if (!e.ctrlKey && !e.metaKey) {
          setCurrentTool('zoom');
        }
      } else if (e.key === 'g' || e.key === 'G') {
        if (e.ctrlKey || e.metaKey) {
          e.preventDefault();
          setShowGrid(prev => !prev);
        }
      } else if (e.key === 'Delete' || e.key === 'Backspace') {
        if (selectedSprites.length > 0 && !e.ctrlKey && !e.metaKey) {
          e.preventDefault();
          selectedSprites.forEach(idx => handleRemoveSprite(idx));
        }
      } else if (e.key === 'd' && (e.ctrlKey || e.metaKey)) {
        e.preventDefault();
        if (selectedSprites.length === 1) {
          handleDuplicateSprite(selectedSprites[0]);
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [selectedSprites, handleRemoveSprite, handleDuplicateSprite]);

  if (selectedFighterId === null) {
    return (
      <div style={styles.emptyContainer}>
        <div style={styles.emptyState}>
          <div style={styles.emptyIcon}>🎨</div>
          <h2 style={styles.emptyTitle}>Frame Reconstructor</h2>
          <p style={styles.emptyText}>
            Select a fighter from the sidebar to start editing frames
          </p>
        </div>
      </div>
    );
  }

  return (
    <div style={styles.container}>
      {/* Header */}
      <div style={styles.header}>
        <div style={styles.headerLeft}>
          <h2 style={styles.title}>Frame Editor</h2>
          
          {frames.length > 0 && (
            <select
              value={currentFrameIndex}
              onChange={(e) => loadFrameDetail(parseInt(e.target.value))}
              style={styles.frameSelect}
            >
              {frames.map((frame, idx) => {
                const annotation = frameAnnotations[idx];
                const hasTags = annotation && annotation.tags.length > 0;
                return (
                  <option key={idx} value={idx}>
                    {frame.name} ({frame.sprite_count} sprites) {hasTags ? '🏷️' : ''}
                  </option>
                );
              })}
            </select>
          )}

          <button
            onClick={() => setShowAnnotationPanel(!showAnnotationPanel)}
            style={{
              ...styles.annotationButton,
              backgroundColor: showAnnotationPanel ? '#0066cc' : '#2a2a3e',
            }}
            title="Toggle Annotation Panel"
          >
            📝 Annotate
          </button>

          {hasChanges && (
            <span style={styles.unsavedIndicator}>● Unsaved changes</span>
          )}
        </div>

        <div style={styles.headerRight}>
          {/* Tool buttons */}
          <div style={styles.toolGroup}>
            <button
              onClick={() => setCurrentTool('select')}
              style={{
                ...styles.toolButton,
                ...(currentTool === 'select' ? styles.toolButtonActive : {}),
              }}
              title="Select Tool (V)"
            >
              🔍 V
            </button>
            <button
              onClick={() => setCurrentTool('move')}
              style={{
                ...styles.toolButton,
                ...(currentTool === 'move' ? styles.toolButtonActive : {}),
              }}
              title="Move Tool (M) - Place tiles"
            >
              ✚ M
            </button>
          </div>

          {/* View options */}
          <div style={styles.viewGroup}>
            <label style={styles.checkbox}>
              <input
                type="checkbox"
                checked={showGrid}
                onChange={(e) => setShowGrid(e.target.checked)}
              />
              <span>Grid (G)</span>
            </label>
            <label style={styles.checkbox}>
              <input
                type="checkbox"
                checked={snapToGrid}
                onChange={(e) => setSnapToGrid(e.target.checked)}
              />
              <span>Snap</span>
            </label>
          </div>

          {/* Zoom */}
          <select
            value={zoom}
            onChange={(e) => setZoom(parseFloat(e.target.value))}
            style={styles.zoomSelect}
          >
            <option value={1}>100%</option>
            <option value={2}>200%</option>
            <option value={3}>300%</option>
            <option value={4}>400%</option>
          </select>

          {/* Save button */}
          <button
            onClick={handleSaveFrame}
            disabled={!hasChanges}
            style={{
              ...styles.saveButton,
              opacity: hasChanges ? 1 : 0.5,
            }}
          >
            Save Frame
          </button>
        </div>
      </div>

      {/* Main content */}
      <div style={styles.mainContent}>
        {/* Left sidebar - Tile Palette */}
        <div style={styles.leftSidebar}>
          <TilePalette
            fighterId={selectedFighterId}
            tilesetId={currentFrame?.tileset1_id || 0}
            paletteId={currentFrame?.palette_id || 0}
            selectedTileId={selectedTileId}
            onSelectTile={setSelectedTileId}
          />
        </div>

        {/* Center - Canvas */}
        <div style={styles.canvasContainer}>
          {loading ? (
            <div style={styles.loading}>Loading...</div>
          ) : error ? (
            <div style={styles.error}>{error}</div>
          ) : (
            <SpriteCanvas
              frame={currentFrame}
              previewUrl={previewUrl}
              selectedSprites={selectedSprites}
              currentTool={currentTool}
              zoom={zoom}
              showGrid={showGrid}
              gridSize={8}
              snapToGrid={snapToGrid}
              canvasWidth={CANVAS_WIDTH}
              canvasHeight={CANVAS_HEIGHT}
              onSelectSprite={handleSelectSprite}
              onMoveSprite={handleMoveSprite}
              onAddSprite={handleAddSprite}
              selectedTileId={selectedTileId}
            />
          )}
          
          {/* Tag badges overlay */}
          {frameAnnotations[currentFrameIndex]?.tags && frameAnnotations[currentFrameIndex].tags.length > 0 && (
            <div style={styles.tagOverlay}>
              {frameAnnotations[currentFrameIndex].tags.map(tagId => {
                const tag = frameTags.find(t => t.id === tagId);
                if (!tag) return null;
                return (
                  <span
                    key={tagId}
                    style={{
                      ...styles.tagBadge,
                      backgroundColor: tag.color,
                    }}
                    title={tag.description}
                  >
                    {tag.display_name}
                  </span>
                );
              })}
            </div>
          )}

          {/* Canvas toolbar */}
          <div style={styles.canvasToolbar}>
            <button onClick={() => setZoom(z => Math.min(z + 0.5, 4))} style={styles.toolbarButton}>+</button>
            <span style={styles.zoomDisplay}>{Math.round(zoom * 100)}%</span>
            <button onClick={() => setZoom(z => Math.max(z - 0.5, 1))} style={styles.toolbarButton}>-</button>
            <button onClick={() => setShowGrid(!showGrid)} style={styles.toolbarButton}>
              {showGrid ? '📐' : '▫️'}
            </button>
          </div>
        </div>

        {/* Right sidebar - Properties */}
        <div style={styles.rightSidebar}>
          {showAnnotationPanel && selectedFighterId !== null ? (
            <AnnotationPanel
              fighterId={selectedFighterId.toString()}
              fighterName={selectedBoxer?.name || `Boxer ${selectedBoxerId}`}
              frameIndex={currentFrameIndex}
              onAnnotationChange={() => loadFrameAnnotations()}
            />
          ) : (
            <SpriteProperties
              frame={currentFrame}
              selectedSprites={selectedSprites}
              onUpdateSprite={handleUpdateSprite}
              onRemoveSprite={handleRemoveSprite}
              onDuplicateSprite={handleDuplicateSprite}
            />
          )}
        </div>
      </div>

      {/* Status bar */}
      <div style={styles.statusBar}>
        <div style={styles.statusLeft}>
          {currentFrame && (
            <>
              <span>Sprites: {currentFrame.sprites.length}</span>
              <span style={styles.statusSeparator}>|</span>
              <span>Bounds: {currentFrame.width}×{currentFrame.height}</span>
              <span style={styles.statusSeparator}>|</span>
              <span>Tileset 1: {currentFrame.tileset1_id}</span>
              {currentFrame.tileset2_id > 0 && (
                <>, <span>Tileset 2: {currentFrame.tileset2_id}</span>
              )}
            </>
          )}
        </div>
        <div style={styles.statusRight}>
          <span>Tip: Drag tiles from palette to canvas</span>
        </div>
      </div>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    height: '100vh',
    backgroundColor: '#16161e',
    color: '#fff',
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '12px 24px',
    backgroundColor: '#1e1e2e',
    borderBottom: '1px solid #333',
  },
  headerLeft: {
    display: 'flex',
    alignItems: 'center',
    gap: 16,
  },
  headerRight: {
    display: 'flex',
    alignItems: 'center',
    gap: 16,
  },
  title: {
    margin: 0,
    fontSize: '18px',
    fontWeight: 600,
  },
  frameSelect: {
    padding: '6px 12px',
    backgroundColor: '#2a2a3e',
    border: '1px solid #444',
    borderRadius: 4,
    color: '#fff',
    fontSize: '13px',
    minWidth: 200,
  },
  annotationButton: {
    padding: '6px 12px',
    backgroundColor: '#2a2a3e',
    border: '1px solid #444',
    borderRadius: 4,
    color: '#fff',
    fontSize: '13px',
    cursor: 'pointer',
    transition: 'all 0.2s',
  },
  tagOverlay: {
    position: 'absolute',
    top: 16,
    left: 16,
    display: 'flex',
    flexWrap: 'wrap',
    gap: 6,
    maxWidth: 200,
    pointerEvents: 'none',
  },
  tagBadge: {
    padding: '4px 10px',
    borderRadius: 12,
    fontSize: 11,
    fontWeight: 500,
    color: '#fff',
    textShadow: '0 1px 2px rgba(0,0,0,0.3)',
    boxShadow: '0 2px 4px rgba(0,0,0,0.3)',
  },
  unsavedIndicator: {
    fontSize: '12px',
    color: '#fbbf24',
  },
  toolGroup: {
    display: 'flex',
    gap: 4,
  },
  toolButton: {
    padding: '8px 12px',
    backgroundColor: '#2a2a3e',
    border: '1px solid #444',
    borderRadius: 4,
    color: '#fff',
    fontSize: '13px',
    cursor: 'pointer',
    transition: 'all 0.2s',
  },
  toolButtonActive: {
    backgroundColor: '#0066cc',
    borderColor: '#0088ff',
  },
  viewGroup: {
    display: 'flex',
    gap: 12,
    fontSize: '12px',
  },
  checkbox: {
    display: 'flex',
    alignItems: 'center',
    gap: 6,
    cursor: 'pointer',
  },
  zoomSelect: {
    padding: '6px 8px',
    backgroundColor: '#2a2a3e',
    border: '1px solid #444',
    borderRadius: 4,
    color: '#fff',
    fontSize: '13px',
  },
  saveButton: {
    padding: '8px 16px',
    backgroundColor: '#22c55e',
    border: 'none',
    borderRadius: 4,
    color: '#fff',
    fontSize: '13px',
    fontWeight: 500,
    cursor: 'pointer',
    transition: 'opacity 0.2s',
  },
  mainContent: {
    display: 'flex',
    flex: 1,
    overflow: 'hidden',
  },
  leftSidebar: {
    width: 280,
    padding: 16,
    borderRight: '1px solid #333',
    overflow: 'hidden',
  },
  canvasContainer: {
    flex: 1,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    position: 'relative',
    backgroundColor: '#0f0f17',
  },
  rightSidebar: {
    width: 280,
    padding: 16,
    borderLeft: '1px solid #333',
    overflow: 'hidden',
  },
  loading: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    fontSize: '14px',
    color: '#888',
  },
  error: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    fontSize: '14px',
    color: '#ff6b6b',
  },
  canvasToolbar: {
    position: 'absolute',
    bottom: 16,
    left: '50%',
    transform: 'translateX(-50%)',
    display: 'flex',
    alignItems: 'center',
    gap: 8,
    padding: '8px 16px',
    backgroundColor: 'rgba(30, 30, 46, 0.9)',
    borderRadius: 8,
    backdropFilter: 'blur(4px)',
  },
  toolbarButton: {
    padding: '6px 12px',
    backgroundColor: '#2a2a3e',
    border: '1px solid #444',
    borderRadius: 4,
    color: '#fff',
    fontSize: '14px',
    cursor: 'pointer',
  },
  zoomDisplay: {
    minWidth: 50,
    textAlign: 'center',
    fontSize: '13px',
    fontFamily: 'monospace',
  },
  statusBar: {
    display: 'flex',
    justifyContent: 'space-between',
    padding: '8px 24px',
    backgroundColor: '#1e1e2e',
    borderTop: '1px solid #333',
    fontSize: '12px',
    color: '#888',
  },
  statusLeft: {
    display: 'flex',
    gap: 8,
  },
  statusSeparator: {
    margin: '0 8px',
    color: '#444',
  },
  statusRight: {
    fontStyle: 'italic',
  },
  emptyContainer: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    height: '100vh',
    backgroundColor: '#16161e',
  },
  emptyState: {
    textAlign: 'center',
    padding: 48,
  },
  emptyIcon: {
    fontSize: 64,
    marginBottom: 24,
  },
  emptyTitle: {
    fontSize: 24,
    fontWeight: 600,
    marginBottom: 12,
    color: '#fff',
  },
  emptyText: {
    fontSize: 14,
    color: '#888',
  },
};
