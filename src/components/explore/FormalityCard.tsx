import { useTranslation } from 'react-i18next';
import { Card } from '../ui/card';
import { Button } from '../ui/button';
import { MessageSquare, Loader2, MessageCircle, Minus, Crown } from 'lucide-react';
import type { FormalityAlternative } from '../../types';
import type { FeatureState } from '../../stores/ExploreStore';

interface FormalityCardProps {
  currentWord: string;
  state: FeatureState;
  percentage: number | null;
  alternatives: FormalityAlternative[];
  error: string | null;
  onGenerate: () => Promise<void> | void;
}

export const FormalityCard = ({
  currentWord,
  state,
  percentage,
  alternatives,
  error,
  onGenerate,
}: FormalityCardProps) => {
  const { t } = useTranslation();

  const cardGradientClass = 'define-card define-card--accent-blue';

  if (state === 'ungenerated') {
    return (
      <Card className={`p-8 text-center ${cardGradientClass}`}>
        <div className="mb-4">
          <MessageSquare className="w-16 h-16 mx-auto text-muted-foreground" />
        </div>
        <h3 className="text-lg font-medium mb-2">{t('explore.formalityCard.title')}</h3>
        <p className="text-muted-foreground mb-6">
          {t('explore.formalityCard.description', { word: currentWord })}
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
        <p className="text-muted-foreground">{t('explore.formalityCard.analyzing')}</p>
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

  const informal = alternatives?.find((alt: FormalityAlternative) => 
    alt.level?.toLowerCase().includes('informal') || alt.level?.toLowerCase().includes('casual')
  );
  const formal = alternatives?.find((alt: FormalityAlternative) => 
    alt.level?.toLowerCase().includes('formal')
  );
  const otherAlternatives = alternatives?.filter((alt: FormalityAlternative) => 
    alt !== informal && alt !== formal
  ) || [];

  return (
    <Card className={`p-6 ${cardGradientClass}`}>
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-xl font-semibold">{t('explore.formalityCard.sectionTitle')}</h2>
        <Button variant="ghost" size="sm" onClick={onGenerate}>
          {t('common.regenerate')}
        </Button>
      </div>

      <div className="space-y-6">
        <div className="pb-6 border-b">
          <div className="flex justify-between items-baseline mb-3">
            <span className="text-sm font-medium text-muted-foreground">{t('explore.formalityCard.formalityLevel')}</span>
            <span className="text-lg font-semibold">
              {percentage ?? 0}% ({(percentage ?? 0) < 40 ? t('explore.formalityCard.informal') : (percentage ?? 0) > 60 ? t('explore.formalityCard.formal') : t('explore.formalityCard.neutral')})
            </span>
          </div>

          <div className="relative mb-4">
            <div className="h-3 bg-muted rounded-full overflow-hidden">
              <div className="h-full flex">
                <div className="flex-1 bg-gradient-to-r from-blue-300 to-blue-200"></div>
                <div className="flex-1 bg-gradient-to-r from-gray-200 to-gray-300"></div>
                <div className="flex-1 bg-gradient-to-r from-purple-200 to-purple-300"></div>
              </div>
            </div>
            <div
              className="absolute left-0 top-1/2 -translate-y-1/2 w-0.5 h-5 bg-foreground"
              style={{ left: `${percentage ?? 0}%` }}
            />
          </div>

          <div className="flex justify-between text-xs text-muted-foreground mb-6">
            <span>{t('explore.formalityCard.informal')}</span>
            <span>{t('explore.formalityCard.neutral')}</span>
            <span>{t('explore.formalityCard.formal')}</span>
          </div>

          <div className="grid grid-cols-3 gap-4">
            <div className="text-center p-4 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg">
              <div className="flex items-center justify-center gap-1.5 text-xs font-medium text-blue-700 dark:text-blue-300 mb-2">
                <MessageCircle className="w-3.5 h-3.5" />
                <span>{t('explore.formalityCard.informal')}</span>
              </div>
              <div className="font-medium">
                {informal?.word || 'N/A'}
              </div>
              <div className="text-xs text-muted-foreground mt-1">
                {informal?.level || ''}
              </div>
            </div>

            <div className="text-center p-4 bg-muted border-2 border-primary rounded-lg">
              <div className="flex items-center justify-center gap-1.5 text-xs font-medium text-primary mb-2">
                <Minus className="w-3.5 h-3.5" />
                <span>{t('explore.formalityCard.yourSearch')}</span>
              </div>
              <div className="font-medium">
                {currentWord}
              </div>
              <div className="text-xs text-muted-foreground mt-1">
                {percentage ?? 0}%
              </div>
            </div>

            <div className="text-center p-4 bg-purple-50 dark:bg-purple-900/20 border border-purple-200 dark:border-purple-800 rounded-lg">
              <div className="flex items-center justify-center gap-1.5 text-xs font-medium text-purple-700 dark:text-purple-300 mb-2">
                <Crown className="w-3.5 h-3.5" />
                <span>{t('explore.formalityCard.formal')}</span>
              </div>
              <div className="font-medium">
                {formal?.word || 'N/A'}
              </div>
              <div className="text-xs text-muted-foreground mt-1">
                {formal?.level || ''}
              </div>
            </div>
          </div>
        </div>

        {otherAlternatives.length > 0 && (
          <div>
            <h3 className="text-base font-semibold mb-4">{t('explore.formalityCard.moreOptions')}</h3>
            <div className="space-y-3">
              {otherAlternatives.map((alt: FormalityAlternative, idx: number) => (
                <div
                  key={idx}
                  className="flex items-center justify-between p-3 border rounded-lg hover:border-muted-foreground/50 cursor-pointer transition"
                >
                  <div className="flex-1">
                    <div className="definition-content font-medium">{alt.word}</div>
                    <div className="definition-content text-sm text-muted-foreground">{alt.context || alt.explanation}</div>
                  </div>
                  <span className="px-3 py-1 bg-muted rounded text-xs font-medium">
                    {alt.level}
                  </span>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </Card>
  );
};