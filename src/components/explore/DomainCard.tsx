import { useTranslation } from 'react-i18next';
import { Card } from '../ui/card';
import { Button } from '../ui/button';
import { Briefcase, Loader2, Building2, Code2, GraduationCap, Coffee } from 'lucide-react';
import type { DomainExploration } from '../../types';
import { HighlightedText } from '../../utils/textHighlight';
import { Badge } from '../ui/badge';
import type { FeatureState } from '../../stores/ExploreStore';

interface DomainCardProps {
  currentWord: string;
  state: FeatureState;
  domains: DomainExploration[];
  error: string | null;
  onGenerate: () => Promise<void> | void;
}

export const DomainCard = ({
  currentWord,
  state,
  domains,
  error,
  onGenerate,
}: DomainCardProps) => {
  const { t } = useTranslation();

  const cardGradientClass = 'define-card define-card--accent-amber';

  const getDomainIcon = (domainName: string) => {
    const lower = domainName.toLowerCase();
    if (lower.includes('business') || lower.includes('corporate')) return Building2;
    if (lower.includes('technical') || lower.includes('technology')) return Code2;
    if (lower.includes('academic') || lower.includes('scientific')) return GraduationCap;
    if (lower.includes('casual') || lower.includes('informal')) return Coffee;
    return Briefcase;
  };

  if (state === 'ungenerated') {
    return (
      <Card className={`p-8 text-center ${cardGradientClass}`}>
        <div className="mb-4">
          <Briefcase className="w-16 h-16 mx-auto text-muted-foreground" />
        </div>
        <h3 className="text-lg font-medium mb-2">{t('explore.domainsCard.title')}</h3>
        <p className="text-muted-foreground mb-6">
          {t('explore.domainsCard.description', { word: currentWord })}
        </p>
        <Button onClick={() => onGenerate()} size="lg">
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
        <p className="text-muted-foreground">{t('explore.domainsCard.analyzing')}</p>
      </Card>
    );
  }

  if (state === 'error') {
    return (
      <Card className={`p-8 text-center border-destructive ${cardGradientClass}`}>
        <p className="text-destructive mb-4">{error}</p>
        <Button variant="outline" onClick={() => onGenerate()}>
          {t('common.tryAgain')}
        </Button>
      </Card>
    );
  }

  return (
    <Card className={`p-6 ${cardGradientClass}`}>
      <div className="mb-6">
        <h2 className="text-xl font-semibold">{t('explore.domainsCard.sectionTitle')}</h2>
      </div>

      <div className="space-y-6">
        {domains.map((domain: DomainExploration, index: number) => {
          const DomainIcon = getDomainIcon(domain.domain);
          return (
            <div key={index} className="pb-6 border-b border-border last:border-0 last:pb-0">
              <div className="flex items-center gap-3 mb-3">
                <DomainIcon className="w-5 h-5 text-muted-foreground" />
                <h3 className="text-lg font-semibold">{domain.domain}</h3>
                <Badge variant="secondary" className="text-xs">
                  {domain.usageFrequency}
                </Badge>
              </div>

              <div className="space-y-4">
                <div>
                  <div className="text-sm font-medium text-muted-foreground mb-2">
                    {t('explore.domainsCard.collocations')}
                  </div>
                  <div className="flex flex-wrap gap-2">
                    {domain.commonCollocations.map((collocation: string, i: number) => (
                      <span key={i} className="text-xs px-3 py-1.5 bg-muted rounded-full">
                        {collocation}
                      </span>
                    ))}
                  </div>
                </div>

                <div>
                  <div className="text-sm font-medium text-muted-foreground mb-2">
                    {t('explore.domainsCard.examples')}
                  </div>
                  <div className="space-y-3">
                    {domain.examples.map((example: string, i: number) => (
                      <div key={i} className="example-text pl-4 border-l-2 border-border text-base text-foreground leading-relaxed">
                        <HighlightedText text={example} word={currentWord} />
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </Card>
  );
};
