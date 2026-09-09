/**
 * State Validation Utilities
 * 
 * Validates state data to ensure type safety and data integrity.
 * These functions should be used when loading persisted state or
 * external data to ensure it matches expected formats.
 */

import type {
  BoxerRecord,
  Color,
  ProjectFile,
  ProjectMetadata,
  ExternalTool,
  UpdateSettings,
  EmulatorSettings,
} from './types';

// ============================================================================
// Validation Error Types
// ============================================================================

export class ValidationError extends Error {
  constructor(message: string, public path: string) {
    super(message);
    this.name = 'ValidationError';
  }
}

export interface ValidationResult<T> {
  valid: boolean;
  data?: T;
  errors: ValidationError[];
}

// ============================================================================
// Helper Functions
// ============================================================================

function assertString(value: unknown, path: string): string {
  if (typeof value !== 'string') {
    throw new ValidationError(`Expected string, got ${typeof value}`, path);
  }
  return value;
}

function assertNumber(value: unknown, path: string): number {
  if (typeof value !== 'number' || isNaN(value)) {
    throw new ValidationError(`Expected number, got ${typeof value}`, path);
  }
  return value;
}

function assertBoolean(value: unknown, path: string): boolean {
  if (typeof value !== 'boolean') {
    throw new ValidationError(`Expected boolean, got ${typeof value}`, path);
  }
  return value;
}

function assertArray<T>(
  value: unknown,
  path: string,
  itemValidator: (item: unknown, path: string) => T
): T[] {
  if (!Array.isArray(value)) {
    throw new ValidationError(`Expected array, got ${typeof value}`, path);
  }
  return value.map((item, index) => itemValidator(item, `${path}[${index}]`));
}

function assertObject(value: unknown, path: string): Record<string, unknown> {
  if (typeof value !== 'object' || value === null || Array.isArray(value)) {
    throw new ValidationError(`Expected object, got ${typeof value}`, path);
  }
  return value as Record<string, unknown>;
}

function assertOptional<T>(
  value: unknown,
  path: string,
  validator: (value: unknown, path: string) => T
): T | null {
  if (value === null || value === undefined) {
    return null;
  }
  return validator(value, path);
}

// ============================================================================
// Validators
// ============================================================================

export function validateColor(value: unknown, path = 'color'): Color {
  const obj = assertObject(value, path);
  return {
    r: assertNumber(obj.r, `${path}.r`),
    g: assertNumber(obj.g, `${path}.g`),
    b: assertNumber(obj.b, `${path}.b`),
  };
}

export function validateBoxerRecord(value: unknown, path = 'boxer'): BoxerRecord {
  const obj = assertObject(value, path);
  
  // Validate required fields
  const fighter = assertString(obj.fighter, `${path}.fighter`);
  const key = assertString(obj.key, `${path}.key`);
  
  if (!fighter) {
    throw new ValidationError('Boxer fighter name is required', `${path}.fighter`);
  }
  if (!key) {
    throw new ValidationError('Boxer key is required', `${path}.key`);
  }
  
  return {
    fighter,
    key,
    reference_sheet: assertString(obj.reference_sheet || '', `${path}.reference_sheet`),
    palette_files: assertArray(obj.palette_files || [], `${path}.palette_files`, validateAssetFile),
    icon_files: assertArray(obj.icon_files || [], `${path}.icon_files`, validateAssetFile),
    portrait_files: assertArray(obj.portrait_files || [], `${path}.portrait_files`, validateAssetFile),
    large_portrait_files: assertArray(obj.large_portrait_files || [], `${path}.large_portrait_files`, validateAssetFile),
    unique_sprite_bins: assertArray(obj.unique_sprite_bins || [], `${path}.unique_sprite_bins`, validateAssetFile),
    shared_sprite_bins: assertArray(obj.shared_sprite_bins || [], `${path}.shared_sprite_bins`, validateAssetFile),
    other_files: assertArray(obj.other_files || [], `${path}.other_files`, validateAssetFile),
  };
}

