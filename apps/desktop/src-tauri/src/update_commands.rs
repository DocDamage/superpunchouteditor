//! Auto-updater module for Super Punch-Out!! Editor
//!
//! Handles checking for updates from GitHub releases, downloading updates,
//! and managing update preferences.

use serde::{Deserialize, Serialize};
use parking_lot::Mutex;
use tauri::{AppHandle, State};
use tauri_plugin_updater::UpdaterExt;

/// Current application version from Cargo.toml
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// GitHub repository for releases
const GITHUB_OWNER: &str = "dferr";
const GITHUB_REPO: &str = "super-punch-out-editor";

/// Update settings stored in app data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSettings {
    /// Whether to check for updates on startup
    pub check_on_startup: bool,
    /// How often to check for updates
    pub check_interval: UpdateInterval,
    /// Release channel (stable or beta)
    pub channel: ReleaseChannel,
    /// Versions the user has chosen to skip
    pub skipped_versions: Vec<String>,
    /// Last time updates were checked (ISO 8601)
    pub last_check: Option<String>,
}

impl Default for UpdateSettings {
    fn default() -> Self {
        Self {
            check_on_startup: true,
            check_interval: UpdateInterval::Weekly,
            channel: ReleaseChannel::Stable,
            skipped_versions: Vec::new(),
            last_check: None,
        }
    }
}

/// Update check interval options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UpdateInterval {
    Daily,
    Weekly,
    Monthly,
    Never,
}

/// Release channel
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReleaseChannel {
    Stable,
    Beta,
}

/// Information about an available update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    /// Version string (e.g., "v2.1.0")
    pub version: String,
    /// Release notes / changelog
    pub notes: String,
    /// Publish date (ISO 8601)
    pub pub_date: Option<String>,
    /// Download URL
    pub download_url: Option<String>,
    /// Whether this is a mandatory update
    pub mandatory: bool,
    /// Whether the current version is the latest
    pub is_latest: bool,
}

/// Update download progress
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DownloadProgress {
    /// Progress percentage (0-100)
    #[serde(default)]
    pub percent: u8,
    /// Bytes downloaded
    #[serde(default)]
    pub downloaded: u64,
    /// Total bytes to download
    #[serde(default)]
    pub total: u64,
    /// Current download state
    #[serde(default)]
    pub state: DownloadState,
}

/// Download state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DownloadState {
    #[default]
    Idle,
    Checking,
    Downloading,
    Verifying,
    Ready,
    Installing,
    Error,
}

/// State for update operations
#[derive(Default)]
pub struct UpdateState {
    pub settings: Mutex<UpdateSettings>,
    pub current_update: Mutex<Option<tauri_plugin_updater::Update>>,
    pub download_progress: Mutex<DownloadProgress>,
}


/// Get the current application version
#[tauri::command]
pub fn get_current_version() -> String {
    CURRENT_VERSION.to_string()
}

/// Get update settings
#[tauri::command]
pub fn get_update_settings(state: State<UpdateState>) -> UpdateSettings {
    state.settings.lock().clone()
}

/// Update update settings
#[tauri::command]
pub fn set_update_settings(
    state: State<UpdateState>,
    settings: UpdateSettings,
) -> Result<(), String> {
    *state.settings.lock() = settings;
    Ok(())
}

