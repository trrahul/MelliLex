import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';
import { Button } from '../ui/button';
import { Input } from '../ui/input';
import { useStores } from '../../stores/RootStore';
import { AlertCircle, CheckCircle2 } from 'lucide-react';

export const ExportSection = observer(() => {
  const { t } = useTranslation();
  const { settingsStore } = useStores();
  const store = settingsStore.exportSettingsStore;

  const getButtonContent = () => {
    if (store.saving) return t('common.applying');
    if (store.saved) return (
      <>
        <CheckCircle2 className="w-4 h-4" />
        {t('common.applied')}
      </>
    );
    return t('common.apply');
  };

  const getButtonClass = () => {
    const baseClass = "w-full flex items-center justify-center gap-2";
    if (store.saved) return `${baseClass} bg-green-600 hover:bg-green-700`;
    return baseClass;
  };
  
  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-semibold text-foreground mb-4">
          {t('settings.export.markdownOptions') || 'Markdown Export Options'}
        </h3>
        <p className="text-sm text-muted-foreground mb-4">
          {t('settings.export.markdownOptionsDesc') || 'Configure what to include when exporting to markdown'}
        </p>

        <div className="space-y-3">
          <div className="flex items-center gap-3">
            <input
              type="checkbox"
              id="include-exploration"
              checked={store.includeExploration}
              onChange={(e) => store.setIncludeExploration(e.target.checked)}
              className="h-4 w-4"
            />
            <label htmlFor="include-exploration" className="text-sm font-medium cursor-pointer">
              {t('settings.export.includeExploration')}
            </label>
          </div>

          <div className="flex items-center gap-3">
            <input
              type="checkbox"
              id="include-timestamp"
              checked={store.includeTimestamp}
              onChange={(e) => store.setIncludeTimestamp(e.target.checked)}
              className="h-4 w-4"
            />
            <label htmlFor="include-timestamp" className="text-sm font-medium cursor-pointer">
              {t('settings.export.includeTimestamp')}
            </label>
          </div>
        </div>
      </div>

      <div>
        <h3 className="text-lg font-semibold text-foreground mb-4">
          {t('settings.export.capacitiesIntegration')}
        </h3>
        <p className="text-sm text-muted-foreground mb-4">
          {t('settings.export.capacitiesDesc')}
        </p>

        <div className="space-y-4">
          <div className="flex items-center gap-3">
            <input
              type="checkbox"
              id="capacities-enabled"
              checked={store.enabled}
              onChange={(e) => store.setEnabled(e.target.checked)}
              className="h-4 w-4"
            />
            <label htmlFor="capacities-enabled" className="text-sm font-medium cursor-pointer">
              {t('settings.export.enableCapacities')}
            </label>
          </div>

          {store.enabled && (
            <div className="space-y-4">
              <div>
                <label className="text-sm font-medium text-foreground mb-2 block">
                  {t('settings.export.spaceId')}
                </label>
                <Input
                  type="text"
                  value={store.spaceId}
                  onChange={(e) => store.setSpaceId(e.target.value)}
                  placeholder={t('settings.export.spaceIdPlaceholder')}
                  className="w-full"
                />
                <p className="text-xs text-muted-foreground mt-1">
                  {t('settings.export.spaceIdHelp')}
                </p>
              </div>

              <div>
                <label className="text-sm font-medium text-foreground mb-2 block">
                  {t('settings.export.apiToken')}
                </label>
                <Input
                  type="password"
                  value={store.apiToken}
                  onChange={(e) => store.setApiToken(e.target.value)}
                  placeholder={t('settings.export.apiTokenPlaceholder')}
                  className="w-full"
                />
              </div>

              <div>
                <label className="text-sm font-medium text-foreground mb-2 block">
                  {t('settings.export.tags')}
                </label>
                <Input
                  type="text"
                  value={store.tags}
                  onChange={(e) => store.setTags(e.target.value)}
                  placeholder={t('settings.export.tagsPlaceholder')}
                  className="w-full"
                />
              </div>
            </div>
          )}

          {store.validationError && store.enabled && (
            <div className="flex items-center gap-2 text-sm text-red-600 dark:text-red-400">
              <AlertCircle className="w-4 h-4" />
              {store.validationError}
            </div>
          )}

          {store.error && (
            <div className="flex items-center gap-2 text-sm text-red-600 dark:text-red-400">
              <AlertCircle className="w-4 h-4" />
              {store.error}
            </div>
          )}

          <Button
            onClick={() => store.save()}
            disabled={!store.canSave}
            className={getButtonClass()}
          >
            {getButtonContent()}
          </Button>
        </div>
      </div>
    </div>
  );
});
