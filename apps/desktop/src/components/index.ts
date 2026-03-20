// Main components
export { AnimationEditor } from './AnimationEditor';
export { AnimationPlayer } from './AnimationPlayer';
export type { 
  AnimationPlayerProps,
  Hitbox,
  Hurtbox,
  HitboxType,
  InterpolationType,
  AnimationType,
  AnimationFrame,
  BoxerAnimation,
  Boxer,
  PlaybackState,
  FrameEffect,
} from './AnimationPlayer';
export { 
  PluginManager,
  type Plugin,
  type PluginCommand,
  type PluginCommandArg,
  type BatchJob,
  type ScriptOutput,
} from './PluginManager';
export { AnimationTimeline } from './AnimationTimeline';
export { AnimationPreview } from './AnimationPreview';
export type { 
  Animation, 
  AnimationFrame, 
  FrameEffect, 
  AnimationCategory,
  FighterAnimations,
  AnimationCategoryInfo 
} from './AnimationEditor';
export { AssetManager } from './AssetManager';
export { BoxerPreviewSheet } from './BoxerPreviewSheet';
export { ExportPanel } from './ExportPanel';
export { FighterViewer } from './FighterViewer';
export { PaletteEditor } from './PaletteEditor';
export { ScriptViewer } from './ScriptViewer';
export { SpriteBinEditor } from './SpriteBinEditor';

// Boxer Comparison components
export { BoxerCompare } from './BoxerCompare';
export { StatComparisonTable } from './StatComparisonTable';
export { SimilarBoxers } from './SimilarBoxers';
export type { ComparisonData, SimilarBoxerData } from './BoxerCompare';

// Shared Bank Warning Engine components
export { 
  SharedBankWarning, 
  type SharedBankInfo,
  type SharedBankPair 
} from './SharedBankWarning';
export { 
  SharedBankIndicator, 
  SharedBankSummary,
  type SharedBankIndicatorProps,
  type SharedBankSummaryProps 
} from './SharedBankIndicator';

// Emulator Integration components
export { 
  EmulatorSettings,
  type EmulatorSettingsData,
  type EmulatorInfo 
} from './EmulatorSettings';
export { TestInEmulatorButton } from './TestInEmulatorButton';

// Comparison Mode components
export { ComparisonView } from './ComparisonView';
export { ComparisonCanvas } from './ComparisonCanvas';
export { ComparisonTable } from './ComparisonTable';
export { DiffReport } from './DiffReport';

// Frame Tagging components
export { FrameTagger } from './FrameTagger';
export { AnnotationPanel } from './AnnotationPanel';

// Project Thumbnail components
export { 
  ProjectThumbnailDisplay, 
  ThumbnailCaptureButton,
  ThumbnailManager,
  type ProjectThumbnailDisplayProps,
  type ThumbnailCaptureButtonProps,
  type ThumbnailManagerProps 
} from './ProjectThumbnail';

// External Tools components
export { 
  ExternalToolsManager,
  type ExternalTool,
  type ToolContext,
  type ToolCategory 
} from './ExternalToolsManager';
export { 
  OpenWithMenu,
  useToolLauncher,
  type OpenWithMenuProps 
} from './OpenWithMenu';

// Roster Metadata Editor components
export { RosterEditor } from './RosterEditor';
export { BoxerNameEditor } from './BoxerNameEditor';
export { CircuitEditor } from './CircuitEditor';
export type { 
  BoxerRosterEntry, 
  Circuit, 
  CircuitType,
  RosterData,
  ValidationReport 
} from '../types/roster';

// Text/Dialog Editor components
export { TextEditor } from './TextEditor';
export { TextPreview } from './TextPreview';

// Keyboard Shortcuts components
export { KeyboardShortcutsCheatSheet } from './KeyboardShortcutsCheatSheet';
export { 
  ShortcutDisplay, 
  ShortcutGroup, 
  ShortcutHint 
} from './ShortcutDisplay';

// Settings Import/Export components
export { 
  SettingsManager,
  type SettingsManagerProps 
} from './SettingsManager';
export { 
  SettingsImportDialog,
  type SettingsImportDialogProps 
} from './SettingsImportDialog';

// Settings configuration
export {
  DEFAULT_SETTINGS,
  SETTINGS_CATEGORIES,
  SETTINGS_DISPLAY_NAMES,
  validateSettingsImport,
  createSettingsExport,
  generateImportPreview,
  mergeSettings,
  exportSettingsToJson,
  parseSettingsFromJson,
  type AppSettings,
  type SettingsExport,
  type ImportReport,
  type SettingsValidation,
  type SettingsChangePreview,
  type PanelLayout,
} from '../config/settings';

// Theme Toggle component
export { ThemeToggle, ThemeToggleWithLabel } from './ThemeToggle';
export type { ThemeToggleProps } from './ThemeToggle';

// Auto-Updater components
export { UpdateChecker } from './UpdateChecker';
export { UpdateAvailableModal } from './UpdateAvailableModal';
export { UpdateProgress } from './UpdateProgress';
export { UpdateSettings } from './UpdateSettings';
export type { UpdateInfo, DownloadProgress, UpdateSettings as UpdateSettingsData } from '../store/useStore';

// Embedded Emulator components
export { EmbeddedEmulator } from './EmbeddedEmulator';
export { EmulatorCanvas } from './EmulatorCanvas';
export { EmulatorControls } from './EmulatorControls';
export { InputMapper } from './InputMapper';
export type { 
  EmbeddedEmulatorProps, 
  EmulatorLayout, 
  RomSource 
} from './EmbeddedEmulator';
export type { 
  EmulatorCanvasProps 
} from './EmulatorCanvas';
export type { 
  EmulatorControlsProps 
} from './EmulatorControls';
export type { 
  InputMapperProps 
} from './InputMapper';
