/**
 * Frame Tagging / Annotation System Types
 * 
 * TypeScript interfaces for the frame tagging and annotation feature.
 */

/** Tag categories */
export type TagCategory = 
  | 'idle' 
  | 'attack' 
  | 'defense' 
  | 'damage' 
  | 'knockdown' 
  | 'special' 
  | 'misc';

/** Category display info */
export interface TagCategoryInfo {
  value: TagCategory;
  label: string;
  icon: string;
  color: string;
}

/** A frame tag definition */
export interface FrameTag {
  id: string;
  name: string;
  display_name: string;
  description: string;
  category: TagCategory;
  color: string;
  created_at: string;
  created_by: string;
}

/** Annotation for a specific frame */
export interface FrameAnnotation {
  frame_index: number;
  tags: string[];      // Tag IDs
  notes: string;       // Freeform notes
  hitbox_description?: string;
}

/** Collection of annotations for a boxer */
export interface BoxerAnnotations {
  boxer_id: string;
  boxer_name: string;
  frame_annotations: Record<string, FrameAnnotation>;
}

/** @deprecated Use BoxerAnnotations instead */
export type FighterAnnotations = BoxerAnnotations;

/** Tag with its applied status */
export interface TagWithStatus extends FrameTag {
  applied: boolean;
}

/** Search result for annotations */
export interface AnnotationSearchResult {
  frame_index: number;
  annotation: FrameAnnotation;
}

/** Category definitions with metadata */
export const TAG_CATEGORIES: TagCategoryInfo[] = [
  { value: 'idle', label: 'Idle', icon: '😐', color: '#4ade80' },
  { value: 'attack', label: 'Attack', icon: '👊', color: '#f87171' },
  { value: 'defense', label: 'Defense', icon: '🛡️', color: '#60a5fa' },
  { value: 'damage', label: 'Damage', icon: '💥', color: '#fbbf24' },
  { value: 'knockdown', label: 'Knockdown', icon: '💫', color: '#a78bfa' },
  { value: 'special', label: 'Special', icon: '✨', color: '#f472b6' },
  { value: 'misc', label: 'Misc', icon: '📌', color: '#9ca3af' },
];

/** Default/predefined tags for Super Punch-Out!! */
export const DEFAULT_TAGS: Omit<FrameTag, 'created_at' | 'created_by'>[] = [
  // Idle
  { id: 'idle_stance', name: 'idle_stance', display_name: 'Idle Stance', description: 'Neutral standing pose', category: 'idle', color: '#4ade80' },
  { id: 'idle_alt', name: 'idle_alt', display_name: 'Idle Alt', description: 'Alternative idle pose', category: 'idle', color: '#4ade80' },
  
  // Attack
  { id: 'left_jab', name: 'left_jab', display_name: 'Left Jab', description: 'Quick left jab attack', category: 'attack', color: '#f87171' },
  { id: 'right_jab', name: 'right_jab', display_name: 'Right Jab', description: 'Quick right jab attack', category: 'attack', color: '#f87171' },
  { id: 'left_hook', name: 'left_hook', display_name: 'Left Hook', description: 'Powerful left hook attack', category: 'attack', color: '#f87171' },
  { id: 'right_hook', name: 'right_hook', display_name: 'Right Hook', description: 'Powerful right hook attack', category: 'attack', color: '#f87171' },
  { id: 'uppercut', name: 'uppercut', display_name: 'Uppercut', description: 'Uppercut attack', category: 'attack', color: '#f87171' },
  
  // Defense
  { id: 'dodge_left', name: 'dodge_left', display_name: 'Dodge Left', description: 'Dodge to the left', category: 'defense', color: '#60a5fa' },
  { id: 'dodge_right', name: 'dodge_right', display_name: 'Dodge Right', description: 'Dodge to the right', category: 'defense', color: '#60a5fa' },
  { id: 'duck', name: 'duck', display_name: 'Duck', description: 'Duck down to avoid high attacks', category: 'defense', color: '#60a5fa' },
  { id: 'block', name: 'block', display_name: 'Block', description: 'Generic block pose', category: 'defense', color: '#60a5fa' },
  { id: 'block_high', name: 'block_high', display_name: 'Block High', description: 'Block high attacks', category: 'defense', color: '#60a5fa' },
  { id: 'block_low', name: 'block_low', display_name: 'Block Low', description: 'Block low attacks', category: 'defense', color: '#60a5fa' },
  
  // Damage
  { id: 'hit_face', name: 'hit_face', display_name: 'Hit Face', description: 'Hit in the face', category: 'damage', color: '#fbbf24' },
  { id: 'hit_body', name: 'hit_body', display_name: 'Hit Body', description: 'Hit in the body', category: 'damage', color: '#fbbf24' },
  
  // Knockdown
  { id: 'knockdown', name: 'knockdown', display_name: 'Knockdown', description: 'Getting knocked down', category: 'knockdown', color: '#a78bfa' },
  { id: 'getup', name: 'getup', display_name: 'Get Up', description: 'Getting up from knockdown', category: 'knockdown', color: '#a78bfa' },
  
  // Special
  { id: 'taunt', name: 'taunt', display_name: 'Taunt', description: 'Taunting the player', category: 'special', color: '#f472b6' },
  { id: 'special_move', name: 'special_move', display_name: 'Special Move', description: 'Special attack or move', category: 'special', color: '#f472b6' },
  
  // Misc
  { id: 'neutral', name: 'neutral', display_name: 'Neutral', description: 'Neutral expression/stance', category: 'misc', color: '#9ca3af' },
  { id: 'intro', name: 'intro', display_name: 'Intro', description: 'Introduction pose', category: 'misc', color: '#9ca3af' },
  { id: 'victory', name: 'victory', display_name: 'Victory', description: 'Victory celebration', category: 'misc', color: '#9ca3af' },
  { id: 'defeat', name: 'defeat', display_name: 'Defeat', description: 'Defeat pose', category: 'misc', color: '#9ca3af' },
];

/** Get category info by value */
export function getCategoryInfo(category: TagCategory): TagCategoryInfo {
  return TAG_CATEGORIES.find(c => c.value === category) || TAG_CATEGORIES[6];
}

/** Get color for a category */
export function getCategoryColor(category: TagCategory): string {
  return getCategoryInfo(category).color;
}

/** Get icon for a category */
export function getCategoryIcon(category: TagCategory): string {
  return getCategoryInfo(category).icon;
}
