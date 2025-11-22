import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';
import { api } from '../../services/api';
import { Trash2 } from 'lucide-react';
import { toast } from 'sonner';
import { useState } from 'react';

export const CacheSection = observer(() => {
  const { t } = useTranslation();
  const [clearingCache, setClearingCache] = useState(false);
  const [confirmDialog, setConfirmDialog] = useState<{
    open: boolean;
    type: 'all' | 'definitions' | 'explorations' | null;
    title: string;
    description: string;
  }>({ open: false, type: null, title: '', description: '' });

  const openConfirmDialog = (type: 'all' | 'definitions' | 'explorations') => {
    const titles = {
      all: t('settings.cache.clearAll'),
      definitions: t('settings.cache.clearDefinitions'),
      explorations: t('settings.cache.clearExplorations')
    };

    const descriptions = {
      all: t('settings.cache.confirmClearAll'),
      definitions: t('settings.cache.confirmClearDefinitions'),
      explorations: t('settings.cache.confirmClearExplorations')
    };

    setConfirmDialog({
      open: true,
      type,
      title: titles[type],
      description: descriptions[type]
    });
  };

  const handleConfirmClear = async () => {
    const type = confirmDialog.type;
    if (!type) return;

    setClearingCache(true);
    setConfirmDialog({ ...confirmDialog, open: false });

    try {
      if (type === 'all') {
        await api.clearAllCache();
        toast.success(t('settings.cache.successAll'));
      } else if (type === 'definitions') {
        await api.clearDefinitionCache();
        toast.success(t('settings.cache.successDefinitions'));
      } else if (type === 'explorations') {
        await api.clearExplorationCache();
        toast.success(t('settings.cache.successExplorations'));
      }
    } catch (error) {
      console.error('Failed to clear cache:', error);
      toast.error(t('errors.cacheOperation'));
    } finally {
      setClearingCache(false);
    }
  };

  return (
    <div className="space-y-4">
      <div>
        <h3 className="text-base font-semibold text-foreground mb-3">{t('settings.cache.clearCache')}</h3>
        <div className="space-y-2">
          <button
            onClick={() => openConfirmDialog('definitions')}
            disabled={clearingCache}
            className="w-full flex items-center justify-between p-3 border border-border rounded-lg hover:bg-accent transition-colors text-left disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <div>
              <div className="text-sm font-medium text-foreground">{t('settings.cache.clearDefinitions')}</div>
              <div className="text-xs text-muted-foreground">{t('settings.cache.clearDefinitionsDesc')}</div>
            </div>
            <Trash2 className="w-4 h-4 text-muted-foreground flex-shrink-0" />
          </button>

          <button
            onClick={() => openConfirmDialog('explorations')}
            disabled={clearingCache}
            className="w-full flex items-center justify-between p-3 border border-border rounded-lg hover:bg-accent transition-colors text-left disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <div>
              <div className="text-sm font-medium text-foreground">{t('settings.cache.clearExplorations')}</div>
              <div className="text-xs text-muted-foreground">{t('settings.cache.clearExplorationsDesc')}</div>
            </div>
            <Trash2 className="w-4 h-4 text-muted-foreground flex-shrink-0" />
          </button>

          <button
            onClick={() => openConfirmDialog('all')}
            disabled={clearingCache}
            className="w-full flex items-center justify-between p-3 border-2 border-destructive/50 rounded-lg hover:bg-destructive/10 transition-colors text-left disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <div>
              <div className="text-sm font-medium text-destructive">{t('settings.cache.clearAll')}</div>
              <div className="text-xs text-muted-foreground">{t('settings.cache.clearAllDesc')}</div>
            </div>
            <Trash2 className="w-4 h-4 text-destructive flex-shrink-0" />
          </button>
        </div>
      </div>

      {confirmDialog.open && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-card p-5 rounded-lg border border-border max-w-sm mx-4">
            <h3 className="text-base font-semibold text-foreground mb-2">{confirmDialog.title}</h3>
            <p className="text-sm text-muted-foreground mb-5">{confirmDialog.description}</p>
            <div className="flex gap-2 justify-end">
              <button
                onClick={() => setConfirmDialog({ ...confirmDialog, open: false })}
                className="px-3 py-1.5 text-sm border border-border rounded-lg hover:bg-accent transition-colors"
              >
                {t('common.cancel')}
              </button>
              <button
                onClick={handleConfirmClear}
                disabled={clearingCache}
                className="px-3 py-1.5 text-sm bg-destructive text-destructive-foreground rounded-lg hover:bg-destructive/90 transition-colors disabled:opacity-50"
              >
                {clearingCache ? t('settings.cache.clearing') : t('settings.cache.clearCache')}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
});
