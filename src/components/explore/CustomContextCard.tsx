import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Card } from '../ui/card';
import { Button } from '../ui/button';
import { Loader2 } from 'lucide-react';
import type { FeatureState } from '../../stores/ExploreStore';

interface CustomContextCardProps {
  state: FeatureState;
  examples: string[];
  contextLabel: string;
  error: string | null;
  onGenerate: (context: string) => Promise<void> | void;
}

export const CustomContextCard = ({
  state,
  examples,
  contextLabel,
  error,
  onGenerate,
}: CustomContextCardProps) => {
  const { t } = useTranslation();
  
  const [context, setContext] = useState('');
  const cardGradientClass = 'define-card define-card--accent-teal';

  const handleGenerate = async () => {
    if (context.trim()) {
      await onGenerate(context.trim());
      setContext('');
    }
  };

  if (state === 'ungenerated' || state === 'error') {
    return (
      <Card className={`${cardGradientClass} ${state === 'error' ? 'border-destructive' : ''}`}>
        <div className="p-6">
          {state === 'error' && (
            <p className="text-destructive mb-4 text-center">{error}</p>
          )}
          <div className="mb-6">
            <label htmlFor="context-input" className="text-sm font-medium mb-2 block">
              {t('explore.customCard.inputLabel')}
            </label>
            <textarea
              id="context-input"
              rows={4}
              placeholder={t('explore.customCard.placeholder')}
              value={context}
              onChange={(e: React.ChangeEvent<HTMLTextAreaElement>) => setContext(e.target.value)}
              className="w-full p-4 border-2 border-border rounded-lg resize-none focus:border-primary focus:outline-none bg-background"
            />
          </div>
          
          <Button 
            onClick={handleGenerate} 
            disabled={!context.trim()}
            size="lg"
            className="w-full"
          >
            {t('common.generate')}
          </Button>
        </div>
      </Card>
    );
  }

  if (state === 'generating') {
    return (
      <Card className={`p-8 text-center border-muted ${cardGradientClass}`}>
        <div className="mb-4">
          <Loader2 className="w-12 h-12 mx-auto text-primary animate-spin" />
        </div>
        <p className="text-muted-foreground">{t('explore.customCard.analyzing')}</p>
      </Card>
    );
  }

  return (
    <Card className={`${cardGradientClass} p-6`}>
      <div className="mb-6">
        <h2 className="text-xl font-semibold">{t('explore.customCard.sectionTitle')}</h2>
        <p className="text-sm text-muted-foreground mt-1">{t('explore.customCard.contextLabel', { context: contextLabel })}</p>
      </div>

      <Card className="p-4 mb-6">
        <div className="space-y-4">
          {examples.map((example: string, index: number) => (
            <div key={index} className="quote-block">
              {example}
            </div>
          ))}
        </div>
      </Card>

      <div className="border-t pt-6">
        <div className="mb-3">
          <label htmlFor="context-input-2" className="text-sm font-medium block">
            {t('explore.customCard.tryDifferent')}
          </label>
        </div>
        <textarea
          id="context-input-2"
          rows={3}
          placeholder={t('explore.customCard.placeholderAlt')}
          value={context}
          onChange={(e: React.ChangeEvent<HTMLTextAreaElement>) => setContext(e.target.value)}
          className="w-full p-4 border-2 border-border rounded-lg resize-none focus:border-primary focus:outline-none bg-background mb-3"
        />
        <Button
          onClick={handleGenerate}
          disabled={!context.trim()}
          variant="outline"
          className="w-full"
        >
          {t('explore.customCard.generateNew')}
        </Button>
      </div>
    </Card>
  );
};
