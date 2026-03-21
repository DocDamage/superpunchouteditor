import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { FrameData, FrameSummary } from '../types/frame';
import { FrameTag, FrameAnnotation } from '../types/frameTags';

/**
 * Capture a screenshot of the current viewport using DOM API.
 * Returns a canvas element with the screenshot.
 */
async function captureScreenshot(): Promise<HTMLCanvasElement | null> {
  try {
    // Get the body element
    const element = document.body;
    
    // Create a canvas
    const canvas = document.createElement('canvas');
    const ctx = canvas.getContext('2d');
    if (!ctx) return null;
    
    // Set canvas dimensions to match viewport
    const rect = element.getBoundingClientRect();
    canvas.width = Math.min(rect.width, 1920); // Cap at 1920px width
    canvas.height = Math.min(rect.height, 1080); // Cap at 1080px height
    
    // Use html2canvas approach - render the document to canvas
    // For simplicity, we capture just the visible viewport
    const dataUrl = await domToImage(element);
    if (!dataUrl) return null;
    
    const img = new Image();
    img.src = dataUrl;
    await new Promise((resolve) => { img.onload = resolve; });
    
    // Draw image to canvas, scaling to fit
    ctx.drawImage(img, 0, 0, canvas.width, canvas.height);
    
    return canvas;
  } catch (e) {
    console.error('Screenshot capture failed:', e);
    return null;
  }
}

/**
 * Simple DOM to image converter using SVG foreignObject
 */
async function domToImage(element: HTMLElement): Promise<string | null> {
  try {
    const rect = element.getBoundingClientRect();
    const width = Math.min(rect.width, 1920);
    const height = Math.min(rect.height, 1080);
    
    // Clone the element to avoid modifying the original
    const clone = element.cloneNode(true) as HTMLElement;
    
    // Serialize the DOM
    const serializer = new XMLSerializer();
    const domString = serializer.serializeToString(clone);
    
    // Create SVG with foreignObject
    const svg = `
      <svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}">
        <foreignObject width="100%" height="100%">
          <div xmlns="http://www.w3.org/1999/xhtml">
            ${domString}
          </div>
        </foreignObject>
      </svg>
    `;
    
    // Convert to data URL
    const svgBlob = new Blob([svg], { type: 'image/svg+xml;charset=utf-8' });
    const url = URL.createObjectURL(svgBlob);
    
    // Load as image
    return new Promise((resolve) => {
      const img = new Image();
      img.onload = () => {
        const canvas = document.createElement('canvas');
        canvas.width = width;
        canvas.height = height;
        const ctx = canvas.getContext('2d');
        if (ctx) {
          ctx.drawImage(img, 0, 0);
          URL.revokeObjectURL(url);
          resolve(canvas.toDataURL('image/png'));
        } else {
          resolve(null);
        }
      };
      img.onerror = () => {
        URL.revokeObjectURL(url);
        resolve(null);
      };
      img.src = url;
    });
  } catch (e) {
    console.error('DOM to image failed:', e);
    return null;
  }
}

// Helper function to convert RGB color to SNES 15-bit BGR format
// SNES format: 0bbbbbgggggrrrrr (little endian: low byte, high byte)
function colorToSnesBytes(color: Color): number[] {
  // Convert 8-bit RGB to 5-bit
  const r5 = Math.round(color.r / 255 * 31) & 0x1F;
  const g5 = Math.round(color.g / 255 * 31) & 0x1F;
  const b5 = Math.round(color.b / 255 * 31) & 0x1F;
  
  // Pack into 15-bit BGR
  const snesColor = (b5 << 10) | (g5 << 5) | r5;
  
  // Return as little-endian bytes
  return [snesColor & 0xFF, (snesColor >> 8) & 0xFF];
}

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
  /** Boxer display name */
  name: string;
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

function normalizeAssetFile(asset: Partial<AssetFile> | null | undefined): AssetFile {
  return {
    file: asset?.file ?? '',
    filename: asset?.filename ?? '',
    category: asset?.category ?? '',
    subtype: asset?.subtype ?? '',
    size: asset?.size ?? 0,
    start_snes: asset?.start_snes ?? '',
    end_snes: asset?.end_snes ?? '',
    start_pc: asset?.start_pc ?? '',
    end_pc: asset?.end_pc ?? '',
    shared_with: Array.isArray(asset?.shared_with) ? asset!.shared_with : [],
  };
}

function normalizeAssetList(assets: Partial<AssetFile>[] | null | undefined): AssetFile[] {
  if (!Array.isArray(assets)) return [];
  return assets.map(normalizeAssetFile);
}

function normalizeBoxerRecord(boxer: Partial<BoxerRecord> | null | undefined): BoxerRecord | null {
  if (!boxer) return null;

  return {
    name: boxer.name ?? '',
    key: boxer.key ?? '',
    reference_sheet: boxer.reference_sheet ?? '',
    palette_files: normalizeAssetList(boxer.palette_files),
    icon_files: normalizeAssetList(boxer.icon_files),
    portrait_files: normalizeAssetList(boxer.portrait_files),
    large_portrait_files: normalizeAssetList(boxer.large_portrait_files),
    unique_sprite_bins: normalizeAssetList(boxer.unique_sprite_bins),
    shared_sprite_bins: normalizeAssetList(boxer.shared_sprite_bins),
    other_files: normalizeAssetList(boxer.other_files),
  };
}

/** @deprecated Use BoxerRecord with 'name' field instead */
export type FighterRecord = BoxerRecord;

export interface Color {
  r: number;
  g: number;
  b: number;
}

export interface BoxerMetadata {
  id: number;
  name: string;
  header_addr: number;
}

/** @deprecated Use BoxerMetadata instead */
export type FighterMetadata = BoxerMetadata;

export interface PoseInfo {
  index: number;
  tileset1_id: number;
  tileset2_id: number;
  palette_id: number;
  data_addr: number;
}

// Frame reconstructor types
export type { FrameData, FrameSummary } from '../types/frame';

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

export interface EditableBoxerParams {
  palette_id: number;
  attack_power: number;
  defense_rating: number;
  speed_rating: number;
}

/** @deprecated Use EditableBoxerParams instead */
export type EditableFighterParams = EditableBoxerParams;

