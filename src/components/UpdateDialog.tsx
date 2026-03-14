import { useTranslation } from 'react-i18next';
import { Download, RefreshCw, RotateCcw, ArrowRight } from 'lucide-react';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from './ui/dialog';
import { Button } from './ui/button';
import type { UpdateStatus } from '../hooks/useUpdateChecker';

interface UpdateDialogProps {
  open: boolean;
  status: UpdateStatus;
  currentVersion: string;
  newVersion: string | null;
  downloadProgress: number;
  error: string | null;
  onDownload: () => void;
  onRestart: () => void;
  onDismiss: () => void;
}

export function UpdateDialog({
  open,
  status,
  currentVersion,
  newVersion,
  downloadProgress,
  error,
  onDownload,
  onRestart,
  onDismiss,
}: UpdateDialogProps) {
  const { t } = useTranslation();

  const isClosable = status !== 'downloading';

  return (
    <Dialog open={open} onOpenChange={(isOpen) => { if (!isOpen && isClosable) onDismiss(); }}>
      <DialogContent className="sm:max-w-md" onPointerDownOutside={(e) => { if (!isClosable) e.preventDefault(); }}>
        <DialogHeader>
          <DialogTitle>
            {status === 'error' ? t('settings.update.errorTitle') : t('settings.update.title')}
          </DialogTitle>
          <DialogDescription>
            {status === 'available' && t('settings.update.description')}
            {status === 'downloading' && t('settings.update.downloading')}
            {status === 'ready' && t('settings.update.readyToRestart')}
            {status === 'error' && t('settings.update.errorOccurred')}
          </DialogDescription>
        </DialogHeader>

        <div className="py-4 space-y-4">
          {/* Version info */}
          {(status === 'available' || status === 'downloading' || status === 'ready') && (
            <div className="flex items-center justify-center gap-3 text-sm">
              <span className="font-mono px-2 py-1 rounded bg-muted text-muted-foreground">
                v{currentVersion}
              </span>
              <ArrowRight className="w-4 h-4 text-muted-foreground" />
              <span className="font-mono px-2 py-1 rounded bg-primary/10 text-primary font-medium">
                v{newVersion}
              </span>
            </div>
          )}

          {/* Download progress */}
          {status === 'downloading' && (
            <div className="space-y-2">
              <div className="w-full h-2 bg-muted rounded-full overflow-hidden">
                <div
                  className="h-full bg-primary rounded-full transition-all duration-300 ease-out"
                  style={{ width: `${downloadProgress}%` }}
                />
              </div>
              <p className="text-xs text-muted-foreground text-center">
                {downloadProgress}%
              </p>
            </div>
          )}

          {/* Error */}
          {status === 'error' && error && (
            <p className="text-sm text-destructive text-center">{error}</p>
          )}
        </div>

        <DialogFooter className="sm:justify-between gap-2">
          {status === 'available' && (
            <>
              <Button variant="ghost" onClick={onDismiss}>
                {t('settings.update.later')}
              </Button>
              <Button onClick={onDownload}>
                <Download className="w-4 h-4 mr-2" />
                {t('settings.update.downloadInstall')}
              </Button>
            </>
          )}

          {status === 'downloading' && (
            <Button disabled className="w-full">
              <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
              {t('settings.update.downloading')}
            </Button>
          )}

          {status === 'ready' && (
            <>
              <Button variant="ghost" onClick={onDismiss}>
                {t('settings.update.later')}
              </Button>
              <Button onClick={onRestart}>
                <RotateCcw className="w-4 h-4 mr-2" />
                {t('settings.update.restartNow')}
              </Button>
            </>
          )}

          {status === 'error' && (
            <Button variant="outline" onClick={onDismiss} className="w-full">
              {t('common.close')}
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
