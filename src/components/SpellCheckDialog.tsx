import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from './ui/dialog';
import { Button } from './ui/button';
import type { SpellCheckResponse } from '../types';

interface SpellCheckDialogProps {
  open: boolean;
  spellCheckData: SpellCheckResponse | null;
  onSelectWord: (word: string) => void;
  onCancel: () => void;
}

export const SpellCheckDialog = observer(
  ({ open, spellCheckData, onSelectWord, onCancel }: SpellCheckDialogProps) => {
    const { t } = useTranslation();
    
    if (!spellCheckData || spellCheckData.isCorrect) {
      return null;
    }

    const { originalWord, suggestedWord, alternatives } = spellCheckData;

    // Build list of unique alternatives (suggested word first, then others)
    const allAlternatives = [
      suggestedWord,
      ...alternatives.filter((alt) => alt !== suggestedWord),
    ].filter((word): word is string => word !== null && word !== undefined);

    return (
      <Dialog open={open} onOpenChange={(isOpen) => !isOpen && onCancel()}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>{t('spellCheck.title')}</DialogTitle>
            <DialogDescription>
              {t('spellCheck.misspelled', { word: originalWord })}
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-2 py-4">
            {allAlternatives.length > 0 ? (
              allAlternatives.map((word, index) => (
                <Button
                  key={`${word}-${index}`}
                  variant={index === 0 ? 'default' : 'outline'}
                  className="w-full justify-start text-left"
                  onClick={() => onSelectWord(word)}
                >
                  {word}
                  {index === 0 && (
                    <span className="ml-2 text-xs text-muted-foreground">{t('spellCheck.recommended')}</span>
                  )}
                </Button>
              ))
            ) : (
              <p className="text-sm text-muted-foreground text-center py-4">
                {t('spellCheck.noSuggestions')}
              </p>
            )}
          </div>

          <DialogFooter className="sm:justify-between">
            <Button variant="ghost" onClick={onCancel}>
              {t('common.cancel')}
            </Button>
            <Button variant="outline" onClick={() => onSelectWord(originalWord)}>
              {t('spellCheck.useAnyway', { word: originalWord })}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    );
  }
);