export interface ParamValidationResult {
  valid: boolean;
  warnings: string[];
  is_extreme: boolean;
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

export interface EditSummary {
  id: number;
  action_type: string;
  description: string;
  pc_offset: string | null;
  timestamp: string;
}

// Animation types
export interface AnimationFrame {
  pose_id: number;
  duration: number;
  tileset_id: number;
  effects: FrameEffect[];
}

export interface FrameEffect {
  type: 'Shake' | 'Flash' | 'Sound' | 'Hitbox';
  data?: number | { x: number; y: number; w: number; h: number };
}

export interface Animation {
  name: string;
  frames: AnimationFrame[];
  looping: boolean;
  category: string;
}

export interface BoxerAnimations {
  boxer_id: number;
  boxer_name: string;
  animations: Animation[];
}

/** @deprecated Use BoxerAnimations instead */
export type FighterAnimations = BoxerAnimations;

export interface BankDuplication {
  original_pc_offset: number;
  new_pc_offset: number;
  size: number;
  boxer_key: string;
  filename: string;
  created_at: string;
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

// Update-related types
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

export interface InGameExpansionOptions {
  targetBoxerCount: number;
  patchEditorHook?: boolean;
  editorHookPcOffset?: string | null;
  editorHookOverwriteLen?: number | null;
}

export interface InGameExpansionWriteRange {
  start_pc: string;
  size: number;
  description: string;
}

export interface InGameHookSiteCandidate {
  hook_pc: string;
  overwrite_len: number;
  return_pc: string;
  first_instruction: string;
  preview_bytes_hex: string;
}

export interface InGameHookPreset extends InGameHookSiteCandidate {
  id: string;
  label: string;
  description: string;
  region: string;
  source: 'curated' | 'scanned';
  verified: boolean;
}

export interface InGameExpansionReport {
  boxer_count: number;
  header_pc: string;
  editor_stub_pc: string;
  editor_hook_patched: boolean;
  editor_hook_overwrite_len: number;
  name_pointer_table_pc: string;
  name_long_pointer_table_pc: string;
  name_blob_pc: string;
  circuit_table_pc: string;
  unlock_table_pc: string;
  intro_table_pc: string;
  write_ranges: InGameExpansionWriteRange[];
  notes: string[];
}

interface AppStore {
  romSha1: string | null;
  boxers: BoxerRecord[];
  selectedBoxer: BoxerRecord | null;
  currentPalette: Color[] | null;
  currentPaletteOffset: string | null;  // PC offset of the current palette for history tracking
  error: string | null;

  fighters: FighterMetadata[];
  selectedFighterId: number | null;
  poses: PoseInfo[];

  // Frame reconstructor state
  frames: FrameSummary[];
  currentFrame: FrameData | null;
  currentFrameIndex: number;

  /** Set of PC offset strings that have been staged for writing */
  pendingWrites: Set<string>;

  /** Current project */
  currentProject: ProjectFile | null;
  currentProjectPath: string | null;
  isProjectModified: boolean;

  // Actions
  loadBoxers: () => Promise<void>;
  openRom: (path: string) => Promise<void>;
  selectBoxer: (key: string) => Promise<void>;
  setError: (error: string | null) => void;
  updateColor: (index: number, newColor: Color, pcOffset?: string) => void;
  exportAsset: (asset: any, palette: any, path: string) => Promise<void>;
  importAsset: (palette: any, path: string) => Promise<Uint8Array | null>;

  setPendingWrite: (pcOffset: string) => void;
  removePendingWrite: (pcOffset: string) => void;
  clearPendingWrites: () => void;

  loadFighterList: () => Promise<void>;
  selectFighter: (id: number) => Promise<void>;
  renderPose: (fighterId: number, poseId: number) => Promise<Uint8Array>;

  // Shared bank utilities
  getSharedBankInfo: (pcOffset: string) => Promise<{
    found: boolean;
    filename?: string;
    category?: string;
    size?: number;
    start_pc?: string;
    shared_with?: string[];
    all_fighters_using?: string[];
    is_shared?: boolean;
    message?: string;
  }>;
  getFighterSharedBanks: (fighterKey: string) => Promise<{
    fighter: string;
    key: string;
    unique_bin_count: number;
    shared_bin_count: number;
    shared_bins: Array<{
      filename: string;
      start_pc: string;
      size: number;
      shared_with: string[];
    }>;
    shares_with: string[];
    is_safe_target: boolean;
  }>;

  // Project management actions
  createProject: (path: string, name: string, author?: string, description?: string) => Promise<void>;
  saveProject: (path?: string, metadata?: ProjectMetadata) => Promise<void>;
  loadProject: (path: string) => Promise<void>;
  validateProject: (path: string) => Promise<boolean>;
  getCurrentProject: () => Promise<void>;
  setProjectModified: (modified: boolean) => void;

  // Project thumbnail actions
  captureThumbnail: (viewType: string) => Promise<ProjectThumbnail | null>;
  saveThumbnail: (thumbnail: ProjectThumbnail) => Promise<void>;
  getThumbnail: () => Promise<ProjectThumbnail | null>;
  clearThumbnail: () => Promise<void>;
  loadThumbnailFromPath: (projectPath: string) => Promise<ProjectThumbnail | null>;

  // Undo/Redo state
  canUndo: boolean;
  canRedo: boolean;
  editHistory: EditSummary[];
  undoStack: EditSummary[];
  redoStack: EditSummary[];

  // Undo/Redo actions
  undo: () => Promise<void>;
  redo: () => Promise<void>;
  refreshUndoState: () => Promise<void>;
  clearHistory: () => Promise<void>;
  
  // History recording helpers
  recordPaletteEdit: (pcOffset: string, colorIndex: number, oldColor: number[], newColor: number[]) => Promise<void>;
  recordSpriteBinEdit: (pcOffset: string, oldBytes: number[], newBytes: number[]) => Promise<void>;
  recordAssetImport: (pcOffset: string, oldBytes: number[], newBytes: number[], sourcePath: string) => Promise<void>;

  // Bank duplication state and actions
  bankDuplications: BankDuplication[];
  loadBankDuplications: (boxerKey: string) => Promise<void>;
  duplicateSharedBank: (
    pcOffset: string, 
    size: number, 
    boxerKey: string, 
    filename: string, 
    isCompressed: boolean,
    allowExpansion: boolean
  ) => Promise<{ success: boolean; duplication?: BankDuplication; error?: string }>;

  // Animation state
  fighterAnimations: FighterAnimations | null;
  selectedAnimationIndex: number;
  selectedFrameIndex: number;
  
  // Animation actions
  loadFighterAnimations: (fighterId: number) => Promise<void>;
  selectAnimation: (index: number) => void;
  selectFrame: (index: number) => void;
  updateAnimationFrame: (frameIndex: number, updates: Partial<AnimationFrame>) => void;
  addAnimationFrame: () => void;
  removeAnimationFrame: (frameIndex: number) => void;