function validateAssetFile(value: unknown, path: string) {
  const obj = assertObject(value, path);
  return {
    file: assertString(obj.file || '', `${path}.file`),
    filename: assertString(obj.filename || '', `${path}.filename`),
    category: assertString(obj.category || '', `${path}.category`),
    subtype: assertString(obj.subtype || '', `${path}.subtype`),
    size: assertNumber(obj.size || 0, `${path}.size`),
    start_snes: assertString(obj.start_snes || '', `${path}.start_snes`),
    end_snes: assertString(obj.end_snes || '', `${path}.end_snes`),
    start_pc: assertString(obj.start_pc || '', `${path}.start_pc`),
    end_pc: assertString(obj.end_pc || '', `${path}.end_pc`),
    shared_with: assertArray(obj.shared_with || [], `${path}.shared_with`, (v, p) => assertString(v, p)),
  };
}

export function validateProjectMetadata(value: unknown, path = 'metadata'): ProjectMetadata {
  const obj = assertObject(value, path);
  
  const name = assertString(obj.name, `${path}.name`);
  if (!name) {
    throw new ValidationError('Project name is required', `${path}.name`);
  }
  
  return {
    name,
    author: assertOptional(obj.author, `${path}.author`, assertString),
    description: assertOptional(obj.description, `${path}.description`, assertString),
    created_at: assertString(obj.created_at || new Date().toISOString(), `${path}.created_at`),
    modified_at: assertString(obj.modified_at || new Date().toISOString(), `${path}.modified_at`),
    version: assertString(obj.version || '0.1.0', `${path}.version`),
  };
}

export function validateProjectFile(value: unknown, path = 'project'): ValidationResult<ProjectFile> {
  const errors: ValidationError[] = [];
  
  try {
    const obj = assertObject(value, path);
    
    const project: ProjectFile = {
      version: assertNumber(obj.version || 1, `${path}.version`),
      rom_base_sha1: assertString(obj.rom_base_sha1 || '', `${path}.rom_base_sha1`),
      manifest_version: assertString(obj.manifest_version || '1.0', `${path}.manifest_version`),
      metadata: validateProjectMetadata(obj.metadata, `${path}.metadata`),
      edits: [], // Simplified - would validate full structure
      assets: [], // Simplified - would validate full structure
      settings: assertObject(obj.settings || {}, `${path}.settings`),
    };
    
    // Validate SHA1 format if present
    if (project.rom_base_sha1 && !/^[a-f0-9]{40}$/i.test(project.rom_base_sha1)) {
      errors.push(new ValidationError('Invalid SHA1 format', `${path}.rom_base_sha1`));
    }
    
    return {
      valid: errors.length === 0,
      data: project,
      errors,
    };
  } catch (e) {
    if (e instanceof ValidationError) {
      errors.push(e);
    } else {
      errors.push(new ValidationError(String(e), path));
    }
    return { valid: false, errors };
  }
}

export function validateExternalTool(value: unknown, path = 'tool'): ExternalTool {
  const obj = assertObject(value, path);
  
  const id = assertString(obj.id, `${path}.id`);
  const name = assertString(obj.name, `${path}.name`);
  const executable_path = assertString(obj.executable_path, `${path}.executable_path`);
  
  if (!id) {
    throw new ValidationError('Tool ID is required', `${path}.id`);
  }
  if (!name) {
    throw new ValidationError('Tool name is required', `${path}.name`);
  }
  
  const category = assertString(obj.category || 'other', `${path}.category`);
  const validCategories = ['graphics_editor', 'hex_editor', 'tile_editor', 'emulator', 'other'];
  if (!validCategories.includes(category)) {
    throw new ValidationError(
      `Invalid category: ${category}. Must be one of: ${validCategories.join(', ')}`,
      `${path}.category`
    );
  }
  
  return {
    id,
    name,
    executable_path,
    arguments_template: assertString(obj.arguments_template || '{file}', `${path}.arguments_template`),
    supported_file_types: assertArray(
      obj.supported_file_types || [],
      `${path}.supported_file_types`,
      (v, p) => assertString(v, p)
    ),
    category: category as ExternalTool['category'],
    enabled: assertBoolean(obj.enabled !== undefined ? obj.enabled : true, `${path}.enabled`),
    working_directory: assertOptional(obj.working_directory, `${path}.working_directory`, assertString) || undefined,
    env_vars: assertObject(obj.env_vars || {}, `${path}.env_vars`) as Record<string, string>,
  };
}

