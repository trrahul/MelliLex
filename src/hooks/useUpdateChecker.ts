import { useState, useEffect, useCallback, useRef } from 'react';
import { check, type Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { getVersion } from '@tauri-apps/api/app';

export type UpdateStatus = 'idle' | 'checking' | 'available' | 'downloading' | 'ready' | 'error';

export interface UpdateState {
  status: UpdateStatus;
  currentVersion: string;
  newVersion: string | null;
  downloadProgress: number; // 0-100
  error: string | null;
}

interface UseUpdateCheckerOptions {
  autoCheck?: boolean;
  autoCheckDelayMs?: number;
}

const STARTUP_CHECK_DELAY = 5000; // 5 seconds after app start

export function useUpdateChecker(options: UseUpdateCheckerOptions = {}) {
  const { autoCheck = false, autoCheckDelayMs = STARTUP_CHECK_DELAY } = options;

  const [state, setState] = useState<UpdateState>({
    status: 'idle',
    currentVersion: '...',
    newVersion: null,
    downloadProgress: 0,
    error: null,
  });

  const updateRef = useRef<Update | null>(null);

  useEffect(() => {
    getVersion().then(v => setState(s => ({ ...s, currentVersion: v }))).catch(() => {});
  }, []);

  const checkForUpdate = useCallback(async (): Promise<'available' | 'up-to-date' | 'error'> => {
    setState(s => ({ ...s, status: 'checking', error: null }));
    try {
      const update = await check();
      if (update?.available) {
        updateRef.current = update;
        setState(s => ({
          ...s,
          status: 'available',
          newVersion: update.version,
        }));
        return 'available';
      } else {
        setState(s => ({ ...s, status: 'idle', newVersion: null }));
        return 'up-to-date';
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setState(s => ({ ...s, status: 'error', error: message }));
      return 'error';
    }
  }, []);

  const downloadAndInstall = useCallback(async () => {
    const update = updateRef.current;
    if (!update) return;

    setState(s => ({ ...s, status: 'downloading', downloadProgress: 0 }));
    try {
      let totalBytes = 0;
      let downloadedBytes = 0;

      await update.downloadAndInstall((event) => {
        if (event.event === 'Started' && event.data.contentLength) {
          totalBytes = event.data.contentLength;
        } else if (event.event === 'Progress') {
          downloadedBytes += event.data.chunkLength;
          const progress = totalBytes > 0 ? Math.round((downloadedBytes / totalBytes) * 100) : 0;
          setState(s => ({ ...s, downloadProgress: Math.min(progress, 100) }));
        } else if (event.event === 'Finished') {
          setState(s => ({ ...s, downloadProgress: 100 }));
        }
      });

      setState(s => ({ ...s, status: 'ready', downloadProgress: 100 }));
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setState(s => ({ ...s, status: 'error', error: message }));
    }
  }, []);

  const restartApp = useCallback(async () => {
    await relaunch();
  }, []);

  const dismiss = useCallback(() => {
    updateRef.current = null;
    setState(s => ({ ...s, status: 'idle', newVersion: null, downloadProgress: 0, error: null }));
  }, []);

  // Auto-check on startup
  useEffect(() => {
    if (!autoCheck) return;
    const timer = setTimeout(() => {
      checkForUpdate();
    }, autoCheckDelayMs);
    return () => clearTimeout(timer);
  }, [autoCheck, autoCheckDelayMs, checkForUpdate]);

  return {
    ...state,
    checkForUpdate,
    downloadAndInstall,
    restartApp,
    dismiss,
  };
}