  // Comparison state
  comparison: RomComparison | null;
  comparisonLoading: boolean;
  selectedComparisonAsset: string | null;
  comparisonViewMode: 'side-by-side' | 'overlay' | 'difference' | 'split' | 'blink';
  
  // Comparison actions
  generateComparison: () => Promise<void>;
  selectComparisonAsset: (assetId: string | null) => void;
  setComparisonViewMode: (mode: 'side-by-side' | 'overlay' | 'difference' | 'split' | 'blink') => void;
  getPaletteDiff: (pcOffset: string) => Promise<PaletteDiff | null>;
  getSpriteBinDiff: (pcOffset: string) => Promise<SpriteDiff | null>;
  getBinaryDiff: (pcOffset: string, size: number) => Promise<BinaryDiff | null>;
  renderComparisonView: (params: ComparisonRenderParams) => Promise<Uint8Array | null>;
  exportComparisonReport: (outputPath: string, format: 'json' | 'html' | 'text') => Promise<void>;

  // Patch notes state
  patchNotesContent: string | null;
  changeSummary: {
    total_boxers_modified: number;
    total_palettes_changed: number;
    total_sprites_edited: number;
    total_animations_modified: number;
    total_headers_edited: number;
    total_changes: number;
  } | null;

  // Patch notes actions
  generatePatchNotes: (format: string, title?: string, author?: string, version?: string) => Promise<string | null>;
  getChangeSummary: () => Promise<void>;
  savePatchNotes: (content: string, outputPath: string) => Promise<void>;

  // External tools state
  externalTools: ExternalTool[];
  defaultToolIds: Record<string, string>;
  
  // External tools actions
  loadExternalTools: () => Promise<void>;
  addExternalTool: (tool: ExternalTool) => Promise<void>;
  removeExternalTool: (toolId: string) => Promise<void>;
  updateExternalTool: (tool: ExternalTool) => Promise<void>;
  launchWithTool: (toolId: string, filePath: string, context?: ToolContext) => Promise<void>;
  getCompatibleTools: (fileExtension: string) => Promise<ExternalTool[]>;
  setDefaultTool: (fileExtension: string, toolId: string) => Promise<void>;
  getDefaultTool: (fileExtension: string) => Promise<ExternalTool | null>;
  verifyTool: (tool: ExternalTool) => Promise<{ valid: boolean; message: string }>;

  // Frame reconstructor actions
  loadFrames: (fighterId: number) => Promise<void>;
  loadFrameDetail: (fighterId: number, frameIndex: number) => Promise<FrameData | null>;
  saveFrame: (fighterId: number, frameIndex: number, frame: FrameData) => Promise<void>;
  addSpriteToFrame: (fighterId: number, frameIndex: number, tileId: number, x: number, y: number) => Promise<FrameData | null>;
  moveSprite: (fighterId: number, frameIndex: number, spriteIndex: number, x: number, y: number) => Promise<FrameData | null>;
  updateSpriteFlags: (fighterId: number, frameIndex: number, spriteIndex: number, hFlip: boolean, vFlip: boolean, palette: number) => Promise<FrameData | null>;
  removeSprite: (fighterId: number, frameIndex: number, spriteIndex: number) => Promise<FrameData | null>;
  duplicateSprite: (fighterId: number, frameIndex: number, spriteIndex: number) => Promise<FrameData | null>;
  renderFramePreview: (fighterId: number, frameIndex: number) => Promise<Uint8Array>;

  // Boxer comparison actions
  compareBoxers: (boxerAKey: string, boxerBKey: string) => Promise<unknown>;
  getSimilarBoxers: (referenceKey: string, limit?: number) => Promise<unknown[]>;
  copyBoxerStat: (sourceKey: string, targetKey: string, statField: string) => Promise<void>;
  copyAllBoxerStats: (sourceKey: string, targetKey: string) => Promise<void>;

  // Frame annotation actions
  addFrameAnnotation: (fighterId: number, frameIndex: number, x: number, y: number, text: string, category: string) => Promise<void>;
  removeFrameAnnotation: (fighterId: number, frameIndex: number, annotationId: string) => Promise<void>;
  updateFrameAnnotation: (fighterId: number, frameIndex: number, annotationId: string, updates: Partial<FrameAnnotation>) => Promise<void>;
  getFrameAnnotations: (fighterId: number, frameIndex: number) => Promise<FrameAnnotation[]>;

  // In-ROM expansion actions
  applyInGameExpansion: (options: InGameExpansionOptions) => Promise<InGameExpansionReport>;
  analyzeInGameHookSites: (options?: {
    startPcOffset?: string | null;
    endPcOffset?: string | null;
    limit?: number;
  }) => Promise<InGameHookSiteCandidate[]>;
  verifyInGameHookSite: (options: {
    hookPcOffset: string;
    overwriteLen?: number | null;
  }) => Promise<InGameHookSiteCandidate>;
  getInGameHookPresets: (limit?: number) => Promise<InGameHookPreset[]>;
  
  // Update state
  updateSettings: UpdateSettings;
  currentVersion: string;
  availableUpdate: UpdateInfo | null;
  downloadProgress: DownloadProgress;
  checkingForUpdate: boolean;
  updateError: string | null;
  
