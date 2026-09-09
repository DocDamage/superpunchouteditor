/**
 * Custom Hooks
 * 
 * Export all custom hooks for the application.
 */

// Emulator hooks
export {
  useEmulator,
  type UseEmulatorOptions,
  type UseEmulatorReturn,
  type EmulatorState,
  type ControllerType,
  type ScalingMode,
  type SpeedMode,
  type InputMapping,
  type EmulatorConfig,
  type SaveState,
  type EmulatorStatus,
  type KeyMappingPreset,
  SNES_BUTTONS,
  DEFAULT_KEY_MAPPINGS_WASD,
  DEFAULT_KEY_MAPPINGS_ARROWS,
  DEFAULT_KEY_MAPPINGS_FIGHTSTICK,
} from './useEmulator';

// Plugin hooks
export {
  usePlugins,
  useLoadPlugin,
  useUnloadPlugin,
  useEnablePlugin,
  useDisablePlugin,
  useExecutePluginCommand,
  useRunScript,
  useRunScriptFile,
  useBatchJobs,
  type UsePluginsState,
  type UsePluginsReturn,
} from './usePlugins';

// Bank management hooks
export {
  useBankManagement,
  useBankMap,
  useBankStatistics,
  useFragmentationAnalysis,
  useDefragmentationPlan,
  useDefragmentation,
  useFreeRegions,
  type UseBankMapReturn,
  type UseBankStatisticsReturn,
  type UseFragmentationAnalysisReturn,
  type UseDefragmentationPlanReturn,
  type UseDefragmentationReturn,
  type UseFreeRegionsReturn,
} from './useBankManagement';

// Animation hooks
export {
  useAnimation,
  useAnimationPlayer,
  useBoxerAnimation,
  usePlayAnimation,
  usePauseAnimation,
  useStopAnimation,
  useSeekAnimation,
  useUpdateAnimation,
  useHitboxes,
  useUpdateHitbox,
  useAddHitbox,
  useRemoveHitbox,
  type UseBoxerAnimationReturn,
  type UsePlayAnimationReturn,
  type UsePauseAnimationReturn,
  type UseStopAnimationReturn,
  type UseSeekAnimationReturn,
  type UseUpdateAnimationReturn,
  type UseAnimationPlayerReturn,
  type UseHitboxesReturn,
  type UseUpdateHitboxReturn,
  type UseAddHitboxReturn,
  type UseRemoveHitboxReturn,
  type UseAnimationReturn,
} from './useAnimation';
