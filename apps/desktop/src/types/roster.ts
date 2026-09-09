/**
 * Type definitions for Roster Metadata Editor
 */

export type CircuitType = 'Minor' | 'Major' | 'World' | 'Special';

export interface BoxerRosterEntry {
  boxer_id?: number;
  fighter_id?: number;
  name: string;
  name_raw: number[];
  circuit: CircuitType;
  unlock_order: number;
  intro_text_id: number;
  is_unlockable: boolean;
  is_champion: boolean;
  name_offset?: number;
}

export interface Circuit {
  circuit_type: CircuitType;
  name: string;
  boxers: number[];
  required_wins: number;
}

export interface RosterData {
  boxers: BoxerRosterEntry[];
  circuits: Circuit[];
}

export interface IntroText {
  text_id: number;
  text: string;
  boxer_id?: number;
  fighter_id?: number;
}

export interface TextEncodingInfo {
  supported_chars: string[];
  max_name_length: number;
  max_intro_length: number;
}

export interface NameValidationResult {
  valid: boolean;
  encoded_length: number;
  max_length: number;
  can_encode: boolean;
  error: string | null;
}

export interface ValidationIssue {
  DuplicateName?: {
    name: string;
    boxer_ids?: number[];
    fighter_ids?: number[];
  };
  GapInUnlockOrder?: {
    from: number;
    to: number;
  };
  MissingChampionFlag?: {
    boxer_id?: number;
    fighter_id?: number;
    circuit: CircuitType;
  };
  BoxerNotInAnyCircuit?: {
    boxer_id?: number;
    fighter_id?: number;
    name: string;
  };
}

export interface ValidationReport {
  is_valid: boolean;
  errors: ValidationIssue[];
  warnings: ValidationIssue[];
}

export interface CircuitTypeInfo {
  value: number;
  label: string;
  name: string;
}
