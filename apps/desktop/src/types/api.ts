/**
 * API Types
 * 
 * TypeScript interfaces for V4 Tauri commands in the Super Punch-Out!! Editor.
 */

// ============================================================================
// Plugin System Types
// ============================================================================

/** Information about a loaded plugin */
export interface PluginInfo {
  /** Unique plugin identifier */
  id: string;
  /** Display name of the plugin */
  name: string;
  /** Plugin version string */
  version: string;
  /** Plugin author */
  author: string;
  /** Plugin description */
  description: string;
  /** Whether the plugin is currently enabled */
  enabled: boolean;
}

/** Result of executing a script */
export interface ScriptExecutionResult {
  /** Whether the script executed successfully */
  success: boolean;
  /** Script output (stdout) */
  output: string;
  /** Error message if execution failed */
  error: string | null;
  /** Execution time in milliseconds */
  execution_time_ms: number;
}

/** Information about a batch job */
export interface BatchJobInfo {
  /** Unique job identifier */
  id: string;
  /** Job name/description */
  name: string;
  /** Current status of the job */
  status: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';
  /** Progress percentage (0-100) */
  progress: number;
  /** Job creation timestamp */
  created_at: string;
  /** Job completion timestamp (if finished) */
  completed_at: string | null;
  /** Error message if job failed */
  error: string | null;
}

// ============================================================================
// Bank Management Types
// ============================================================================

/** A single entry in the bank map */
export interface BankMapEntry {
  /** Bank number (0-127 for LoROM) */
  bank: number;
  /** Starting address within the bank */
  start_addr: number;
  /** Ending address within the bank */
  end_addr: number;
  /** Size of the region in bytes */
  size: number;
  /** Type of data stored in this region */
  data_type: string;
  /** Description of the content */
  description: string;
}

/** Complete bank map showing all banks and their usage */
export interface BankMap {
  /** ROM type (lorom, hirom, etc.) */
  rom_type: string;
  /** Total ROM size in bytes */
  total_size: number;
  /** Number of banks */
  bank_count: number;
  /** Size of each bank */
  bank_size: number;
  /** Array of bank entries */
  entries: BankMapEntry[];
}

/** Statistics for a single bank */
export interface BankStats {
  /** Bank number */
  bank: number;
  /** Total capacity of the bank */
  capacity: number;
  /** Used bytes */
  used: number;
  /** Free bytes available */
  free: number;
  /** Usage percentage (0-100) */
  usage_percent: number;
  /** Number of allocated regions */
  region_count: number;
}

/** Overall bank statistics */
export interface BankStatistics {
  /** Total ROM capacity */
  total_capacity: number;
  /** Total used bytes across all banks */
  total_used: number;
  /** Total free bytes across all banks */
  total_free: number;
  /** Overall usage percentage */
  overall_usage_percent: number;
  /** Statistics per bank */
  banks: BankStats[];
}

/** A free/allocated memory region */
export interface MemoryRegion {
  /** Bank number */
  bank: number;
  /** Starting address */
  start_addr: number;
  /** Ending address (exclusive) */
  end_addr: number;
  /** Size in bytes */
  size: number;
  /** Whether this region is free or allocated */
  is_free: boolean;
  /** Data type if allocated */
  data_type: string | null;
  /** Description if allocated */
  description: string | null;
}

/** Fragmentation analysis result */
export interface FragmentationAnalysis {
  /** Total free bytes available */
  total_free_bytes: number;
  /** Number of free regions */
  free_region_count: number;
  /** Average free region size */
  avg_free_region_size: number;
  /** Largest contiguous free region */
  largest_free_region: MemoryRegion | null;
  /** Fragmentation score (0-100, higher = more fragmented) */
  fragmentation_score: number;
  /** Regions that could be consolidated */
  consolidation_opportunities: Array<{
    regions: MemoryRegion[];
    potential_savings: number;
  }>;
}

/** A single step in a defragmentation plan */
export interface DefragStep {
  /** Step number */
  step: number;
  /** Description of the operation */
  description: string;
  /** Source region */
  source: MemoryRegion;
  /** Destination region */
  destination: MemoryRegion;
  /** Size of data to move */
  size: number;
}