  // Update actions
  loadUpdateSettings: () => Promise<void>;
  saveUpdateSettings: (settings: UpdateSettings) => Promise<void>;
  checkForUpdates: () => Promise<UpdateInfo | null>;
  skipVersion: (version: string) => Promise<void>;
  downloadAndInstallUpdate: () => Promise<void>;
  clearSkippedVersions: () => Promise<void>;
  shouldAutoCheck: () => Promise<boolean>;
}

// Comparison-related types
export interface RomComparison {
  original_sha1: string;
  modified_sha1: string;
  differences: Difference[];
  summary: ComparisonSummary;
}

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

export const useStore = create<AppStore>((set, get) => ({
  romSha1: null,
  boxers: [],
  selectedBoxer: null,
  currentPalette: null,
  currentPaletteOffset: null,
  error: null,
  pendingWrites: new Set(),

  currentProject: null,
  currentProjectPath: null,
  isProjectModified: false,

  fighters: [],
  selectedFighterId: null,
  poses: [],

  // Undo/Redo state
  canUndo: false,
  canRedo: false,
  editHistory: [],
  undoStack: [],
  redoStack: [],

  // Bank duplication state
  bankDuplications: [],

  // Frame reconstructor state
  frames: [],
  currentFrame: null,
  currentFrameIndex: 0,

  // Animation state
  fighterAnimations: null,
  selectedAnimationIndex: 0,
  selectedFrameIndex: 0,

  // Comparison state
  comparison: null,
  comparisonLoading: false,
  selectedComparisonAsset: null,
  comparisonViewMode: 'side-by-side',

  // Patch notes state
  patchNotesContent: null,
  changeSummary: null,
  
  // Frame tagging state
  frameTags: [],
  frameAnnotations: {},

  // External tools state
  externalTools: [],
  defaultToolIds: {},
  
  // Update state (initial values)
  updateSettings: {
    check_on_startup: true,
    check_interval: 'weekly',
    channel: 'stable',
    skipped_versions: [],
    last_check: null,
  },
  currentVersion: '0.1.0',
  availableUpdate: null,
  downloadProgress: {
    percent: 0,
    downloaded: 0,
    total: 0,
    state: 'idle',
  },
  checkingForUpdate: false,
  updateError: null,

  setError: (error) => set({ error }),

  loadBoxers: async () => {
    try {
      const boxers = await invoke<BoxerRecord[]>('get_boxers');
      set({ boxers: boxers.map(normalizeBoxerRecord).filter((boxer): boxer is BoxerRecord => boxer !== null) });
    } catch (e) {
      console.error('Failed to load boxers:', e);
      set({ error: (e as Error).toString() });
    }
  },

  openRom: async (path: string) => {
    try {
      // Reset all ROM-bound state before invoking the backend so stale data
      // is never visible if the load fails partway through.
      set({
        romSha1: null,
        boxers: [],
        selectedBoxer: null,
        currentPalette: null,
        currentPaletteOffset: null,
        fighters: [],
        selectedFighterId: null,
        poses: [],
        frames: [],
        currentFrame: null,
        currentFrameIndex: 0,
        pendingWrites: new Set(),
        error: null,
      });

      const sha1 = await invoke<string>('open_rom', { path });
      set({ romSha1: sha1 });

      // Reload boxer list from the region-specific manifest now loaded by the backend.
      await get().loadBoxers();
      // Sync undo state (history was cleared server-side on ROM load).
      await get().refreshUndoState();
    } catch (e) {
      console.error('Failed to open ROM:', e);
      set({ error: (e as Error).toString() });
    }
  },

  selectBoxer: async (key: string) => {
    try {
      const boxer = normalizeBoxerRecord(await invoke<BoxerRecord | null>('get_boxer', { key }));
      set({ selectedBoxer: boxer, currentPalette: null, currentPaletteOffset: null });

      // Auto-load first palette if available
      if (boxer && boxer.palette_files.length > 0) {
        const palette = await invoke<Color[]>('get_palette', {
          pcOffset: boxer.palette_files[0].start_pc,
          size: boxer.palette_files[0].size,
        });
        set({ 
          currentPalette: palette,
          currentPaletteOffset: boxer.palette_files[0].start_pc,
        });
      }
    } catch (e) {
      console.error('Failed to select boxer:', e);
    }
  },

  updateColor: (index, newColor, pcOffset) => {
    const { currentPalette, currentPaletteOffset, recordPaletteEdit } = get();
    if (!currentPalette) return;
    
    // Get the old color before updating
    const oldColor = currentPalette[index];
    
    const next = [...currentPalette];
    next[index] = newColor;
    set({ currentPalette: next });
    
    // Record the edit in history (convert Color to byte array)
    // SNES uses 15-bit BGR format: bbbbbgggggrrrrr
    const oldBytes = colorToSnesBytes(oldColor);
    const newBytes = colorToSnesBytes(newColor);
    const offset = pcOffset || currentPaletteOffset;
    
    if (offset) {
      recordPaletteEdit(offset, index, oldBytes, newBytes);
    }
  },

  exportAsset: async (asset, palette, path) => {
    try {
      // Icons are 32x32 pixels (4x4 tiles), Portraits are usually 128x128 (16x16 tiles)
      const widthTiles = asset.subtype === 'icon' ? 4 : 16;
      await invoke('export_asset_to_png', {
        pcOffset: asset.start_pc,
        size: asset.size,
        widthTiles,
        category: asset.category,
        palettePcOffset: palette.start_pc,
        paletteSize: palette.size,
        outputPath: path,
      });
      set({ error: null });
    } catch (e) {
      set({ error: (e as Error).toString() });
    }
  },

  importAsset: async (palette, path) => {
    try {
      const bytes = await invoke<number[]>('import_asset_from_png', {
        pngPath: path,
        palettePcOffset: palette.start_pc,
        paletteSize: palette.size,
      });
      set({ error: null });
      return new Uint8Array(bytes);
    } catch (e) {
      set({ error: (e as Error).toString() });
      return null;
    }
  },

  setPendingWrite: (pcOffset: string) => {
    set(state => ({ 
      pendingWrites: new Set([...state.pendingWrites, pcOffset]),
      isProjectModified: true 
    }));
  },

  removePendingWrite: (pcOffset: string) => {
    set(state => {
      const next = new Set(state.pendingWrites);
      next.delete(pcOffset);
      return { pendingWrites: next };
    });
  },

  clearPendingWrites: () => set({ pendingWrites: new Set() }),

  loadFighterList: async () => {
    try {
      const fighters = await invoke<FighterMetadata[]>('get_fighter_list');
      set({ fighters });
    } catch (e) {
      set({ error: (e as Error).toString() });
    }
  },

  selectFighter: async (id: number) => {
    try {
      const poses = await invoke<PoseInfo[]>('get_fighter_poses', { fighterId: id });
      set({ selectedFighterId: id, poses });
    } catch (e) {
      set({ error: (e as Error).toString() });
    }
  },

  renderPose: async (fighterId: number, poseId: number) => {
    try {
      const bytes = await invoke<number[]>('render_fighter_pose', { fighterId, poseId });
      return new Uint8Array(bytes);
    } catch (e) {
      set({ error: (e as Error).toString() });
      throw e;
    }
  },

  // Shared bank utilities
  getSharedBankInfo: async (pcOffset: string) => {
    try {
      const result = await invoke<{
        found: boolean;
        filename?: string;
        category?: string;
        size?: number;
        start_pc?: string;
        shared_with?: string[];
        all_fighters_using?: string[];
        is_shared?: boolean;
        message?: string;
      }>('get_shared_bank_info', { pcOffset });
      return result;
    } catch (e) {
      console.error('Failed to get shared bank info:', e);
      throw e;
    }
  },

  getFighterSharedBanks: async (fighterKey: string) => {
    try {
      const result = await invoke<{
        fighter: string;
        key: string;
        unique_bin_count: number;
        shared_bin_count: number;
        shared_bins: Array<{
          filename: string;
          start_pc: string;
          size: number;
          shared_with: string[];
        }>;
        shares_with: string[];
        is_safe_target: boolean;
      }>('get_fighter_shared_banks', { fighterKey });
      return result;
    } catch (e) {
      console.error('Failed to get fighter shared banks:', e);
      throw e;
    }
  },

  // Project management actions
  createProject: async (path: string, name: string, author?: string, description?: string) => {
    try {
      const project = await invoke<ProjectFile>('create_project', {
        projectPath: path,
        name,
        author: author || null,
        description: description || null,
      });
      set({ 
        currentProject: project, 
        currentProjectPath: path,
        isProjectModified: false,
        error: null 
      });
    } catch (e) {
      console.error('Failed to create project:', e);
      set({ error: (e as Error).toString() });
      throw e;
    }
  },

  saveProject: async (path?: string, metadata?: ProjectMetadata) => {
    try {
      const project = await invoke<ProjectFile>('save_project', {
        projectPath: path || null,
        metadata: metadata || null,
      });
      set({ 
        currentProject: project, 
        isProjectModified: false,
        error: null 
      });
    } catch (e) {
      console.error('Failed to save project:', e);
      set({ error: (e as Error).toString() });
      throw e;
    }
  },

  loadProject: async (path: string) => {
    try {
      const project = await invoke<ProjectFile>('load_project', { projectPath: path });
      set({ 
        currentProject: project, 
        currentProjectPath: path,
        isProjectModified: false,
        error: null 
      });
    } catch (e) {
      console.error('Failed to load project:', e);
      set({ error: (e as Error).toString() });
      throw e;
    }
  },

  validateProject: async (path: string) => {
    try {
      const isValid = await invoke<boolean>('validate_project', { projectPath: path });
      return isValid;
    } catch (e) {
      console.error('Failed to validate project:', e);
      return false;
    }
  },

  getCurrentProject: async () => {
    try {
      const project = await invoke<ProjectFile | null>('get_current_project');
      const path = await invoke<string | null>('get_current_project_path');
      set({ 
        currentProject: project, 
        currentProjectPath: path 
      });
    } catch (e) {
      console.error('Failed to get current project:', e);
    }
  },

  setProjectModified: (modified: boolean) => set({ isProjectModified: modified }),

  // Project thumbnail actions
  captureThumbnail: async (viewType: string) => {
    try {
      // Capture screenshot using DOM API
      const canvas = await captureScreenshot();
      if (!canvas) {
        throw new Error('Failed to capture screenshot');
      }
      
      // Convert canvas to PNG bytes
      const pngBlob = await new Promise<Blob | null>((resolve) => {
        canvas.toBlob((blob) => resolve(blob), 'image/png');
      });
      
      if (!pngBlob) {
        throw new Error('Failed to convert canvas to PNG');
      }
      
      const pngBytes = new Uint8Array(await pngBlob.arrayBuffer());
      
      // Send to backend for processing
      const result = await invoke<ProjectThumbnail>('capture_project_thumbnail', { 
        pngBytes: Array.from(pngBytes),
        viewType 
      });
      return result;
    } catch (e) {
      console.error('Failed to capture thumbnail:', e);
      set({ error: (e as Error).toString() });
      return null;
    }
  },

  saveThumbnail: async (thumbnail: ProjectThumbnail) => {
    try {
      await invoke('save_project_thumbnail', { thumbnailData: thumbnail });
      // Update current project with new thumbnail
      const { currentProject } = get();
      if (currentProject) {
        set({
          currentProject: {
            ...currentProject,
            thumbnail,
          },
        });
      }
    } catch (e) {
      console.error('Failed to save thumbnail:', e);
      set({ error: (e as Error).toString() });
      throw e;
    }
  },

  getThumbnail: async () => {
    try {
      const result = await invoke<ProjectThumbnail | null>('get_project_thumbnail');
      return result;
    } catch (e) {
      console.error('Failed to get thumbnail:', e);
      return null;
    }
  },

  clearThumbnail: async () => {
    try {
      await invoke('clear_project_thumbnail');
      // Update current project to remove thumbnail
      const { currentProject } = get();
      if (currentProject) {
        const { thumbnail: _, ...projectWithoutThumbnail } = currentProject;
        set({ currentProject: projectWithoutThumbnail });
      }
    } catch (e) {
      console.error('Failed to clear thumbnail:', e);
      set({ error: (e as Error).toString() });
      throw e;
    }
  },

  loadThumbnailFromPath: async (projectPath: string) => {
    try {
      const result = await invoke<ProjectThumbnail | null>('load_project_thumbnail_from_path', { projectPath });
      return result;
    } catch (e) {
      console.error('Failed to load thumbnail from path:', e);
      return null;
    }
  },

  // Undo/Redo actions
  undo: async () => {
    try {
      await invoke('undo');
      await get().refreshUndoState();
    } catch (e) {
      console.error('Undo failed:', e);
      set({ error: (e as Error).toString() });
    }
  },

  redo: async () => {
    try {
      await invoke('redo');
      await get().refreshUndoState();
    } catch (e) {
      console.error('Redo failed:', e);
      set({ error: (e as Error).toString() });
    }
  },

  refreshUndoState: async () => {
    try {
      const [canUndoResult, canRedoResult, undoStackResult, redoStackResult] = await Promise.all([
        invoke<boolean>('can_undo'),
        invoke<boolean>('can_redo'),
        invoke<EditSummary[]>('get_undo_stack'),
        invoke<EditSummary[]>('get_redo_stack'),
      ]);
      set({ 
        canUndo: canUndoResult, 
        canRedo: canRedoResult,
        undoStack: undoStackResult,
        redoStack: redoStackResult,
        editHistory: [...undoStackResult, ...redoStackResult]
      });
    } catch (e) {
      console.error('Failed to refresh undo state:', e);
    }
  },

  clearHistory: async () => {
    try {
      await invoke('clear_history');
      set({ canUndo: false, canRedo: false, editHistory: [], undoStack: [], redoStack: [] });
    } catch (e) {
      console.error('Failed to clear history:', e);
    }
  },

  // History recording helpers
  recordPaletteEdit: async (pcOffset: string, colorIndex: number, oldColor: number[], newColor: number[]) => {
    try {
      await invoke('record_palette_edit', {
        pcOffset,
        colorIndex,
        oldColor,
        newColor,
      });
      await get().refreshUndoState();
    } catch (e) {
      console.error('Failed to record palette edit:', e);
    }
  },

  recordSpriteBinEdit: async (pcOffset: string, oldBytes: number[], newBytes: number[]) => {
    try {
      await invoke('record_sprite_bin_edit', {
        pcOffset,
        oldBytes,
        newBytes,
      });
      await get().refreshUndoState();
    } catch (e) {
      console.error('Failed to record sprite bin edit:', e);
    }
  },

  recordAssetImport: async (pcOffset: string, oldBytes: number[], newBytes: number[], sourcePath: string) => {
    try {
      await invoke('record_asset_import', {
        pcOffset,
        oldBytes,
        newBytes,
        sourcePath,
      });
      await get().refreshUndoState();
    } catch (e) {
      console.error('Failed to record asset import:', e);
    }
  },

  // Bank duplication actions (stub implementations)
  loadBankDuplications: async () => {
    // Stub - duplications would be loaded from project file
    set({ bankDuplications: [] });
  },

  duplicateSharedBank: async () => {
    // Stub - full implementation would call a Tauri command
    return { success: false, error: 'Bank duplication not yet implemented' };
  },

  // Animation actions
  loadFighterAnimations: async (fighterId: number) => {
    try {
      const result = await invoke<FighterAnimations>('get_fighter_animations', { fighterId });
      set({ 
        fighterAnimations: result, 
        selectedAnimationIndex: 0, 
        selectedFrameIndex: 0 
      });
    } catch (e) {
      console.error('Failed to load fighter animations:', e);
      set({ error: (e as Error).toString() });
    }
  },

  selectAnimation: (index: number) => {
    set({ selectedAnimationIndex: index, selectedFrameIndex: 0 });
  },

  selectFrame: (index: number) => {
    set({ selectedFrameIndex: index });
  },

  updateAnimationFrame: (frameIndex: number, updates: Partial<AnimationFrame>) => {
    const { fighterAnimations, selectedAnimationIndex } = get();
    if (!fighterAnimations) return;

    const newAnimations = [...fighterAnimations.animations];
    const anim = { ...newAnimations[selectedAnimationIndex] };
    const newFrames = [...anim.frames];
    newFrames[frameIndex] = { ...newFrames[frameIndex], ...updates };
    anim.frames = newFrames;
    newAnimations[selectedAnimationIndex] = anim;

    set({ 
      fighterAnimations: { ...fighterAnimations, animations: newAnimations } 
    });
  },

  addAnimationFrame: () => {
    const { fighterAnimations, selectedAnimationIndex } = get();
    if (!fighterAnimations) return;

    const newAnimations = [...fighterAnimations.animations];
    const anim = { ...newAnimations[selectedAnimationIndex] };
    const newFrame: AnimationFrame = {
      pose_id: 0,
      duration: 4,
      tileset_id: 0,
      effects: [],
    };
    anim.frames = [...anim.frames, newFrame];
    newAnimations[selectedAnimationIndex] = anim;

    set({ 
      fighterAnimations: { ...fighterAnimations, animations: newAnimations },
      selectedFrameIndex: anim.frames.length - 1,
    });
  },

  removeAnimationFrame: (frameIndex: number) => {
    const { fighterAnimations, selectedAnimationIndex, selectedFrameIndex } = get();
    if (!fighterAnimations) return;

    const anim = fighterAnimations.animations[selectedAnimationIndex];
    if (anim.frames.length <= 1) return;

    const newAnimations = [...fighterAnimations.animations];
    const newAnim = { ...newAnimations[selectedAnimationIndex] };
    const newFrames = [...newAnim.frames];
    newFrames.splice(frameIndex, 1);
    newAnim.frames = newFrames;
    newAnimations[selectedAnimationIndex] = newAnim;

    // Adjust selected frame if needed
    let newFrameIndex = selectedFrameIndex;
    if (selectedFrameIndex >= newFrames.length) {
      newFrameIndex = Math.max(0, newFrames.length - 1);
    }

    set({ 
      fighterAnimations: { ...fighterAnimations, animations: newAnimations },
      selectedFrameIndex: newFrameIndex,
    });
  },

  // Comparison actions
  generateComparison: async () => {
    set({ comparisonLoading: true });
    try {
      const result = await invoke<RomComparison>('generate_comparison');
      set({ comparison: result, comparisonLoading: false });
    } catch (e) {
      console.error('Failed to generate comparison:', e);
      set({ error: (e as Error).toString(), comparisonLoading: false });
    }
  },

  selectComparisonAsset: (assetId: string | null) => {
    set({ selectedComparisonAsset: assetId });
  },

  setComparisonViewMode: (mode: 'side-by-side' | 'overlay' | 'difference' | 'split' | 'blink') => {
    set({ comparisonViewMode: mode });
  },

  getPaletteDiff: async (pcOffset: string) => {
    try {
      return await invoke<PaletteDiff>('get_palette_diff', { pcOffset });
    } catch (e) {
      console.error('Failed to get palette diff:', e);
      return null;
    }
  },

  getSpriteBinDiff: async (pcOffset: string) => {
    try {
      return await invoke<SpriteDiff>('get_sprite_bin_diff_comparison', { pcOffset });
    } catch (e) {
      console.error('Failed to get sprite bin diff:', e);
      return null;
    }
  },

  getBinaryDiff: async (pcOffset: string, size: number) => {
    try {
      return await invoke<BinaryDiff>('get_binary_diff', { pcOffset, size });
    } catch (e) {
      console.error('Failed to get binary diff:', e);
      return null;
    }
  },

  renderComparisonView: async (params: ComparisonRenderParams) => {
    try {
      const bytes = await invoke<number[]>('render_comparison_view', {
        boxerKey: params.boxer_key,
        viewType: params.view_type,
        showOriginal: params.show_original,
        showModified: params.show_modified,
        assetOffset: params.asset_offset,
        paletteOffset: params.palette_offset,
        mode: params.mode,
      });
      return new Uint8Array(bytes);
    } catch (e) {
      console.error('Failed to render comparison view:', e);
      return null;
    }
  },

  exportComparisonReport: async (outputPath: string, format: 'json' | 'html' | 'text') => {
    try {
      await invoke('export_comparison_report', { outputPath, format });
    } catch (e) {
      console.error('Failed to export comparison report:', e);
      throw e;
    }
  },

  // Patch notes actions
  generatePatchNotes: async (format: string, title?: string, author?: string, version?: string) => {
    try {
      const content = await invoke<string>('generate_patch_notes', {
        format,
        title: title || null,
        author: author || null,
        version: version || null,
      });
      set({ patchNotesContent: content });
      return content;
    } catch (e) {
      console.error('Failed to generate patch notes:', e);
      set({ error: (e as Error).toString() });
      return null;
    }
  },

  getChangeSummary: async () => {
    try {
      const summary = await invoke<{
        total_boxers_modified: number;
        total_palettes_changed: number;
        total_sprites_edited: number;
        total_animations_modified: number;
        total_headers_edited: number;
        total_changes: number;
      }>('get_change_summary');
      set({ changeSummary: summary });
    } catch (e) {
      console.error('Failed to get change summary:', e);
    }
  },

  savePatchNotes: async (content: string, outputPath: string) => {
    try {
      await invoke('save_patch_notes', { content, outputPath });
    } catch (e) {
      console.error('Failed to save patch notes:', e);
      set({ error: (e as Error).toString() });
    }
  },

  // Boxer comparison actions
  compareBoxers: async (boxerAKey: string, boxerBKey: string) => {
    try {
      const result = await invoke('compare_boxers', {
        boxerAKey,
        boxerBKey,
      });
      return result;
    } catch (e) {
      console.error('Failed to compare boxers:', e);
      throw e;
    }
  },

  getSimilarBoxers: async (referenceKey: string, limit?: number) => {
    try {
      const result = await invoke('get_similar_boxers', {
        referenceKey,
        limit: limit || 5,
      });
      return result;
    } catch (e) {
      console.error('Failed to get similar boxers:', e);
      throw e;
    }
  },

  copyBoxerStat: async (sourceKey: string, targetKey: string, statField: string) => {
    try {
      await invoke('copy_boxer_stat', {
        sourceKey,
        targetKey,
        statField,
      });
    } catch (e) {
      console.error('Failed to copy boxer stat:', e);
      throw e;
    }
  },

  copyAllBoxerStats: async (sourceKey: string, targetKey: string) => {
    try {
      await invoke('copy_all_boxer_stats', {
        sourceKey,
        targetKey,
      });
    } catch (e) {
      console.error('Failed to copy all boxer stats:', e);
      throw e;
    }
  },

  // Frame reconstructor actions
  loadFrames: async (fighterId: number) => {
    try {
      const frames = await invoke<FrameSummary[]>('get_fighter_frames', { fighterId });
      set({ frames });
    } catch (e) {
      console.error('Failed to load frames:', e);
      set({ error: (e as Error).toString() });
    }
  },

  loadFrameDetail: async (fighterId: number, frameIndex: number) => {
    try {
      const frame = await invoke<FrameData>('get_frame_detail', { fighterId, frameIndex });
      set({ currentFrame: frame, currentFrameIndex: frameIndex });
      return frame;
    } catch (e) {
      console.error('Failed to load frame detail:', e);
      set({ error: (e as Error).toString() });
      return null;
    }
  },

  saveFrame: async (fighterId: number, frameIndex: number, frame: FrameData) => {
    try {
      await invoke('save_frame', { fighterId, frameIndex, frame });
    } catch (e) {
      console.error('Failed to save frame:', e);
      set({ error: (e as Error).toString() });
    }
  },

  addSpriteToFrame: async (fighterId: number, frameIndex: number, tileId: number, x: number, y: number) => {
    try {
      const frame = await invoke<FrameData>('add_sprite_to_frame', { fighterId, frameIndex, tileId, x, y });
      set({ currentFrame: frame });
      return frame;
    } catch (e) {
      console.error('Failed to add sprite:', e);
      set({ error: (e as Error).toString() });
      return null;
    }
  },

  moveSprite: async (fighterId: number, frameIndex: number, spriteIndex: number, x: number, y: number) => {
    try {
      const frame = await invoke<FrameData>('move_sprite', { fighterId, frameIndex, spriteIndex, x, y });
      set({ currentFrame: frame });
      return frame;
    } catch (e) {
      console.error('Failed to move sprite:', e);
      set({ error: (e as Error).toString() });
      return null;
    }
  },

  updateSpriteFlags: async (fighterId: number, frameIndex: number, spriteIndex: number, hFlip: boolean, vFlip: boolean, palette: number) => {
    try {
      const frame = await invoke<FrameData>('update_sprite_flags', { fighterId, frameIndex, spriteIndex, hFlip, vFlip, palette });
      set({ currentFrame: frame });
      return frame;
    } catch (e) {
      console.error('Failed to update sprite flags:', e);
      set({ error: (e as Error).toString() });
      return null;
    }
  },

  removeSprite: async (fighterId: number, frameIndex: number, spriteIndex: number) => {
    try {
      const frame = await invoke<FrameData>('remove_sprite', { fighterId, frameIndex, spriteIndex });
      set({ currentFrame: frame });
      return frame;
    } catch (e) {
      console.error('Failed to remove sprite:', e);
      set({ error: (e as Error).toString() });
      return null;
    }
  },

  duplicateSprite: async (fighterId: number, frameIndex: number, spriteIndex: number) => {
    try {
      const frame = await invoke<FrameData>('duplicate_sprite', { fighterId, frameIndex, spriteIndex });
      set({ currentFrame: frame });
      return frame;
    } catch (e) {
      console.error('Failed to duplicate sprite:', e);
      set({ error: (e as Error).toString() });
      return null;
    }
  },

  renderFramePreview: async (fighterId: number, frameIndex: number) => {
    try {
      const bytes = await invoke<number[]>('render_frame_preview', { fighterId, frameIndex });
      return new Uint8Array(bytes);
    } catch (e) {
      console.error('Failed to render frame preview:', e);
      set({ error: (e as Error).toString() });
      throw e;
    }
  },

  // External tools actions
  loadExternalTools: async () => {
    try {
      const tools = await invoke<ExternalTool[]>('get_external_tools');
      set({ externalTools: tools });
    } catch (e) {
      console.error('Failed to load external tools:', e);
    }
  },

  addExternalTool: async (tool: ExternalTool) => {
    try {
      await invoke('add_external_tool', { tool });
      await get().loadExternalTools();
    } catch (e) {
      console.error('Failed to add external tool:', e);
      throw e;
    }
  },

  removeExternalTool: async (toolId: string) => {
    try {
      await invoke('remove_external_tool', { toolId });
      await get().loadExternalTools();
    } catch (e) {
      console.error('Failed to remove external tool:', e);
      throw e;
    }
  },

  updateExternalTool: async (tool: ExternalTool) => {
    try {
      await invoke('update_external_tool', { tool });
      await get().loadExternalTools();
    } catch (e) {
      console.error('Failed to update external tool:', e);
      throw e;
    }
  },

  launchWithTool: async (toolId: string, filePath: string, context?: ToolContext) => {
    try {
      await invoke('launch_with_tool', { toolId, filePath, context });
    } catch (e) {
      console.error('Failed to launch tool:', e);
      throw e;
    }
  },

  getCompatibleTools: async (fileExtension: string) => {
    try {
      const tools = await invoke<ExternalTool[]>('get_compatible_tools', { fileExtension });
      return tools;
    } catch (e) {
      console.error('Failed to get compatible tools:', e);
      return [];
    }
  },

  setDefaultTool: async (fileExtension: string, toolId: string) => {
    try {
      await invoke('set_default_tool', { fileExtension, toolId });
      const { defaultToolIds } = get();
      set({ defaultToolIds: { ...defaultToolIds, [fileExtension]: toolId } });
    } catch (e) {
      console.error('Failed to set default tool:', e);
      throw e;
    }
  },

  getDefaultTool: async (fileExtension: string) => {
    try {
      const tool = await invoke<ExternalTool | null>('get_default_tool', { fileExtension });
      return tool;
    } catch (e) {
      console.error('Failed to get default tool:', e);
      return null;
    }
  },

  verifyTool: async (tool: ExternalTool) => {
    try {
      const result = await invoke<{ valid: boolean; message: string }>('verify_tool', { tool });
      return result;
    } catch (e) {
      console.error('Failed to verify tool:', e);
      return { valid: false, message: String(e) };
    }
  },

  // Frame annotation actions
  addFrameAnnotation: async (fighterId: number, frameIndex: number, x: number, y: number, text: string, category: string) => {
    try {
      await invoke('add_frame_annotation', { fighterId, frameIndex, x, y, text, category });
    } catch (e) {
      console.error('Failed to add frame annotation:', e);
      throw e;
    }
  },

  removeFrameAnnotation: async (fighterId: number, frameIndex: number, annotationId: string) => {
    try {
      await invoke('remove_frame_annotation', { fighterId, frameIndex, annotationId });
    } catch (e) {
      console.error('Failed to remove frame annotation:', e);
      throw e;
    }
  },

  updateFrameAnnotation: async (fighterId: number, frameIndex: number, annotationId: string, updates: Partial<FrameAnnotation>) => {
    try {
      await invoke('update_frame_annotation', { fighterId, frameIndex, annotationId, updates });
    } catch (e) {
      console.error('Failed to update frame annotation:', e);
      throw e;
    }
  },

  getFrameAnnotations: async (fighterId: number, frameIndex: number) => {
    try {
      const annotations = await invoke<FrameAnnotation[]>('get_frame_annotations', { fighterId, frameIndex });
      return annotations;
    } catch (e) {
      console.error('Failed to get frame annotations:', e);
      return [];
    }
  },

  applyInGameExpansion: async (options: InGameExpansionOptions) => {
    try {
      const report = await invoke<InGameExpansionReport>('apply_in_game_expansion', {
        request: {
          target_boxer_count: options.targetBoxerCount,
          patch_editor_hook: options.patchEditorHook ?? false,
          editor_hook_pc_offset: options.editorHookPcOffset ?? null,
          editor_hook_overwrite_len: options.editorHookOverwriteLen ?? null,
        },
      });
      set({ isProjectModified: true, error: null });
      return report;
    } catch (e) {
      const message = String(e);
      console.error('Failed to apply in-game expansion:', e);
      set({ error: message });
      throw e;
    }
  },

  analyzeInGameHookSites: async (options) => {
    try {
      const request = {
        start_pc_offset: options?.startPcOffset ?? null,
        end_pc_offset: options?.endPcOffset ?? null,
        limit: options?.limit ?? 25,
      };
      return await invoke<InGameHookSiteCandidate[]>('analyze_in_game_hook_sites', { request });
    } catch (e) {
      console.error('Failed to analyze in-game hook sites:', e);
      throw e;
    }
  },

  verifyInGameHookSite: async (options) => {
    try {
      const request = {
        hook_pc_offset: options.hookPcOffset,
        overwrite_len: options.overwriteLen ?? null,
      };
      return await invoke<InGameHookSiteCandidate>('verify_in_game_hook_site', { request });
    } catch (e) {
      console.error('Failed to verify in-game hook site:', e);
      throw e;
    }
  },

  getInGameHookPresets: async (limit = 8) => {
    try {
      return await invoke<InGameHookPreset[]>('get_in_game_hook_presets', { limit });
    } catch (e) {
      console.error('Failed to get in-game hook presets:', e);
      throw e;
    }
  },
  
  // Update actions implementation
  loadUpdateSettings: async () => {
    try {
      const settings = await invoke<UpdateSettings>('get_update_settings');
      const version = await invoke<string>('get_current_version');
      set({ updateSettings: settings, currentVersion: version });
    } catch (e) {
      console.error('Failed to load update settings:', e);
    }
  },
  
  saveUpdateSettings: async (settings: UpdateSettings) => {
    try {
      await invoke('set_update_settings', { settings });
      set({ updateSettings: settings });
    } catch (e) {
      console.error('Failed to save update settings:', e);
    }
  },
  
  checkForUpdates: async () => {
    set({ checkingForUpdate: true, updateError: null });
    try {
      const update = await invoke<UpdateInfo | null>('check_for_updates');
      set({ availableUpdate: update, checkingForUpdate: false });
      return update;
    } catch (e) {
      console.error('Failed to check for updates:', e);
      set({ checkingForUpdate: false, updateError: String(e) });
      return null;
    }
  },
  
  skipVersion: async (version: string) => {
    try {
      await invoke('skip_version', { version });
      const { updateSettings } = get();
      set({ 
        updateSettings: { 
          ...updateSettings, 
          skipped_versions: [...updateSettings.skipped_versions, version] 
        },
        availableUpdate: null 
      });
    } catch (e) {
      console.error('Failed to skip version:', e);
    }
  },
  
  downloadAndInstallUpdate: async () => {
    set({ downloadProgress: { ...get().downloadProgress, state: 'downloading' } });
    try {
      await invoke('download_and_install_update');
      set({ downloadProgress: { percent: 100, downloaded: 0, total: 0, state: 'ready' } });
    } catch (e) {
      console.error('Failed to download and install update:', e);
      set({ 
        downloadProgress: { percent: 0, downloaded: 0, total: 0, state: 'error' },
        updateError: String(e)
      });
    }
  },
  
  clearSkippedVersions: async () => {
    try {
      await invoke('clear_skipped_versions');
      const { updateSettings } = get();
      set({ 
        updateSettings: { 
          ...updateSettings, 
          skipped_versions: [] 
        } 
      });
    } catch (e) {
      console.error('Failed to clear skipped versions:', e);
    }
  },
  
  shouldAutoCheck: async () => {
    try {
      return await invoke<boolean>('should_auto_check');
    } catch (e) {
      console.error('Failed to check auto update:', e);
      return false;
    }
  },
}));
