import type { PhraseSection3Related } from '../../types';
import { useTranslation } from 'react-i18next';
import { useStores } from '../../stores/RootStore';
import { Button } from '../ui/button';
import { Shuffle, ThumbsUp, ThumbsDown, Link2 } from 'lucide-react';

interface Props {
  data: PhraseSection3Related;
  onNavigate?: (term: string) => void | Promise<void>;
}

export const PhraseRelatedSection = ({ data, onNavigate }: Props) => {
  const { t } = useTranslation();
  const { searchCoordinator } = useStores();
  const regionLabels: Record<string, string> = {
    universal: t('phrase.regions.universal'),
    american: t('phrase.regions.american'),
    british: t('phrase.regions.british'),
    australian: t('phrase.regions.australian'),
  };

  const handleClick = (term: string) => {
    if (onNavigate) {
      onNavigate(term);
    } else {
      searchCoordinator.search(term, { source: 'related-phrase' });
    }
  };

  return (
    <div className="space-y-6">
      {data.variations.length > 0 && (
        <div>
          <h3 className="flex items-center gap-2 text-[0.9375rem] font-semibold mb-4">
            <Shuffle className="w-4 h-4 text-muted-foreground" />
            {t('phrase.variations')}
          </h3>
          <div className="space-y-0">
            {data.variations.map((variation, index) => (
              <button
                key={`${variation.phrase}-${variation.region ?? 'none'}-${index}`}
                onClick={() => handleClick(variation.phrase)}
                className="flex items-start justify-between gap-4 py-3 w-full text-left hover:bg-muted/50 px-2 -mx-2 rounded transition-colors cursor-pointer border-b border-border last:border-b-0"
              >
                <div className="flex-1">
                  <span className="text-[0.9375rem] font-medium">
                    {variation.phrase}
                  </span>
                  {variation.note && (
                    <span className="ml-2 text-[0.8125rem] text-muted-foreground">
                      — {variation.note}
                    </span>
                  )}
                </div>
                {variation.region && variation.region !== 'universal' && (
                  <span className="text-[0.6875rem] font-semibold bg-muted px-2 py-1 rounded uppercase text-muted-foreground">
                    {regionLabels[variation.region] || variation.region}
                  </span>
                )}
              </button>
            ))}
          </div>
        </div>
      )}

      {data.similarPhrases.length > 0 && (
        <div>
          <h3 className="flex items-center gap-2 text-xs font-bold text-muted-foreground uppercase tracking-widest mb-3">
            <ThumbsUp className="w-3.5 h-3.5" />
            {t('phrase.similarMeaning')}
          </h3>
          <div className="space-y-2">
            {data.similarPhrases.map((related, index) => (
              <button
                key={`${related.phrase}-${index}`}
                onClick={() => handleClick(related.phrase)}
                className="w-full p-3 bg-blue-50/50 dark:bg-blue-900/10 rounded-lg hover:bg-blue-100/70 dark:hover:bg-blue-900/20 border border-blue-200/50 dark:border-blue-800/30 hover:border-blue-300 dark:hover:border-blue-700 transition-all text-left cursor-pointer"
              >
                <div className="text-[0.9375rem] font-medium mb-1 text-blue-900 dark:text-blue-100">
                  {related.phrase}
                </div>
                <div className="text-[0.8125rem] text-blue-700/70 dark:text-blue-300/70">
                  {related.meaningHint}
                </div>
              </button>
            ))}
          </div>
        </div>
      )}

      {data.oppositePhrases.length > 0 && (
        <div>
          <h3 className="flex items-center gap-2 text-xs font-bold text-muted-foreground uppercase tracking-widest mb-3">
            <ThumbsDown className="w-3.5 h-3.5" />
            {t('phrase.oppositeMeaning')}
          </h3>
          <div className="space-y-2">
            {data.oppositePhrases.map((related, index) => (
              <button
                key={`${related.phrase}-${index}`}
                onClick={() => handleClick(related.phrase)}
                className="w-full p-3 bg-purple-50/50 dark:bg-purple-900/10 rounded-lg hover:bg-purple-100/70 dark:hover:bg-purple-900/20 border border-purple-200/50 dark:border-purple-800/30 hover:border-purple-300 dark:hover:border-purple-700 transition-all text-left cursor-pointer"
              >
                <div className="text-[0.9375rem] font-medium mb-1 text-purple-900 dark:text-purple-100">
                  {related.phrase}
                </div>
                <div className="text-[0.8125rem] text-purple-700/70 dark:text-purple-300/70">
                  {related.meaningHint}
                </div>
              </button>
            ))}
          </div>
        </div>
      )}

      {data.seeAlso.length > 0 && (
        <div>
          <h3 className="flex items-center gap-2 text-xs font-bold text-muted-foreground uppercase tracking-widest mb-3">
            <Link2 className="w-3.5 h-3.5" />
            {t('phrase.seeAlso')}
          </h3>
          <div className="flex flex-wrap gap-1.5">
            {data.seeAlso.map((phrase, index) => (
              <Button
                key={`${phrase}-${index}`}
                variant="outline"
                size="sm"
                onClick={() => handleClick(phrase)}
                className="h-7 px-2.5 text-xs hover:bg-accent"
              >
                {phrase}
              </Button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};
