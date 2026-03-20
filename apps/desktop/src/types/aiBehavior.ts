/**
 * AI Behavior types for Super Punch-Out!! editor
 */

export type MoveType = 
  | 'left_jab' 
  | 'right_jab' 
  | 'left_hook' 
  | 'right_hook' 
  | 'left_uppercut' 
  | 'right_uppercut' 
  | 'special' 
  | 'taunt'
  | 'step_left'
  | 'step_right'
  | 'step_forward'
  | 'step_back';

export type DefenseType = 
  | 'dodge_left' 
  | 'dodge_right' 
  | 'duck' 
  | 'block_high' 
  | 'block_low' 
  | 'counter' 
  | 'sway_back' 
  | 'clinch';

export type HeightZone = 'high' | 'mid' | 'low';

export type DifficultyRating = 'VeryEasy' | 'Easy' | 'Medium' | 'Hard' | 'VeryHard' | 'Extreme';

export interface Hitbox {
  x: number;
  y: number;
  width: number;
  height: number;
  height_zone: HeightZone;
}

export interface AttackMove {
  move_type: MoveType;
  windup_frames: number;
  active_frames: number;
  recovery_frames: number;
  damage: number;
  stun: number;
  hitbox: Hitbox;
  pose_id: number;
  sound_effect: number | null;
}

export interface AttackPattern {
  id: string;
  name: string;
  sequence: AttackMove[];
  frequency: number;
  conditions: Condition[];
  difficulty_min: number;
  difficulty_max: number;
  available_round_1: boolean;
  available_round_2: boolean;
  available_round_3: boolean;
  weight: number;
}

export interface DefenseBehavior {
  behavior_type: DefenseType;
  frequency: number;
  conditions: Condition[];
  success_rate: number;
  recovery_frames: number;
  leads_to_counter: boolean;
  counter_pattern_id: string | null;
}

export interface RoundDifficulty {
  round: number;
  aggression: number;
  defense: number;
  speed: number;
  pattern_complexity: number;
  damage_multiplier: number;
  reaction_time: number;
}

export interface DifficultyCurve {
  rounds: RoundDifficulty[];
  base_aggression: number;
  base_defense: number;
  base_speed: number;
}

export type Condition =
  | { type: 'health_below'; value: number }
  | { type: 'health_above'; value: number }
  | { type: 'round'; value: number }
  | { type: 'time_below'; value: number }
  | { type: 'player_stunned' }
  | { type: 'player_blocking' }
  | { type: 'random_chance'; value: number }
  | { type: 'combo_count'; value: number }
  | { type: 'player_missed' }
  | { type: 'player_attacking' }
  | { type: 'player_using'; move_type: MoveType }
  | { type: 'player_health_below'; value: number }
  | { type: 'times_hit'; value: number }
  | { type: 'always' }
  | { type: 'all'; conditions: Condition[] }
  | { type: 'any'; conditions: Condition[] };

export type AiAction =
  | { type: 'use_pattern'; pattern_id: string }
  | { type: 'change_behavior'; behavior_id: string }
  | { type: 'taunt' }
  | { type: 'special_move' }
  | { type: 'defend'; defense_type: DefenseType }
  | { type: 'move'; direction: 'left' | 'right' | 'forward' | 'back' }
  | { type: 'reset_behavior' }
  | { type: 'sequence'; actions: AiAction[] };

export interface AiTrigger {
  condition: Condition;
  action: AiAction;
  priority: number;
  cooldown: number;
  once_per_round: boolean;
}

export interface AiBehavior {
  boxer_id: number;
  boxer_name: string;
  attack_patterns: AttackPattern[];
  defense_behaviors: DefenseBehavior[];
  difficulty_curve: DifficultyCurve;
  triggers: AiTrigger[];
  raw_bytes: number[];
  pc_offset: number | null;
}

export interface SimulationResult {
  average_fight_time: number;
  player_damage_taken: number;
  ai_damage_dealt: number;
  pattern_usage: Record<string, number>;
  difficulty_rating: DifficultyRating;
  estimated_win_rate: number;
  insights: string[];
  warnings: string[];
}

export interface MoveTypeOption {
  id: string;
  name: string;
  icon: string;
  is_left: boolean;
  is_right: boolean;
}

export interface DefenseTypeOption {
  id: string;
  name: string;
  icon: string;
}

export interface ConditionTypeOption {
  id: string;
  name: string;
  params: string[];
}
