# Settings Import/Export Feature

This document describes the Settings Import/Export feature for the Super Punch-Out!! Editor.

## Overview

The Settings Import/Export feature allows users to:

1. **Export** their complete editor configuration to a JSON file
2. **Import** settings from a previously exported file
3. **Merge** imported settings with existing settings (optional)
4. **Reset** all settings to defaults

This is useful for:
- Backing up your configuration
- Sharing configurations with other users
- Migrating settings between machines
- Quickly setting up a new installation

## Settings Structure

Settings are stored in the following location:
- **Windows**: `%APPDATA%/super-punch-out-editor/app-settings.json`
- **macOS**: `~/Library/Application Support/super-punch-out-editor/app-settings.json`
- **Linux**: `~/.config/super-punch-out-editor/app-settings.json`

### Settings Categories

#### Appearance
- `theme`: UI theme ('dark', 'light', 'system')
- `uiScale`: UI scale factor (0.75, 1.0, 1.25, 1.5)
- `sidebarCollapsed`: Whether sidebar is collapsed

#### Editor
- `defaultExportFormat`: Default export format ('png', 'bmp')
- `autoSaveInterval`: Auto-save interval in minutes (0 = disabled)
- `confirmOnClose`: Show confirmation on close with unsaved changes
- `showTooltips`: Show tooltips throughout the UI

#### Emulator
- `emulatorPath`: Path to emulator executable
- `emulatorType`: Type of emulator ('snes9x', 'bsnes', 'mesen-s', 'other')
- `autoSaveBeforeLaunch`: Auto-save ROM before launching emulator

#### Paths
- `defaultExportDirectory`: Default directory for exports
- `recentProjects`: List of recently opened projects
- `recentRoms`: List of recently opened ROM files

#### External Tools
- `externalTools`: Array of configured external tools
- `defaultToolIds`: Map of file extensions to default tool IDs

#### Keyboard Shortcuts
- `customShortcuts`: Map of action IDs to keyboard shortcuts

#### Layout
- `panelLayout`: Panel layout preference ('default', 'compact', 'wide', 'custom')

## Export File Format

Exported settings files have the following structure:

```json
{
  "version": "2.0",
  "exported_at": "2026-03-19T12:00:00Z",
  "app": "Super Punch-Out!! Editor",
  "settings": {
    "theme": "dark",
    "ui_scale": 1.0,
    "emulator_path": "/path/to/snes9x",
    "external_tools": [...],
    ...
  }
}
```

## Import Modes

### Merge Mode (Default)
- Imported settings are merged with existing settings
- Existing settings are preserved where not specified in the import
- Arrays (like recent projects) are combined and deduplicated

### Replace Mode
- All settings are replaced with imported values
- Existing settings not in the import are reset to defaults
- Use with caution - this will erase custom settings!

## Version Compatibility

Settings files include a version number. The editor will:
- Import settings from the same major version without issues
- Show a warning for different major versions
- Attempt to import compatible settings where possible

## Frontend Components

### SettingsManager
Main dialog for settings management. Provides:
- Export functionality
- Import with validation
- Merge/replace mode selection
- Reset to defaults with confirmation

### SettingsImportDialog
Advanced import dialog with:
- Detailed preview of changes
- Category-based organization
- Conflict highlighting
- Selective import (choose which settings to import)

## Rust Commands

### `export_settings(output_path: String) -> Result<(), String>`
Exports all current settings to the specified file path.

### `import_settings(settings_path: String, merge: bool) -> Result<ImportReport, String>`
Imports settings from a file. Returns a report of what was imported/merged/skipped.

### `preview_settings_import(settings_path: String) -> Result<Vec<SettingsChangePreview>, String>`
Returns a preview of what settings will change during import.

### `validate_settings_file(settings_path: String) -> Result<Value, String>`
Validates a settings file without importing it.

### `reset_settings_to_defaults() -> Result<(), String>`
Resets all settings to their default values.

### `get_app_settings() -> Result<AppSettings, String>`
Returns the current app settings.

### `save_settings(settings: AppSettings) -> Result<(), String>`
Saves the provided settings.

### `update_settings(updates: Value) -> Result<AppSettings, String>`
Updates specific settings fields.

## TypeScript Types

### AppSettings
Main settings interface with all configuration options.

### SettingsExport
Export file structure including metadata.

### ImportReport
Result of an import operation showing what changed.

### SettingsChangePreview
Preview of a single setting change for the import dialog.

### SettingsValidation
Result of validating a settings file.

## Usage Example

```typescript
import { SettingsManager } from './components';

function App() {
  const [showSettings, setShowSettings] = useState(false);

  return (
    <>
      <button onClick={() => setShowSettings(true)}>
        Open Settings
      </button>
      
      <SettingsManager
        isOpen={showSettings}
        onClose={() => setShowSettings(false)}
      />
    </>
  );
}
```

## Security Considerations

- Settings files are JSON and can be manually edited
- Path validation is performed on import (warnings for non-existent paths)
- External tool paths are checked for accessibility
- No executable code is stored in settings files

## Future Enhancements

Potential improvements for future versions:

1. **Cloud Sync**: Sync settings across devices via cloud storage
2. **Profiles**: Multiple named configuration profiles
3. **Selective Export**: Choose which categories to export
4. **Import Presets**: Shareable preset configurations
5. **Settings Diff**: Compare two settings files
6. **Rollback**: Undo last import operation
