/**
 * Core Type Definitions for State Management
 * 
 * This file contains shared types used across all stores.
 * Domain-specific types are defined in their respective store files.
 */

// ============================================================================
// Asset Types
// ============================================================================

export interface AssetFile {
  file: string;
  filename: string;
  category: string;
  subtype: string;
  size: number;
  start_snes: string;
  end_snes: string;
  start_pc: string;
  end_pc: string;
  shared_with: string[];
}

export interface BoxerRecord {
  fighter: string;
  key: string;
  reference_sheet: string;
  palette_files: AssetFile[];
  icon_files: AssetFile[];
  portrait_files: AssetFile[];
  large_portrait_files: AssetFile[];
  unique_sprite_bins: AssetFile[];
  shared_sprite_bins: AssetFile[];
  other_files: AssetFile[];
}

export interface FighterMetadata {
  id: number;
  name: string;
  header_addr: number;
}

export interface PoseInfo {
  index: number;
  tileset1_id: number;
  tileset2_id: number;
  palette_id: number;
  data_addr: number;
}

// ============================================================================
// Color Types
// ============================================================================

export interface Color {
  r: number;
  g: number;
  b: number;
}

// ============================================================================
// Project Types
// ============================================================================

export interface ProjectMetadata {
  name: string;
  author: string | null;
  description: string | null;
  created_at: string;
  modified_at: string;
  version: string;
}

export interface ProjectEdit {
  asset_id: string;
  type: 'palette' | 'tile_import' | 'sprite_bin' | 'script' | 'fighter_params' | 'other';
  description: string | null;
  original_hash: string;
  edited_hash: string;
  pc_offset: string;
  size: number;
  timestamp: string;
  asset_path: string | null;
}

export interface ProjectAsset {
  id: string;
  name: string;
  asset_type: string;
  source_pc_offset: string;
  filename: string;
  exported_at: string;
}

export interface ProjectThumbnail {
  data_base64: string;
  width: number;
  height: number;
  captured_at: string;
  captured_view: string;
}

export interface ProjectFile {
  version: number;
  rom_base_sha1: string;
  manifest_version: string;
  metadata: ProjectMetadata;
  edits: ProjectEdit[];
  assets: ProjectAsset[];
  settings: Record<string, unknown>;
  thumbnail?: ProjectThumbnail;
}

// ============================================================================
// Edit History Types
// ============================================================================

export interface EditSummary {
  id: number;
  action_type: string;
  description: string;
  pc_offset: string | null;
  timestamp: string;
}

// ============================================================================
// Animation Types
// ============================================================================

export interface FrameEffect {
  type: 'Shake' | 'Flash' | 'Sound' | 'Hitbox';
  data?: number | { x: number; y: number; w: number; h: number };
}

export interface AnimationFrame {
  pose_id: number;
  duration: number;
  tileset_id: number;
  effects: FrameEffect[];
}

export interface Animation {
  name: string;
  frames: AnimationFrame[];
  looping: boolean;
  category: string;
}

export interface FighterAnimations {
  fighter_id: number;
  fighter_name: string;
  animations: Animation[];
}

// ============================================================================
// Comparison Types
// ============================================================================

export interface ComparisonSummary {
  total_changes: number;
  palettes_modified: number;
  sprite_bins_changed: number;
  tiles_changed: number;
  fighter_headers_edited: number;
  animation_timings_adjusted: number;
  total_bytes_changed: number;
}

export type Difference = 
  | { type: 'Palette'; offset: number; asset_id: string; boxer: string; changed_indices: number[] }
  | { type: 'Sprite'; boxer: string; bin_name: string; pc_offset: number; total_tiles: number; changed_tile_indices: number[] }
  | { type: 'Header'; boxer: string; fighter_index: number; changed_fields: HeaderFieldChange[] }
  | { type: 'Animation'; boxer: string; anim_name: string; frame_index: number; change_type: AnimationChangeType }
  | { type: 'Binary'; offset: number; size: number; bytes_changed: number; description: string };

