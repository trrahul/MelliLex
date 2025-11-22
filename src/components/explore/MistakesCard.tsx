import { useTranslation } from 'react-i18next';
import { Card } from '../ui/card';
import { Button } from '../ui/button';
import { AlertTriangle, CheckCircle, Loader2, XCircle } from 'lucide-react';
import { HighlightedText } from '../../utils/textHighlight';
import type { MistakeItem } from '../../types';
import type { FeatureState } from '../../stores/ExploreStore';

interface MistakesCardProps {
  currentWord: string;
  state: FeatureState;
  mistakes: MistakeItem[];
  error: string | null;
  onGenerate: () => Promise<void> | void;
}

export const MistakesCard = ({
  currentWord,
  state,
  mistakes,
  error,
  onGenerate,
}: MistakesCardProps) => {
  const { t } = useTranslation();

  const cardGradientClass = 'define-card define-card--accent-rose';

  if (state === 'ungenerated') {
    return (
      <Card className={`p-8 text-center ${cardGradientClass}`}>
        <div className="mb-4">
          <AlertTriangle className="w-16 h-16 mx-auto text-muted-foreground" />
        </div>
        <h3 className="text-lg font-medium mb-2">{t('explore.mistakesCard.title')}</h3>
        <p className="text-muted-foreground mb-6">
          {t('explore.mistakesCard.description', { word: currentWord })}
        </p>
        <Button onClick={onGenerate} size="lg">
          {t('common.generate')}
        </Button>
      </Card>
    );
  }

  if (state === 'generating') {
    return (
      <Card className={`p-8 text-center border-muted ${cardGradientClass}`}>
        <div className="mb-4">
          <Loader2 className="w-12 h-12 mx-auto text-primary animate-spin" />
        </div>
        <p className="text-muted-foreground">{t('explore.mistakesCard.analyzing')}</p>
      </Card>
    );
  }

  if (state === 'error') {
    return (
      <Card className={`p-8 text-center border-destructive ${cardGradientClass}`}>
        <p className="text-destructive mb-4">{error}</p>
        <Button variant="outline" onClick={onGenerate}>
          {t('common.tryAgain')}
        </Button>
      </Card>
    );
  }

  return (
    <Card className={`p-6 ${cardGradientClass}`}>
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-xl font-semibold">{t('explore.mistakesCard.sectionTitle')}</h2>
        <Button variant="ghost" size="sm" onClick={onGenerate}>
          {t('common.regenerate')}
        </Button>
      </div>

      <div className="space-y-6">
        {mistakes.length > 0 ? (
          mistakes.map((mistake: MistakeItem, index: number) => (
            <div key={index} className="pb-6 border-b border-border last:border-b-0 last:pb-0">
              <div className="mb-3">
                <h3 className="text-base font-semibold">{mistake.type}</h3>
                <span className="text-xs px-2 py-1 bg-muted rounded text-muted-foreground">
                  {mistake.category}
                </span>
              </div>

              <div className="mb-3">
                <span className="text-sm font-medium text-muted-foreground flex items-center gap-1 mb-2">
                  <XCircle className="h-4 w-4 text-red-500" />
                  <span>{t('explore.mistakesCard.incorrect')}</span>
                </span>
                <div className="example-text pl-4 border-l-2 border-red-300 text-base leading-relaxed text-muted-foreground">
                  <HighlightedText text={mistake.incorrectUsage} word={currentWord} />
                </div>
              </div>

              <div className="bg-blue-50 dark:bg-blue-900/20 border-l-4 border-blue-500 dark:border-blue-600 p-4 rounded">
                <div className="text-xs text-blue-900 dark:text-blue-200 font-bold mb-2 uppercase tracking-wider flex items-center gap-1">
                  <CheckCircle className="h-3.5 w-3.5" />
                  <span>{t('explore.mistakesCard.correctUsage')}</span>
                </div>
                <div className="definition-content text-sm text-blue-900 dark:text-blue-100 leading-relaxed">
                  <HighlightedText text={mistake.correction} word={currentWord} />
                </div>
              </div>
            </div>
          ))
        ) : (
          <p className="text-center text-muted-foreground">{t('explore.mistakesCard.noMistakes')}</p>
        )}
      </div>
    </Card>
  );
};
