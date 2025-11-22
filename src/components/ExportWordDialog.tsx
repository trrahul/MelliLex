import { useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogTrigger } from './ui/dialog';
import { Button } from './ui/button';
import { Download, Send } from 'lucide-react';
import { ExportService, type WordExportPayload } from '../services/ExportService';
import { useStores } from '../stores/RootStore';
import { toast } from 'sonner';
import { ERROR_MESSAGES, getErrorMessage } from '../utils/errorHandler';

export function ExportWordDialog() {
  const { t } = useTranslation();
  const { progressiveWordStore, settingsStore, exploreStore } = useStores();
  const [open, setOpen] = useState(false);
  const [busy, setBusy] = useState<'markdown' | 'capacities' | null>(null);

  const includeExploration =
    settingsStore.settings.exportSettings?.includeExploration ?? false;
  const includeTimestamp =
    settingsStore.settings.exportSettings?.capacities?.noTimestamp === true
      ? false
      : true;

  const explorationPayload = useMemo(() => {
    const result: WordExportPayload['exploration'] = {};
    let hasContent = false;

    if (
      exploreStore.formalityPercentage !== null ||
      exploreStore.formalityAlternatives.length > 0
    ) {
      result.formality = {
        percentage: exploreStore.formalityPercentage,
        alternatives: exploreStore.formalityAlternatives.slice(),
      };
      hasContent = true;
    }

    if (exploreStore.domainExplorations.length > 0) {
      result.domains = exploreStore.domainExplorations.map((domain) => ({ ...domain }));
      hasContent = true;
    }

    if (exploreStore.usagePatterns.length > 0) {
      result.usage = exploreStore.usagePatterns.map((pattern) => ({ ...pattern }));
      hasContent = true;
    }

    if (exploreStore.commonMistakes.length > 0) {
      result.mistakes = exploreStore.commonMistakes.map((mistake) => ({ ...mistake }));
      hasContent = true;
    }

    if (exploreStore.practiceExercises.length > 0) {
      result.practice = exploreStore.practiceExercises.map((exercise) => ({ ...exercise }));
      hasContent = true;
    }

    if (exploreStore.customExamples.length > 0) {
      result.customContext = {
        label: exploreStore.customContext,
        examples: exploreStore.customExamples.slice(),
      };
      hasContent = true;
    }

    return hasContent ? result : undefined;
  }, [
    exploreStore.commonMistakes,
    exploreStore.customContext,
    exploreStore.customExamples,
    exploreStore.domainExplorations,
    exploreStore.formalityAlternatives,
    exploreStore.formalityPercentage,
    exploreStore.practiceExercises,
    exploreStore.usagePatterns,
  ]);

  const payload = useMemo<WordExportPayload>(
    () => ({
      header: progressiveWordStore.headerSection,
      meanings: progressiveWordStore.meaningsSection,
      related: progressiveWordStore.relatedSection,
      exploration: explorationPayload,
    }),
    [
      progressiveWordStore.headerSection,
      progressiveWordStore.meaningsSection,
      progressiveWordStore.relatedSection,
      explorationPayload,
    ]
  );

  const word = progressiveWordStore.headerSection?.word ?? 'word';
  const capacitiesConfig = settingsStore.settings.exportSettings?.capacities;
  const hasCapacitiesConfig =
    Boolean(capacitiesConfig?.apiToken) && Boolean(capacitiesConfig?.spaceId);

  const handleMarkdownExport = async () => {
    setBusy('markdown');
    try {
      const provider = settingsStore.settings.aiProvider;
      const savedPath = await ExportService.exportToMarkdownFile(
        word,
        provider,
        includeTimestamp
      );
      toast.success(t('exportDialog.markdownSuccess', { word, path: savedPath }));
      setOpen(false);
    } catch (error) {
      const message = getErrorMessage(error, ERROR_MESSAGES.EXPORT_MARKDOWN_FAILED);
      toast.error(message);
    } finally {
      setBusy(null);
    }
  };

  const handleCapacitiesExport = async () => {
    if (!hasCapacitiesConfig || !capacitiesConfig) {
      toast.error(ERROR_MESSAGES.EXPORT_CONFIG_MISSING);
      return;
    }

    setBusy('capacities');
    try {
      await ExportService.exportToCapacities(payload, capacitiesConfig, {
        includeExploration: includeExploration && Boolean(explorationPayload),
        includeTimestamp,
      });
      toast.success(t('exportDialog.capacitiesSuccess', { word }));
      setOpen(false);
    } catch (error) {
      const message = getErrorMessage(error, ERROR_MESSAGES.EXPORT_CAPACITIES_FAILED);
      toast.error(message);
    } finally {
      setBusy(null);
    }
  };

  const disabled = !progressiveWordStore.headerSection;

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <button
          disabled={disabled}
          className="flex items-center gap-2 px-4 py-2 rounded-lg border border-border bg-card hover:bg-accent hover:border-foreground transition-all text-sm font-medium disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <Download className="w-4 h-4" />
          {t('exportDialog.export')}
        </button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t('exportDialog.title', { word })}</DialogTitle>
          <DialogDescription>
            {t('exportDialog.description')}
          </DialogDescription>
        </DialogHeader>

        <div className="grid gap-4">
          <Button
            onClick={handleMarkdownExport}
            disabled={busy !== null}
            className="justify-start"
          >
            <Download className="w-4 h-4" />
            {busy === 'markdown' ? t('exportDialog.saving') : t('exportDialog.exportMarkdown')}
          </Button>

          <Button
            variant="secondary"
            onClick={handleCapacitiesExport}
            disabled={busy !== null || !hasCapacitiesConfig}
            className="justify-start"
          >
            <Send className="w-4 h-4" />
            {busy === 'capacities'
              ? t('exportDialog.sending')
              : hasCapacitiesConfig
              ? t('exportDialog.sendCapacities')
              : t('exportDialog.configureFirst')}
          </Button>
        </div>

        {!hasCapacitiesConfig && (
          <p className="text-xs text-muted-foreground">
            {t('exportDialog.capacitiesNote')}
          </p>
        )}
      </DialogContent>
    </Dialog>
  );
}
