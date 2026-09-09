/**
 * Type Definitions
 *
 * Central export surface for app types.
 */

export * from './api';
export * from './aiBehavior';
export * from './frameTags';
export * from './roster';

// Export frame types explicitly to avoid Hitbox name collision with aiBehavior.
export type {
  SpriteEntry,
  FrameData,
  FrameSummary,
  EditorTool,
  FrameEditorState,
  TileData,
  CanvasPoint,
  SpriteBounds,
  Hitbox as FrameHitbox,
} from './frame';
export { defaultEditorState } from './frame';

// Export layout pack types explicitly to avoid ValidationReport collision with roster.
export type {
  LayoutPackMetadata,
  LayoutBin,
  PackBoxerLayout,
  LayoutPack,
  LayoutPackInfo,
  BoxerValidation,
  BoxerLayoutComparison,
  PackPreviewData,
  ExportSelection,
  LayoutPackSortField,
  LayoutPackSortOrder,
  ValidationReport as LayoutPackValidationReport,
} from './layoutPack';