export interface HeaderFieldChange {
  field_name: string;
  original_value: number;
  modified_value: number;
  display_name: string;
}

export interface AnimationChangeType {
  type: 'FrameCount' | 'Timing' | 'FrameData';
  original?: number;
  modified?: number;
  description?: string;
}

export interface RomComparison {
  original_sha1: string;
  modified_sha1: string;
  differences: Difference[];
  summary: ComparisonSummary;
}

export interface PaletteDiff {
  offset: number;
  boxer: string;
  asset_id: string;
  colors: ColorComparison[];
}

export interface ColorComparison {
  index: number;
  original: ColorDiff;
  modified: ColorDiff;
  changed: boolean;
}

export interface ColorDiff {
  r: number;
  g: number;
  b: number;
}

export interface SpriteDiff {
  pc_offset: number;
  boxer: string;
  bin_name: string;
  total_tiles: number;
  changed_tiles: TileDiff[];
}

export interface TileDiff {
  tile_index: number;
  pixel_diffs: PixelDiff[];
  has_changes: boolean;
}

export interface PixelDiff {
  x: number;
  y: number;
  original_pixel: number;
  modified_pixel: number;
  changed: boolean;
}

export interface BinaryDiff {
  offset: number;
  size: number;
  rows: HexRow[];
}

export interface HexRow {
  address: string;
  bytes: HexByte[];
  ascii: string;
}

export interface HexByte {
  value: number;
  changed: boolean;
  original_value?: number;
}

export interface ComparisonRenderParams {
  boxer_key: string;
  view_type: 'sprite' | 'frame' | 'animation' | 'palette' | 'portrait' | 'icon';
  show_original: boolean;
  show_modified: boolean;
  asset_offset?: string;
  palette_offset?: string;
  mode?: 'side-by-side' | 'overlay' | 'difference' | 'split' | 'blink';
}

// ============================================================================
// External Tools Types
// ============================================================================

export interface ExternalTool {
  id: string;
  name: string;
  executable_path: string;
  arguments_template: string;
  supported_file_types: string[];
  category: 'graphics_editor' | 'hex_editor' | 'tile_editor' | 'emulator' | 'other';
  enabled: boolean;
  working_directory?: string;
  env_vars: Record<string, string>;
}

export interface ToolContext {
  offset?: string;
  size?: number;
  snes_address?: string;
  category?: string;
  boxer?: string;
  metadata?: Record<string, string>;
}

// ============================================================================
// Update Types
// ============================================================================

export interface UpdateSettings {
  check_on_startup: boolean;
  check_interval: 'daily' | 'weekly' | 'monthly' | 'never';
  channel: 'stable' | 'beta';
  skipped_versions: string[];
  last_check: string | null;
}

export interface UpdateInfo {
  version: string;
  notes: string;
  pub_date: string | null;
  download_url: string | null;
  mandatory: boolean;
  is_latest: boolean;
}

export interface DownloadProgress {
  percent: number;
  downloaded: number;
  total: number;
  state: 'idle' | 'checking' | 'downloading' | 'verifying' | 'ready' | 'installing' | 'error';
}

// ============================================================================
// Bank Duplication Types
// ============================================================================

export interface BankDuplication {
  original_pc_offset: number;
  new_pc_offset: number;
  size: number;
  boxer_key: string;
  filename: string;
  created_at: string;
}

// ============================================================================
// UI Types
// ============================================================================

export type ViewMode = 'side-by-side' | 'overlay' | 'difference' | 'split' | 'blink';

export interface Toast {
  id: string;
  message: string;
  type: 'info' | 'success' | 'warning' | 'error';
  duration?: number;
}

export interface ModalState {
  isOpen: boolean;
  type: string | null;
  data?: unknown;
}

export interface EmulatorSettings {
  emulatorPath: string;
  emulatorType: 'snes9x' | 'bsnes' | 'mesen-s' | 'other';
  autoSaveBeforeLaunch: boolean;
  commandLineArgs: string;
  jumpToSelectedBoxer: boolean;
  defaultRound: number;
  saveStateDir: string | null;
}
