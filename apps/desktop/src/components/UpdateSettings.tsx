import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { UpdateSettings as UpdateSettingsType, UpdateInfo } from '../store/useStore';
import { useStore } from '../store/useStore';
import { UpdateAvailableModal } from './UpdateAvailableModal';
import { showToast } from './ToastContainer';
import { UpdateProgress } from './UpdateProgress';
import { openUrl } from '@tauri-apps/plugin-opener';

const UPDATE_INTERVALS = [
  { value: 'daily', label: 'Daily' },
  { value: 'weekly', label: 'Weekly' },
  { value: 'monthly', label: 'Monthly' },
  { value: 'never', label: 'Never' },
] as const;

const RELEASE_CHANNELS = [
  { value: 'stable', label: 'Stable' },
  { value: 'beta', label: 'Beta' },
] as const;

export function UpdateSettings() {
  const {
    updateSettings,
    currentVersion,
    availableUpdate,
    downloadProgress,
    checkingForUpdate,
    updateError,
    loadUpdateSettings,
    saveUpdateSettings,
    checkForUpdates,
    skipVersion,
    downloadAndInstallUpdate,
    clearSkippedVersions,
  } = useStore();

  const [showUpdateModal, setShowUpdateModal] = useState(false);
  const [showProgress, setShowProgress] = useState(false);
  const [manualUrl, setManualUrl] = useState('');

  useEffect(() => {
    loadUpdateSettings();
    loadManualUrl();
  }, []);

  useEffect(() => {
    if (availableUpdate && !availableUpdate.is_latest) {
      setShowUpdateModal(true);
    }
  }, [availableUpdate]);

  useEffect(() => {
    if (downloadProgress.state !== 'idle') {
      setShowProgress(true);
    }
    if (downloadProgress.state === 'ready' || downloadProgress.state === 'error') {
      setTimeout(() => setShowProgress(false), 3000);
    }
  }, [downloadProgress.state]);

  const loadManualUrl = async () => {
    try {
      const url = await invoke<string>('get_manual_download_url');
      setManualUrl(url);
    } catch (e) {
      console.error('Failed to load manual URL:', e);
      setManualUrl('https://github.com/dferr/super-punch-out-editor/releases');
    }
  };

  const handleCheckForUpdates = async () => {
    const update = await checkForUpdates();
    if (!update || update.is_latest) {
      showToast(update ? 'You are running the latest version.' : 'No update information available.', 'info');
    }
  };

  const handleDownload = async () => {
    setShowUpdateModal(false);
    await downloadAndInstallUpdate();
  };

  const handleSkip = async (version: string) => {
    await skipVersion(version);
    setShowUpdateModal(false);
  };

  const handleSettingChange = async <K extends keyof UpdateSettingsType>(
    key: K,
    value: UpdateSettingsType[K]
  ) => {
    const newSettings = { ...updateSettings, [key]: value };
    await saveUpdateSettings(newSettings);
  };

  const handleOpenManualDownload = async () => {
    try {
      await openUrl(manualUrl);
    } catch (e) {
      console.error('Failed to open download URL:', e);
    }
  };

  const skippedCount = updateSettings.skipped_versions?.length || 0;

  return (
    <div
      style={{
        padding: '1.5rem',
        backgroundColor: 'var(--panel-bg)',
        borderRadius: '12px',
        border: '1px solid var(--border)',
        maxWidth: '600px',
      }}
    >
      <h2
        style={{
          margin: '0 0 1.5rem 0',
          fontSize: '1.25rem',
          color: 'var(--text)',
          display: 'flex',
          alignItems: 'center',
          gap: '0.5rem',
        }}
      >
        🔄 Auto-Updater
      </h2>

      {/* Current Version */}
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          padding: '1rem',
          backgroundColor: 'var(--glass)',
          borderRadius: '8px',
          marginBottom: '1.5rem',
        }}
      >
        <div>
          <div style={{ fontSize: '0.875rem', color: 'var(--text-dim)' }}>Current Version</div>
          <div style={{ fontSize: '1.25rem', fontWeight: 600, color: 'var(--text)' }}>
            v{currentVersion}
          </div>
        </div>
        <button
          onClick={handleCheckForUpdates}
          disabled={checkingForUpdate}
          style={{
            padding: '0.5rem 1rem',
            backgroundColor: checkingForUpdate ? 'var(--glass)' : 'var(--blue)',
            border: 'none',
            borderRadius: '6px',
            color: 'white',
            cursor: checkingForUpdate ? 'not-allowed' : 'pointer',
            fontSize: '0.875rem',
            opacity: checkingForUpdate ? 0.6 : 1,
          }}
        >
          {checkingForUpdate ? '⏳ Checking...' : '🔍 Check Now'}
        </button>
      </div>

      {/* Settings */}
      <div style={{ marginBottom: '1.5rem' }}>
        <h3
          style={{
            fontSize: '0.875rem',
            color: 'var(--text-dim)',
            textTransform: 'uppercase',
            letterSpacing: '0.05em',
            marginBottom: '1rem',
          }}
        >
          Update Settings
        </h3>

        {/* Check on startup */}
        <label
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '0.75rem',
            padding: '0.75rem 0',
            cursor: 'pointer',
          }}
        >
          <input
            type="checkbox"
            checked={updateSettings.check_on_startup}
            onChange={(e) => handleSettingChange('check_on_startup', e.target.checked)}
            style={{
              width: '1.25rem',
              height: '1.25rem',
              accentColor: 'var(--blue)',
            }}
          />
          <span style={{ color: 'var(--text)' }}>Check for updates on startup</span>
        </label>

        {/* Check interval */}
        <div style={{ marginTop: '1rem' }}>
          <label
            style={{
              display: 'block',
              fontSize: '0.875rem',
              color: 'var(--text-dim)',
              marginBottom: '0.5rem',
            }}
          >
            Check Interval
          </label>
          <select
            value={updateSettings.check_interval}
            onChange={(e) =>
              handleSettingChange('check_interval', e.target.value as UpdateSettingsType['check_interval'])
            }
            disabled={!updateSettings.check_on_startup}
            style={{
              width: '100%',
              padding: '0.5rem',
              backgroundColor: 'var(--glass)',
              border: '1px solid var(--border)',
              borderRadius: '6px',
              color: 'var(--text)',
              fontSize: '0.875rem',
              opacity: !updateSettings.check_on_startup ? 0.5 : 1,
            }}
          >
            {UPDATE_INTERVALS.map((interval) => (
              <option key={interval.value} value={interval.value}>
                {interval.label}
              </option>
            ))}
          </select>
        </div>

        {/* Release channel */}
        <div style={{ marginTop: '1rem' }}>
          <label
            style={{
              display: 'block',
              fontSize: '0.875rem',
              color: 'var(--text-dim)',
              marginBottom: '0.5rem',
            }}
          >
            Release Channel
          </label>
          <select
            value={updateSettings.channel}
            onChange={(e) =>
              handleSettingChange('channel', e.target.value as UpdateSettingsType['channel'])
            }
            style={{
              width: '100%',
              padding: '0.5rem',
              backgroundColor: 'var(--glass)',
              border: '1px solid var(--border)',
              borderRadius: '6px',
              color: 'var(--text)',
              fontSize: '0.875rem',
            }}
          >
            {RELEASE_CHANNELS.map((channel) => (
              <option key={channel.value} value={channel.value}>
                {channel.label}
              </option>
            ))}
          </select>
        </div>
      </div>

      {/* Skipped Versions */}
      {skippedCount > 0 && (
        <div
          style={{
            padding: '1rem',
            backgroundColor: 'rgba(251, 191, 36, 0.1)',
            border: '1px solid rgba(251, 191, 36, 0.3)',
            borderRadius: '8px',
            marginBottom: '1.5rem',
          }}
        >
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <div>
              <div style={{ fontSize: '0.875rem', color: '#fbbf24' }}>
                ⚠️ {skippedCount} version{skippedCount === 1 ? '' : 's'} skipped
              </div>
              <div style={{ fontSize: '0.75rem', color: 'var(--text-dim)', marginTop: '0.25rem' }}>
                You won't be notified about these versions again
              </div>
            </div>
            <button
              onClick={clearSkippedVersions}
              style={{
                padding: '0.375rem 0.75rem',
                backgroundColor: 'transparent',
                border: '1px solid #fbbf24',
                borderRadius: '4px',
                color: '#fbbf24',
                cursor: 'pointer',
                fontSize: '0.75rem',
              }}
            >
              Clear
            </button>
          </div>
        </div>
      )}

      {/* Manual Update */}
      <div
        style={{
          padding: '1rem',
          backgroundColor: 'var(--glass)',
          borderRadius: '8px',
        }}
      >
        <div style={{ fontSize: '0.875rem', color: 'var(--text-dim)', marginBottom: '0.5rem' }}>
          Manual Update
        </div>
        <p style={{ fontSize: '0.75rem', color: 'var(--text-dim)', margin: '0 0 0.75rem 0' }}>
          If automatic updates aren't working, you can download the latest version manually.
        </p>
        <button
          onClick={handleOpenManualDownload}
          style={{
            padding: '0.5rem 1rem',
            backgroundColor: 'transparent',
            border: '1px solid var(--border)',
            borderRadius: '6px',
            color: 'var(--text)',
            cursor: 'pointer',
            fontSize: '0.875rem',
          }}
        >
          📥 Download from GitHub
        </button>
      </div>

      {/* Error Display */}
      {updateError && (
        <div
          style={{
            marginTop: '1rem',
            padding: '0.75rem',
            backgroundColor: 'rgba(239, 68, 68, 0.1)',
            border: '1px solid rgba(239, 68, 68, 0.3)',
            borderRadius: '6px',
            color: '#ef4444',
            fontSize: '0.875rem',
          }}
        >
          ❌ {updateError}
        </div>
      )}

      {/* Last Check */}
      {updateSettings.last_check && (
        <div
          style={{
            marginTop: '1rem',
            fontSize: '0.75rem',
            color: 'var(--text-dim)',
            textAlign: 'center',
          }}
        >
          Last checked: {new Date(updateSettings.last_check).toLocaleString()}
        </div>
      )}

      {/* Update Available Modal */}
      {showUpdateModal && availableUpdate && !availableUpdate.is_latest && (
        <UpdateAvailableModal
          update={availableUpdate}
          currentVersion={currentVersion}
          onClose={() => setShowUpdateModal(false)}
          onDownload={handleDownload}
          onSkip={handleSkip}
          manualDownloadUrl={manualUrl}
        />
      )}

      {/* Update Progress */}
      {showProgress && <UpdateProgress progress={downloadProgress} />}
    </div>
  );
}
