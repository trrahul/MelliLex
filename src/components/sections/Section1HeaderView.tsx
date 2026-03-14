import type { WordSection1Header } from '../../types';
import { Volume2, Eye, Share2, Globe, Layers } from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { toast } from 'sonner';
import { ExportWordDialog } from '../ExportWordDialog';
import { useTranslation } from 'react-i18next';

interface Props {
  data: WordSection1Header;
}

export const Section1HeaderView = ({ data }: Props) => {
  const navigate = useNavigate();
  const { t } = useTranslation();

  const getOriginDisplay = (origin: string) => {
    const firstWord = origin.split(' ')[0];
    return { display: firstWord, full: origin };
  };

  const originInfo = getOriginDisplay(data.origin);

  const handlePronounce = () => {
    if ('speechSynthesis' in window) {
      const utterance = new SpeechSynthesisUtterance(data.word);
      window.speechSynthesis.speak(utterance);
    }
  };

  const handleExplore = () => {
    navigate('/explore');
  };

  const handleShare = async () => {
    const shareData = {
      title: `Word: ${data.word}`,
      text: `${data.word} (${data.pronunciation})\n\n${data.tldr}`,
    };

    if (navigator.share) {
      try {
        await navigator.share(shareData);
      } catch (error) {
        if (!(error instanceof DOMException && error.name === 'AbortError')) {
          toast.error(t('wordHeader.failedToCopy'));
        }
      }
    } else {
      try {
        await navigator.clipboard.writeText(shareData.text);
        toast.success(t('wordHeader.copiedToClipboard'));
      } catch (error) {
        toast.error(t('wordHeader.failedToCopy'));
      }
    }
  };

  return (
    <div className="space-y-6">
      <h1 className="text-5xl font-bold tracking-tight leading-none">
        {data.word}
      </h1>

      <div className="flex items-center gap-3 pb-6 border-b border-border">
        <button
          onClick={handlePronounce}
          className="p-2 hover:bg-accent rounded-lg transition-colors border border-border hover:border-foreground"
          aria-label={t('wordHeader.pronounceAriaLabel')}
        >
          <Volume2 className="w-4 h-4" />
        </button>
        <span className="text-base font-mono text-foreground font-medium">
          {data.pronunciation}
        </span>
        <span className="text-sm text-muted-foreground">·</span>
        <span className="text-sm text-muted-foreground">{data.syllables}</span>
      </div>

      <div className="grid grid-cols-2 gap-6">
        <div className="flex items-center gap-3">
          <div className="p-2 bg-muted rounded-lg flex-shrink-0">
            <Globe className="w-4 h-4 text-muted-foreground" />
          </div>
          <div className="flex flex-col gap-0.5 min-w-0">
            <div className="text-xs font-semibold text-muted-foreground uppercase tracking-wide">
              {t('wordHeader.origin')}
            </div>
            <div 
              className="text-base font-medium cursor-help truncate" 
              title={originInfo.full}
            >
              {originInfo.display}
            </div>
          </div>
        </div>

        <div className="flex items-center gap-3">
          <div className="p-2 bg-muted rounded-lg flex-shrink-0">
            <Layers className="w-4 h-4 text-muted-foreground" />
          </div>
          <div className="flex flex-col gap-0.5 min-w-0">
            <div className="text-xs font-semibold text-muted-foreground uppercase tracking-wide">
              {t('wordHeader.formality')}
            </div>
            <div className="text-base font-medium truncate">
              {data.formality.level} · {data.formality.percentage}%
            </div>
          </div>
        </div>
      </div>

      {data.domains.length > 0 && (
        <div className="flex items-center gap-3 flex-wrap">
          <span className="text-xs font-semibold text-muted-foreground uppercase tracking-wide">
            {t('wordHeader.usedIn')}
          </span>
          {data.domains.map((domain, i) => (
            <span
              key={i}
              className="px-3 py-1.5 bg-muted text-sm font-medium rounded-full border border-border hover:border-muted-foreground/50 transition-colors"
            >
              {domain}
            </span>
          ))}
        </div>
      )}

      <div className="bg-blue-50 dark:bg-blue-900/20 border-l-4 border-blue-500 dark:border-blue-600 p-4 rounded">
        <div className="text-xs font-bold text-blue-900 dark:text-blue-200 uppercase tracking-wider mb-2">
          {t('sections.tldr')}
        </div>
        <div className="definition-content text-base text-blue-900 dark:text-blue-100 leading-relaxed">
          {data.tldr}
        </div>
      </div>

      <div className="flex gap-2 flex-wrap pt-2">
        <button 
          onClick={handleExplore}
          className="flex items-center gap-2 px-4 py-2 rounded-lg bg-primary text-primary-foreground hover:opacity-90 transition-all text-sm font-medium"
        >
          <Eye className="w-4 h-4" />
          {t('common.explore')}
        </button>
        <button 
          onClick={handleShare}
          className="flex items-center gap-2 px-4 py-2 rounded-lg border border-border bg-card hover:bg-accent hover:border-foreground transition-all text-sm font-medium"
        >
          <Share2 className="w-4 h-4" />
          {t('common.share')}
        </button>
        <ExportWordDialog />
      </div>
    </div>
  );
};