/// Check if an update is available
///
/// Returns UpdateInfo if an update is available and should be shown to the user
/// (i.e., not skipped and newer than current version).
#[tauri::command]
pub async fn check_for_updates(
    app: AppHandle,
    state: State<'_, UpdateState>,
) -> Result<Option<UpdateInfo>, String> {
    let settings = state.settings.lock().clone();

    // Update last check time
    let mut new_settings = settings.clone();
    new_settings.last_check = Some(chrono::Utc::now().to_rfc3339());
    *state.settings.lock() = new_settings;

    // Use tauri-plugin-updater to check for updates
    let updater = app.updater().map_err(|e| e.to_string())?;

    match updater.check().await {
        Ok(Some(update)) => {
            let version = update.version.clone();

            // Check if this version should be skipped
            if settings.skipped_versions.contains(&version) {
                return Ok(None);
            }

            // Compare versions using semver
            let current = semver::Version::parse(CURRENT_VERSION)
                .map_err(|e| format!("Failed to parse current version: {}", e))?;
            let latest = semver::Version::parse(&version.trim_start_matches('v'))
                .map_err(|e| format!("Failed to parse latest version: {}", e))?;

            if latest <= current {
                return Ok(Some(UpdateInfo {
                    version,
                    notes: update.body.clone().unwrap_or_default(),
                    pub_date: update.date.map(|d| d.to_string()),
                    download_url: None,
                    mandatory: false,
                    is_latest: true,
                }));
            }

            // Store the update for later download
            *state.current_update.lock() = Some(update.clone());

            Ok(Some(UpdateInfo {
                version: version.clone(),
                notes: update.body.clone().unwrap_or_default(),
                pub_date: update.date.map(|d| d.to_string()),
                download_url: Some(update.download_url.to_string()),
                mandatory: false,
                is_latest: false,
            }))
        }
        Ok(None) => {
            // No update available
            Ok(Some(UpdateInfo {
                version: CURRENT_VERSION.to_string(),
                notes: "You are running the latest version.".to_string(),
                pub_date: None,
                download_url: None,
                mandatory: false,
                is_latest: true,
            }))
        }
        Err(e) => Err(format!("Failed to check for updates: {}", e)),
    }
}

/// Skip a specific version
#[tauri::command]
pub fn skip_version(state: State<UpdateState>, version: String) -> Result<(), String> {
    let mut settings = state.settings.lock();
    if !settings.skipped_versions.contains(&version) {
        settings.skipped_versions.push(version);
    }
    Ok(())
}

/// Download and install the update
#[tauri::command]
pub async fn download_and_install_update(
    app: AppHandle,
    state: State<'_, UpdateState>,
) -> Result<(), String> {
    let update = {
        let guard = state.current_update.lock();
        guard.clone().ok_or("No update available to download")?
    };

    // Update download state
    {
        let mut progress = state.download_progress.lock();
        progress.state = DownloadState::Downloading;
    }

    // Download and install the update
    match update
        .download_and_install(
            |chunk_length, content_length| {
                // Progress callback
                let downloaded = chunk_length as u64;
                let total = content_length.unwrap_or(0);
                let percent = if total > 0 {
                    ((downloaded as f64 / total as f64) * 100.0) as u8
                } else {
                    0
                };

                // Note: In a real implementation, you might want to emit events
                // to the frontend for real-time progress updates
                let _ = (percent, downloaded, total);
            },
            || {
                // Finished callback
            },
        )
        .await
    {
        Ok(_) => {
            // Clear the current update and reset progress
            *state.current_update.lock() = None;
            *state.download_progress.lock() = DownloadProgress {
                percent: 100,
                downloaded: 0,
                total: 0,
                state: DownloadState::Ready,
            };

            // Restart the app - this never returns
            app.restart();
            #[allow(unreachable_code)]
            Ok(())
        }
        Err(e) => {
            state.download_progress.lock().state = DownloadState::Error;
            Err(format!("Failed to install update: {}", e))
        }
    }
}

/// Get the current download progress
#[tauri::command]
pub fn get_download_progress(state: State<UpdateState>) -> DownloadProgress {
    state.download_progress.lock().clone()
}

/// Get the manual download URL for the latest release
#[tauri::command]
pub fn get_manual_download_url() -> String {
    format!(
        "https://github.com/{}/{}/releases/latest",
        GITHUB_OWNER, GITHUB_REPO
    )
}

/// Clear the list of skipped versions
#[tauri::command]
pub fn clear_skipped_versions(state: State<UpdateState>) -> Result<(), String> {
    state.settings.lock().skipped_versions.clear();
    Ok(())
}

/// Check if it's time to auto-check for updates based on settings
#[tauri::command]
pub fn should_auto_check(state: State<UpdateState>) -> bool {
    let settings = state.settings.lock();

    if !settings.check_on_startup {
        return false;
    }

    if let Some(last_check) = &settings.last_check {
        let last = chrono::DateTime::parse_from_rfc3339(last_check)
            .unwrap_or_else(|_| chrono::DateTime::UNIX_EPOCH.into());
        let now = chrono::Utc::now();
        let duration = now.signed_duration_since(last);

        match settings.check_interval {
            UpdateInterval::Daily => duration.num_hours() >= 24,
            UpdateInterval::Weekly => duration.num_days() >= 7,
            UpdateInterval::Monthly => duration.num_days() >= 30,
            UpdateInterval::Never => false,
        }
    } else {
        // Never checked before
        true
    }
}
