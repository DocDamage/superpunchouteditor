# Auto-Updater Mechanism

The Super Punch-Out!! Editor includes an auto-updater that checks for updates from GitHub releases and allows users to download and install new versions automatically.

## Components

### UpdateChecker
Wraps the application and handles automatic update checking on startup based on user preferences.

```tsx
<UpdateChecker>
  <App />
</UpdateChecker>
```

### UpdateAvailableModal
Displays when a new version is available, showing:
- Version information
- Release notes/changelog
- Options to download, skip, or remind later

### UpdateProgress
Shows download and installation progress with:
- Progress bar
- Download percentage
- Status messages
- Cancel option

### UpdateSettings
Settings panel for configuring:
- Check on startup toggle
- Check interval (daily, weekly, monthly, never)
- Release channel (stable, beta)
- Skipped versions management
- Manual download link

## Rust Backend

The `update_commands.rs` module provides:

### Commands
- `get_current_version` - Returns current app version
- `get_update_settings` / `set_update_settings` - Manage update preferences
- `check_for_updates` - Check GitHub releases for new versions
- `skip_version` - Add version to skip list
- `download_and_install_update` - Download and install update
- `get_download_progress` - Get current download status
- `get_manual_download_url` - Get GitHub releases URL
- `clear_skipped_versions` - Reset skip list
- `should_auto_check` - Check if auto-update should run

### Types
```rust
UpdateSettings {
    check_on_startup: bool,
    check_interval: UpdateInterval,
    channel: ReleaseChannel,
    skipped_versions: Vec<String>,
    last_check: Option<String>,
}

UpdateInfo {
    version: String,
    notes: String,
    pub_date: Option<String>,
    download_url: Option<String>,
    mandatory: bool,
    is_latest: bool,
}
```

## Configuration

### tauri.conf.json
```json
{
  "plugins": {
    "updater": {
      "pubkey": "<base64-encoded-public-key>",
      "endpoints": [
        "https://api.github.com/repos/OWNER/REPO/releases/latest"
      ]
    }
  }
}
```

**Note:** The public key in the configuration is a placeholder. For production use:
1. Generate a proper keypair using `tauri signer generate`
2. Update the pubkey field with your public key
3. Sign your releases with the private key

## Store Integration

The update system integrates with the Zustand store:

```typescript
interface AppStore {
  // Update state
  updateSettings: UpdateSettings;
  currentVersion: string;
  availableUpdate: UpdateInfo | null;
  downloadProgress: DownloadProgress;
  checkingForUpdate: boolean;
  updateError: string | null;
  
  // Actions
  loadUpdateSettings: () => Promise<void>;
  saveUpdateSettings: (settings: UpdateSettings) => Promise<void>;
  checkForUpdates: () => Promise<UpdateInfo | null>;
  skipVersion: (version: string) => Promise<void>;
  downloadAndInstallUpdate: () => Promise<void>;
}
```

## Usage Flow

1. **App Startup**: UpdateChecker loads settings and optionally checks for updates
2. **Update Available**: UpdateAvailableModal is displayed
3. **User Actions**:
   - Download & Install → Shows UpdateProgress → App restarts
   - Skip This Version → Added to skipped list
   - Remind Me Later → Modal closes, will prompt again
4. **Settings**: User can configure check interval and view skipped versions

## Manual Update

If automatic updates fail, users can:
1. Go to Settings tab → Auto-Updater section
2. Click "Download from GitHub"
3. Install the new version manually

## GitHub Releases

The updater expects GitHub releases with:
- Tag name matching version (e.g., "v2.1.0")
- Release assets for each platform
- Optional release notes in the body

## Dependencies

### Rust
- `tauri-plugin-updater = "2"`
- `semver = "1.0"`

### Frontend
- `@tauri-apps/plugin-updater = "^2"`
