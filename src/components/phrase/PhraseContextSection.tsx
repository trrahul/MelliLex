import type { PhraseSection2Context } from '../../types';
import { useTranslation } from 'react-i18next';
import {
  AlertCircle,
  BookMarked,
  Calendar,
  CheckCircle,
  FileText,
  MessageSquare,
  XCircle,
} from 'lucide-react';
import { HighlightedText } from '../../utils/textHighlight';

interface Props {
  data: PhraseSection2Context;
  phrase: string;
}

export const PhraseContextSection = ({ data, phrase }: Props) => {
  const { t } = useTranslation();
  
  return (
    <div className="space-y-6">
      <div className="bg-blue-50 dark:bg-blue-900/20 border-l-4 border-blue-500 dark:border-blue-600 p-4 rounded">
        <div className="flex items-center gap-2 mb-2">
          <BookMarked className="w-4 h-4 text-blue-600 dark:text-blue-400" />
          <span className="text-xs font-bold text-blue-900 dark:text-blue-200 uppercase tracking-wider">
            {t('phrase.originStory')}
          </span>
        </div>
        
        {(data.origin.era || data.origin.source) && (
          <div className="flex flex-wrap gap-2 mb-3">
            {data.origin.era && (
              <span className="inline-flex items-center gap-1.5 text-xs bg-blue-100 dark:bg-blue-900/50 text-blue-900 dark:text-blue-100 px-2 py-1 rounded">
                <Calendar className="w-3 h-3 shrink-0" />
                <span>{data.origin.era}</span>
              </span>
            )}
            {data.origin.source && (
              <span className="inline-flex items-center gap-1.5 text-xs bg-blue-100 dark:bg-blue-900/50 text-blue-900 dark:text-blue-100 px-2 py-1 rounded">
                <FileText className="w-3 h-3 shrink-0" />
                <span>{data.origin.source}</span>
              </span>
            )}
          </div>
        )}
        
        <p className="definition-content text-sm text-blue-900 dark:text-blue-100 leading-relaxed">
          {data.origin.story}
        </p>
        
        {data.origin.evolution && (
          <p className="definition-content mt-3 pt-3 border-t border-blue-300 dark:border-blue-700 text-sm text-blue-900 dark:text-blue-200">
            {data.origin.evolution}
          </p>
        )}
      </div>

      {data.usageNotes.length > 0 && (
        <div>
          <h3 className="flex items-center gap-2 text-[0.9375rem] font-semibold mb-4">
            <MessageSquare className="w-4 h-4 text-muted-foreground" />
            {t('phrase.usageNotes')}
          </h3>
          <div className="space-y-4">
            {data.usageNotes.map((note, index) => (
              <div key={index}>
                <div className="flex items-center gap-2 mb-2">
                  <span className="text-xs font-semibold text-muted-foreground uppercase tracking-wide">
                    {note.context}
                  </span>
                  {note.tone && (
                    <span className="text-[0.6875rem] bg-muted px-2 py-0.5 rounded-full capitalize">
                      {note.tone}
                    </span>
                  )}
                </div>
                <div className="example-text pl-4 border-l-2 border-border text-base text-foreground leading-relaxed">
                  <HighlightedText text={note.example} word={phrase} />
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {data.commonMistakes.length > 0 && (
        <div>
          <h3 className="text-[0.9375rem] font-semibold mb-4 flex items-center gap-2">
            <AlertCircle className="w-4 h-4 text-red-500" />
            {t('phrase.commonMistakes')}
          </h3>
          <div className="space-y-6">
            {data.commonMistakes.map((mistake, index) => (
              <div key={index} className="pb-6 border-b border-border last:border-b-0 last:pb-0">
                <div className="mb-3">
                  <span className="text-xs px-2 py-1 bg-muted rounded text-muted-foreground">
                    {mistake.mistakeType}
                  </span>
                </div>

                <div className="mb-3">
                  <span className="text-sm font-medium text-muted-foreground flex items-center gap-1 mb-2">
                    <XCircle className="h-4 w-4 text-red-500" />
                    <span>{t('phrase.incorrect')}</span>
                  </span>
                  <div className="example-text pl-4 border-l-2 border-red-300 text-base leading-relaxed text-muted-foreground">
                    <HighlightedText text={mistake.incorrect} word={phrase} />
                  </div>
                </div>

                <div className="bg-blue-50 dark:bg-blue-900/20 border-l-4 border-blue-500 dark:border-blue-600 p-4 rounded mb-3">
                  <div className="text-xs text-blue-900 dark:text-blue-200 font-bold mb-2 uppercase tracking-wider flex items-center gap-1">
                    <CheckCircle className="h-3.5 w-3.5" />
                    <span>{t('phrase.correctUsage')}</span>
          
                  </div>
                  <div className="definition-content text-sm text-blue-900 dark:text-blue-100 leading-relaxed">
                    <HighlightedText text={mistake.correct} word={phrase} />
                  </div>
                </div>

                <p className="text-sm text-muted-foreground leading-relaxed">
                  {mistake.explanation}
                </p>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};