/** Complete defragmentation plan */
export interface DefragPlan {
  /** Whether defragmentation is recommended */
  recommended: boolean;
  /** Current fragmentation score */
  current_score: number;
  /** Projected score after defrag */
  projected_score: number;
  /** Total bytes that would be moved */
  total_bytes_to_move: number;
  /** Number of operations required */
  operation_count: number;
  /** Estimated time in milliseconds */
  estimated_time_ms: number;
  /** List of operations to perform */
  steps: DefragStep[];
  /** Warnings about potential issues */
  warnings: string[];
}

// ============================================================================
// Animation System Types
// ============================================================================

/** Data for a single animation frame */
export interface AnimationFrameData {
  /** Frame index within the animation */
  frame_index: number;
  /** Duration of this frame in game frames */
  duration: number;
  /** Sprite tile data references */
  sprite_tiles: SpriteTileRef[];
  /** Hitboxes for this frame */
  hitboxes: HitboxData[];
  /** Hurtboxes for this frame */
  hurtboxes: HurtboxData[];
  /** Frame-specific metadata */
  metadata: Record<string, unknown>;
}

/** Reference to a sprite tile */
export interface SpriteTileRef {
  /** Tile ID in VRAM */
  tile_id: number;
  /** X position on screen */
  x: number;
  /** Y position on screen */
  y: number;
  /** Palette number */
  palette: number;
  /** Horizontal flip */
  h_flip: boolean;
  /** Vertical flip */
  v_flip: boolean;
  /** Priority level */
  priority: number;
  /** Size: 0 = 8x8, 1 = 16x16, etc. */
  size: number;
}

/** Hitbox data for collision detection */
export interface HitboxData {
  /** Hitbox identifier within the frame */
  id: number;
  /** X position relative to entity */
  x: number;
  /** Y position relative to entity */
  y: number;
  /** Width of hitbox */
  width: number;
  /** Height of hitbox */
  height: number;
  /** Damage dealt on hit */
  damage: number;
  /** Stun duration */
  stun: number;
  /** Knockback direction */
  knockback_dir: number;
  /** Knockback strength */
  knockback_power: number;
  /** Hitbox type/flags */
  flags: number;
}

/** Hurtbox data for receiving damage */
export interface HurtboxData {
  /** Hurtbox identifier within the frame */
  id: number;
  /** X position relative to entity */
  x: number;
  /** Y position relative to entity */
  y: number;
  /** Width of hurtbox */
  width: number;
  /** Height of hurtbox */
  height: number;
  /** Vulnerability flags */
  vulnerability: number;
  /** Armor value (damage reduction) */
  armor: number;
}

/** Complete animation data */
export interface AnimationData {
  /** Animation identifier */
  name: string;
  /** Human-readable description */
  description: string;
  /** Total number of frames */
  frame_count: number;
  /** Frames per second */
  fps: number;
  /** Whether animation loops */
  loops: boolean;
  /** Frame data */
  frames: AnimationFrameData[];
}

/** Animation player state */
export interface AnimationPlayerState {
  /** Currently playing animation name */
  current_animation: string | null;
  /** Current frame index */
  current_frame: number;
  /** Whether animation is playing */
  is_playing: boolean;
  /** Whether animation is paused */
  is_paused: boolean;
  /** Current playback time in milliseconds */
  current_time_ms: number;
  /** Total animation duration in milliseconds */
  total_duration_ms: number;
  /** Playback speed multiplier */
  speed: number;
  /** Whether currently looping */
  is_looping: boolean;
}

// ============================================================================
// Boxer/Entity Types
// ============================================================================

/** Boxer identifier */
export type BoxerKey = 
  | 'mac' 
  | 'gabby_jay' 
  | 'bear_hugger' 
  | 'piston_hurricane' 
  | 'bald_bull' 
  | 'bob_charlie' 
  | 'dragon_chan' 
  | 'mask_muscle' 
  | 'mr_sandman' 
  | 'aron_remes' 
  | 'super_macho_man' 
  | 'nick_bruiser' 
  | 'rick_bruiser';

/** Boxer information */
export interface BoxerInfo {
  /** Boxer key identifier */
  key: BoxerKey;
  /** Display name */
  name: string;
  /** Circuit/class */
  circuit: 'minor' | 'major' | 'world' | 'special';
  /** Available animations */
  animations: string[];
}