export function validateUpdateSettings(value: unknown, path = 'updateSettings'): UpdateSettings {
  const obj = assertObject(value, path);
  
  const check_interval = assertString(obj.check_interval || 'weekly', `${path}.check_interval`);
  const validIntervals = ['daily', 'weekly', 'monthly', 'never'];
  if (!validIntervals.includes(check_interval)) {
    throw new ValidationError(
      `Invalid check_interval: ${check_interval}`,
      `${path}.check_interval`
    );
  }
  
  const channel = assertString(obj.channel || 'stable', `${path}.channel`);
  const validChannels = ['stable', 'beta'];
  if (!validChannels.includes(channel)) {
    throw new ValidationError(
      `Invalid channel: ${channel}`,
      `${path}.channel`
    );
  }
  
  return {
    check_on_startup: assertBoolean(obj.check_on_startup !== undefined ? obj.check_on_startup : true, `${path}.check_on_startup`),
    check_interval: check_interval as UpdateSettings['check_interval'],
    channel: channel as UpdateSettings['channel'],
    skipped_versions: assertArray(
      obj.skipped_versions || [],
      `${path}.skipped_versions`,
      (v, p) => assertString(v, p)
    ),
    last_check: assertOptional(obj.last_check, `${path}.last_check`, assertString),
  };
}

export function validateEmulatorSettings(value: unknown, path = 'emulatorSettings'): EmulatorSettings {
  const obj = assertObject(value, path);
  
  const emulatorType = assertString(obj.emulatorType || 'snes9x', `${path}.emulatorType`);
  const validTypes = ['snes9x', 'bsnes', 'mesen-s', 'other'];
  if (!validTypes.includes(emulatorType)) {
    throw new ValidationError(
      `Invalid emulatorType: ${emulatorType}`,
      `${path}.emulatorType`
    );
  }
  
  return {
    emulatorPath: assertString(obj.emulatorPath || '', `${path}.emulatorPath`),
    emulatorType: emulatorType as EmulatorSettings['emulatorType'],
    autoSaveBeforeLaunch: assertBoolean(
      obj.autoSaveBeforeLaunch !== undefined ? obj.autoSaveBeforeLaunch : true,
      `${path}.autoSaveBeforeLaunch`
    ),
    commandLineArgs: assertString(obj.commandLineArgs || '', `${path}.commandLineArgs`),
    jumpToSelectedBoxer: assertBoolean(
      obj.jumpToSelectedBoxer !== undefined ? obj.jumpToSelectedBoxer : true,
      `${path}.jumpToSelectedBoxer`
    ),
    defaultRound: Math.max(1, Math.min(4, assertNumber(obj.defaultRound || 1, `${path}.defaultRound`))),
    saveStateDir: assertOptional(obj.saveStateDir, `${path}.saveStateDir`, assertString),
  };
}

// ============================================================================
// Safe Validation Helpers
// ============================================================================

export function safeValidate<T>(
  validator: (value: unknown) => T,
  value: unknown,
  defaultValue: T
): T {
  try {
    return validator(value);
  } catch (e) {
    console.warn('Validation failed, using default:', e);
    return defaultValue;
  }
}

export function safeValidateProject(value: unknown): ProjectFile | null {
  const result = validateProjectFile(value);
  return result.valid ? result.data! : null;
}
