import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogTrigger } from '../ui/dialog';
import { Button } from '../ui/button';
import { Download, Send } from 'lucide-react';
import { ExportService } from '../../services/ExportService';
import { useStores } from '../../stores/RootStore';
import { toast } from 'sonner';
import { ERROR_MESSAGES, getErrorMessage } from '../../utils/errorHandler';
import type { PhraseDefinitionData } from '../../types';

export function ExportPhraseDialog() {
  const { t } = useTranslation();
  const { phraseStore, settingsStore } = useStores();
  const [open, setOpen] = useState(false);
  const [busy, setBusy] = useState<'markdown' | 'capacities' | null>(null);

  const includeTimestamp =
    settingsStore.settings.exportSettings?.capacities?.noTimestamp === true
      ? false
      : true;

  const phraseData = phraseStore.getPhraseDefinitionData();
  const phrase = phraseData?.section1.phrase ?? 'phrase';
  const capacitiesConfig = settingsStore.settings.exportSettings?.capacities;
  const hasCapacitiesConfig =
    Boolean(capacitiesConfig?.apiToken) && Boolean(capacitiesConfig?.spaceId);

  const handleMarkdownExport = async () => {
    setBusy('markdown');
    try {
      const provider = settingsStore.settings.aiProvider;
      const savedPath = await ExportService.exportPhraseToMarkdownFile(
        phrase,
        provider,
        includeTimestamp
      );
      toast.success(t('phrase.exportSuccess', { path: savedPath }));
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

    if (!phraseData) {
      toast.error(t('phrase.exportNoData'));
      return;
    }

    setBusy('capacities');
    try {
      // For Capacities, build markdown locally since Capacities API needs specific format
      const markdown = buildPhraseMarkdownForCapacities(phraseData, includeTimestamp);
      await ExportService.exportPhraseToCapacities(markdown, capacitiesConfig);
      toast.success(t('exportDialog.capacitiesSuccess', { word: phrase }));
      setOpen(false);
    } catch (error) {
      const message = getErrorMessage(error, ERROR_MESSAGES.EXPORT_CAPACITIES_FAILED);
      toast.error(message);
    } finally {
      setBusy(null);
    }
  };

  const disabled = !phraseData;

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
          <DialogTitle>{t('exportDialog.title', { word: phrase })}</DialogTitle>
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

function buildPhraseMarkdownForCapacities(
  data: PhraseDefinitionData,
  includeTimestamp: boolean
): string {
  const lines: string[] = [];

  // Header
  lines.push(`# ${data.section1.phrase}`);
  lines.push('');
  lines.push(`**Type:** ${data.section1.phraseType} · **Region:** ${data.section1.region} · **Formality:** ${data.section1.formality.level}`);
  lines.push('');

  // TL;DR
  lines.push('## In a Nutshell');
  lines.push(data.section1.tldr);
  lines.push('');

  // Meanings
  if (data.section1.literalMeaning) {
    lines.push('## Literal Meaning');
    lines.push(data.section1.literalMeaning);
    lines.push('');
  }

  lines.push('## Actual Meaning');
  lines.push(data.section1.actualMeaning);
  lines.push('');

  // Origin Story
  lines.push('## Origin Story');
  if (data.section2.origin.era) {
    lines.push(`**Era:** ${data.section2.origin.era}`);
  }
  if (data.section2.origin.source) {
    lines.push(`**Source:** ${data.section2.origin.source}`);
  }
  lines.push('');
  lines.push(data.section2.origin.story);
  if (data.section2.origin.evolution) {
    lines.push('');
    lines.push(`**Evolution:** ${data.section2.origin.evolution}`);
  }
  lines.push('');

  // Usage Notes
  if (data.section2.usageNotes.length > 0) {
    lines.push('## Usage Notes');
    data.section2.usageNotes.forEach((note) => {
      lines.push(`### ${note.context}${note.tone ? ` (${note.tone})` : ''}`);
      lines.push(`> ${note.example}`);
      lines.push('');
    });
  }

  // Common Mistakes
  if (data.section2.commonMistakes.length > 0) {
    lines.push('## Common Mistakes');
    data.section2.commonMistakes.forEach((mistake) => {
      lines.push(`### ${mistake.mistakeType}`);
      lines.push(`Incorrect: ${mistake.incorrect}`);
      lines.push(`Correct: ${mistake.correct}`);
      lines.push(`> ${mistake.explanation}`);
      lines.push('');
    });
  }

  // Related
  if (data.section3.variations.length > 0) {
    lines.push('## Variations');
    data.section3.variations.forEach((v) => {
      lines.push(`- **${v.phrase}**${v.note ? ` — ${v.note}` : ''}${v.region && v.region !== 'universal' ? ` (${v.region})` : ''}`);
    });
    lines.push('');
  }

  if (data.section3.similarPhrases.length > 0) {
    lines.push('## Similar Phrases');
    data.section3.similarPhrases.forEach((p) => {
      lines.push(`- **${p.phrase}** — ${p.meaningHint}`);
    });
    lines.push('');
  }

  if (data.section3.oppositePhrases.length > 0) {
    lines.push('## Opposite Phrases');
    data.section3.oppositePhrases.forEach((p) => {
      lines.push(`- **${p.phrase}** — ${p.meaningHint}`);
    });
    lines.push('');
  }

  // Timestamp
  if (includeTimestamp) {
    lines.push('---');
    lines.push(`*Exported from MelliLex on ${new Date().toLocaleDateString()}*`);
  }

  return lines.join('\n');
}
