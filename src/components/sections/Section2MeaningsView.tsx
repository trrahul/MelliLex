import type { WordSection2Meanings } from '../../types';
import { HighlightedText } from '../../utils/textHighlight';
import { useTranslation } from 'react-i18next';

interface Props {
  data: WordSection2Meanings;
  word: string;
}

export const Section2MeaningsView = ({ data, word }: Props) => {
  const { t } = useTranslation();
  
  return (
    <div className="space-y-6">
      <h2 className="text-xl font-semibold mb-4">{t('sections.meanings')}</h2>
      {data.meanings.map((meaning, idx) => (
        <div
          key={meaning.number}
          className={`pb-6 ${idx !== data.meanings.length - 1 ? 'border-b border-border' : ''}`}
        >
          <div className="flex items-baseline gap-3 mb-3">
            <span className="text-[0.875rem] font-semibold text-primary">
              {meaning.number}.
            </span>
            <span className="text-xs text-muted-foreground italic">
              {meaning.partOfSpeech}
            </span>
          </div>

          <p className="meaning-text text-[0.9375rem] leading-relaxed mb-3">
            {meaning.definition}
          </p>

          {meaning.memoryTip && (
            <div className="bg-blue-50 dark:bg-blue-900/20 border-l-4 border-blue-500 dark:border-blue-600 p-4 mb-3 rounded">
              <div className="text-xs text-blue-900 dark:text-blue-200 font-bold mb-2 uppercase tracking-wider">
                {t('sections.memoryTip')}
              </div>
              <div className="definition-content text-sm text-blue-900 dark:text-blue-100 leading-relaxed">
                {meaning.memoryTip}
              </div>
            </div>
          )}

          {meaning.examples.length > 0 && (
            <div className="space-y-4">
              {meaning.examples.map((example, i) => (
                <div
                  key={i}
                  className="example-text pl-4 border-l-2 border-border text-base text-foreground leading-relaxed"
                >
                  <HighlightedText text={example} word={word} />
                </div>
              ))}
            </div>
          )}
        </div>
      ))}
    </div>
  );
};
