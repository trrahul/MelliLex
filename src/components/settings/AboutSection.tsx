import { observer } from 'mobx-react-lite';
import { ExternalLink, RefreshCw } from 'lucide-react';
import { Button } from '../ui/button';
import { openUrl } from '@tauri-apps/plugin-opener';
import { toast } from 'sonner';
import { useTranslation } from 'react-i18next';
import { useUpdateChecker } from '../../hooks/useUpdateChecker';
import { UpdateDialog } from '../UpdateDialog';

export const AboutSection = observer(() => {
  const { t } = useTranslation();
  const {
    status,
    currentVersion,
    newVersion,
    downloadProgress,
    error,
    checkForUpdate,
    downloadAndInstall,
    restartApp,
    dismiss,
  } = useUpdateChecker();

  const checking = status === 'checking';
  const showDialog = status === 'available' || status === 'downloading' || status === 'ready' || status === 'error';

  const handleCheckUpdates = async () => {
    const result = await checkForUpdate();
    if (result === 'up-to-date') {
      toast.success(t('settings.about.upToDate'));
    }
  };

  return (
    <div className="space-y-6">
      <div>
        <div className="space-y-2 text-sm">
          <div className="flex justify-between py-2 border-b border-border">
            <span className="text-muted-foreground">{t('settings.about.version')}</span>
            <span className="font-mono text-foreground">{currentVersion}</span>
          </div>
        </div>
      </div>

      <div>
        <Button
          onClick={handleCheckUpdates}
          disabled={checking}
          className="w-full"
          variant="outline"
        >
          <RefreshCw className={`w-4 h-4 mr-2 ${checking ? 'animate-spin' : ''}`} />
          {checking ? t('settings.about.checkingUpdates') : t('settings.about.checkUpdates')}
        </Button>
      </div>

      <div>
        <div className="space-y-2">
          <button
            onClick={() => openUrl('https://github.com/trrahul/MelliLex')}
            className="w-full flex items-center justify-between p-3 border border-border rounded-lg hover:bg-accent transition-colors text-left"
          >
            <span className="text-sm font-medium">{t('settings.about.github')}</span>
            <ExternalLink className="w-4 h-4 text-muted-foreground" />
          </button>

          <button
            onClick={() => openUrl('https://github.com/trrahul/MelliLex/issues')}
            className="w-full flex items-center justify-between p-3 border border-border rounded-lg hover:bg-accent transition-colors text-left"
          >
            <span className="text-sm font-medium">{t('settings.about.reportIssue')}</span>
            <ExternalLink className="w-4 h-4 text-muted-foreground" />
          </button>

          <button
            onClick={() => openUrl('https://github.com/trrahul/MelliLex/blob/master/LICENSE')}
            className="w-full flex items-center justify-between p-3 border border-border rounded-lg hover:bg-accent transition-colors text-left"
          >
            <span className="text-sm font-medium">{t('settings.about.license')}</span>
            <ExternalLink className="w-4 h-4 text-muted-foreground" />
          </button>
        </div>
      </div>

      <UpdateDialog
        open={showDialog}
        status={status}
        currentVersion={currentVersion}
        newVersion={newVersion}
        downloadProgress={downloadProgress}
        error={error}
        onDownload={downloadAndInstall}
        onRestart={restartApp}
        onDismiss={dismiss}
      />
    </div>
  );
});
