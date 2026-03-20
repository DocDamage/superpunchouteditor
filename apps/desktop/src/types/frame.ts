/**
 * Frame Reconstructor Types
 * 
 * TypeScript interfaces for the Frame Reconstructor feature.
 */

/** A sprite entry in a frame (OAM-style) */
export interface SpriteEntry {
  x: number;           // Screen X position (can be negative for off-screen)
  y: number;           // Screen Y position
  tile_id: number;     // Which tile to use
  palette: number;     // Palette number (0-7)
  h_flip: boolean;     // Horizontal flip
  v_flip: boolean;     // Vertical flip
  priority: number;    // Sprite priority (0-3)
}

/** Hitbox for collision detection */
export interface Hitbox {
  x: number;
  y: number;
  w: number;
  h: number;
  damage: number;
  stun: number;
}

/** Complete frame data */
export interface FrameData {
  name: string;
  sprites: SpriteEntry[];
  width: number;
  height: number;
  hitbox: Hitbox | null;
  tileset1_id: number;
  tileset2_id: number;
  palette_id: number;
  data_addr: number;
}

/** Summary info for a frame (for UI lists) */
export interface FrameSummary {
  name: string;
  sprite_count: number;
  width: number;
  height: number;
  has_hitbox: boolean;
}

/** Available tools in the frame editor */
export type EditorTool = 'select' | 'move' | 'zoom';

/** Editor state */
export interface FrameEditorState {
  currentFrame: FrameData | null;
  selectedSprites: number[];
  currentTool: EditorTool;
  zoom: number;
  showGrid: boolean;
  snapToGrid: boolean;
  gridSize: number;
  canvasOffsetX: number;
  canvasOffsetY: number;
  isDragging: boolean;
  dragStartX: number;
  dragStartY: number;
}

/** Default editor state */
export const defaultEditorState: FrameEditorState = {
  currentFrame: null,
  selectedSprites: [],
  currentTool: 'select',
  zoom: 2,
  showGrid: true,
  snapToGrid: true,
  gridSize: 8,
  canvasOffsetX: 0,
  canvasOffsetY: 0,
  isDragging: false,
  dragStartX: 0,
  dragStartY: 0,
};

/** Tile data for the tile palette */
export interface TileData {
  id: number;
  previewUrl: string | null;
}

/** Canvas coordinates */
export interface CanvasPoint {
  x: number;
  y: number;
}

/** Sprite bounds for hit testing */
export interface SpriteBounds {
  index: number;
  x: number;
  y: number;
  width: number;
  height: number;
}
