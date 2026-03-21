import { useEffect, useState } from 'react';
import { useStore } from '../store/useStore';
import { UpdateAvailableModal } from './UpdateAvailableModal';
import { UpdateProgress } from './UpdateProgress';
import { openUrl } from '@tauri-apps/plugin-opener';

interface UpdateCheckerProps {
  children: React.ReactNode;
}

export function UpdateChecker({ children }: UpdateCheckerProps) {
  const {
    updateSettings,
    currentVersion,
    availableUpdate,
    downloadProgress,
    loadUpdateSettings,
    checkForUpdates,
    skipVersion,
    downloadAndInstallUpdate,
  } = useStore();

  const [showUpdateModal, setShowUpdateModal] = useState(false);
  const [showProgress, setShowProgress] = useState(false);
  const [manualUrl, setManualUrl] = useState('');
  const [hasCheckedOnStartup, setHasCheckedOnStartup] = useState(false);

  // Load settings and check for updates on mount
  useEffect(() => {
    const initialize = async () => {
      await loadUpdateSettings();
      await loadManualUrl();
    };
    initialize();
  }, []);

  // Auto-check on startup if enabled
  useEffect(() => {
    if (hasCheckedOnStartup) return;
    
    const autoCheck = async () => {
      if (!updateSettings.check_on_startup) {
        setHasCheckedOnStartup(true);
        return;
      }

      // Check if enough time has passed since last check
      const shouldCheck = await checkShouldAutoCheck();
      if (shouldCheck) {
        const update = await checkForUpdates();
        if (update && !update.is_latest) {
          // Delay showing the modal slightly so the app can finish loading
          setTimeout(() => {
            setShowUpdateModal(true);
          }, 2000);
        }
      }
      setHasCheckedOnStartup(true);
    };

    if (updateSettings.check_on_startup !== undefined) {
      autoCheck();
    }
  }, [updateSettings.check_on_startup, hasCheckedOnStartup]);

  // Show modal when an update becomes available
  useEffect(() => {
    if (availableUpdate && !availableUpdate.is_latest && hasCheckedOnStartup) {
      setShowUpdateModal(true);
    }
  }, [availableUpdate, hasCheckedOnStartup]);

  // Show progress when downloading
  useEffect(() => {
    if (downloadProgress.state !== 'idle') {
      setShowProgress(true);
    }
    if (downloadProgress.state === 'ready' || downloadProgress.state === 'error') {
      // Keep the progress visible for a moment before hiding
      const timer = setTimeout(() => setShowProgress(false), 3000);
      return () => clearTimeout(timer);
    }
  }, [downloadProgress.state]);

  const loadManualUrl = async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const url = await invoke<string>('get_manual_download_url');
      setManualUrl(url);
    } catch (e) {
      console.error('Failed to load manual URL:', e);
      setManualUrl('https://github.com/dferr/super-punch-out-editor/releases');
    }
  };

  const checkShouldAutoCheck = async (): Promise<boolean> => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke<boolean>('should_auto_check');
    } catch (e) {
      console.error('Failed to check auto update eligibility:', e);
      return false;
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

  const handleClose = () => {
    setShowUpdateModal(false);
  };

  const handleOpenChangelog = async () => {
    try {
      await openUrl(manualUrl);
    } catch (e) {
      console.error('Failed to open changelog:', e);
    }
  };

  return (
    <>
      {children}
      
      {/* Update Available Modal */}
      {showUpdateModal && availableUpdate && !availableUpdate.is_latest && (
        <UpdateAvailableModal
          update={availableUpdate}
          currentVersion={currentVersion}
          onClose={handleClose}
          onDownload={handleDownload}
          onSkip={handleSkip}
          manualDownloadUrl={manualUrl}
        />
      )}

      {/* Update Progress */}
      {showProgress && (
        <UpdateProgress 
          progress={downloadProgress} 
          onCancel={() => {
            // Cancel functionality would require implementing cancel in the backend
            // For now, we just hide the progress
            setShowProgress(false);
          }}
        />
      )}
    </>
  );
}
