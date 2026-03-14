import { useTranslation } from 'react-i18next';
import { Card } from '../ui/card';
import { Button } from '../ui/button';
import { List, Loader2 } from 'lucide-react';
import type { UsagePattern } from '../../types';
import { HighlightedText } from '../../utils/textHighlight';
import type { FeatureState } from '../../stores/ExploreStore';

interface UsageCardProps {
  currentWord: string;
  state: FeatureState;
  patterns: UsagePattern[];
  error: string | null;
  onGenerate: () => Promise<void> | void;
}

export const UsageCard = ({
  currentWord,
  state,
  patterns,
  error,
  onGenerate,
}: UsageCardProps) => {
  const { t } = useTranslation();

  const cardGradientClass = 'define-card define-card--accent-plum';

  if (state === 'ungenerated') {
    return (
      <Card className={`p-8 text-center ${cardGradientClass}`}>
        <div className="mb-4">
          <List className="w-16 h-16 mx-auto text-muted-foreground" />
        </div>
        <h3 className="text-lg font-medium mb-2">{t('explore.usageCard.title')}</h3>
        <p className="text-muted-foreground mb-6">
          {t('explore.usageCard.description', { word: currentWord })}
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
        <p className="text-muted-foreground">{t('explore.usageCard.analyzing')}</p>
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
      <div className="mb-6">
        <h2 className="text-xl font-semibold">{t('explore.usageCard.sectionTitle')}</h2>
      </div>

        <div className="space-y-6">
        {patterns.map((pattern: UsagePattern, index: number) => (
          <div key={`${pattern.template}-${index}`} className="pb-6 border-b border-border last:border-0 last:pb-0">
            <div className="mb-3">
              <code className="font-mono text-sm bg-muted px-3 py-1.5 rounded">
                {pattern.template}
              </code>
            </div>
            
            <p className="definition-content text-sm text-muted-foreground mb-4">{pattern.description}</p>
            
              <div className="space-y-3">
                {pattern.examples.map((example: string, i: number) => (
                <div key={`${example}-${i}`} className="example-text pl-4 border-l-2 border-border text-base text-foreground leading-relaxed">
                  <HighlightedText text={example} word={currentWord} />
                </div>
              ))}
            </div>
          </div>
        ))}
      </div>
    </Card>
  );
};
