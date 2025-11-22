import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Card } from '../ui/card';
import { Button } from '../ui/button';
import { ClipboardCheck, Loader2 } from 'lucide-react';
import type { PracticeExercise } from '../../types';
import type { FeatureState } from '../../stores/ExploreStore';

interface PracticeCardProps {
  currentWord: string;
  state: FeatureState;
  exercises: PracticeExercise[];
  error: string | null;
  onGenerate: () => Promise<void> | void;
}

export const PracticeCard = ({
  currentWord,
  state,
  exercises,
  error,
  onGenerate,
}: PracticeCardProps) => {
  const { t } = useTranslation();

  const [answers, setAnswers] = useState<Record<number, string>>({});
  const [showResults, setShowResults] = useState(false);
  const cardGradientClass = 'define-card';

  const handleAnswerSelect = (exerciseIndex: number, answer: string) => {
    setAnswers({ ...answers, [exerciseIndex]: answer });
  };

  const handleCheckAnswers = () => {
    setShowResults(true);
  };

  useEffect(() => {
    setAnswers({});
    setShowResults(false);
  }, [exercises]);

  if (state === 'ungenerated') {
    return (
      <Card className={`p-8 text-center ${cardGradientClass}`}>
        <div className="mb-4">
          <ClipboardCheck className="w-16 h-16 mx-auto text-muted-foreground" />
        </div>
        <h3 className="text-lg font-medium mb-2">{t('explore.practiceCard.title')}</h3>
        <p className="text-muted-foreground mb-6">
          {t('explore.practiceCard.description', { word: currentWord })}
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
        <p className="text-muted-foreground">{t('explore.practiceCard.analyzing')}</p>
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
        <h2 className="text-xl font-semibold">{t('explore.practiceCard.sectionTitle')}</h2>
        <Button variant="ghost" size="sm" onClick={onGenerate}>
          {t('common.regenerate')}
        </Button>
      </div>

      <div className="space-y-6">
        <div className="space-y-4">
          {exercises.map((exercise: PracticeExercise, index: number) => {
            const userAnswer = answers[index];
            const isCorrect = showResults && userAnswer === exercise.correctAnswer;
            const isIncorrect = showResults && userAnswer && userAnswer !== exercise.correctAnswer;

            return (
              <Card
                key={`${exercise.question}-${index}`}
                className={`p-4 ${
                  isCorrect
                    ? 'border-green-500 bg-green-50 dark:bg-green-900/10'
                    : isIncorrect
                    ? 'border-red-500 bg-red-50 dark:bg-red-900/10'
                    : ''
                }`}
              >
                <div className="mb-3">
                  <p className="definition-content font-medium">
                    {index + 1}. {exercise.question}
                  </p>
                </div>

                <div className="space-y-2 mb-3">
                  {exercise.options.map((option, optIndex) => {
                    const isSelected = userAnswer === option;
                    const isCorrectOption = option === exercise.correctAnswer;

                    return (
                      <button
                        key={`${exercise.question}-${option}-${optIndex}`}
                        onClick={() => !showResults && handleAnswerSelect(index, option)}
                        disabled={showResults}
                        className={`definition-content w-full text-left p-3 rounded border-2 transition-colors ${
                          showResults && isCorrectOption
                            ? 'border-green-500 bg-green-50 dark:bg-green-900/20'
                            : isSelected && !showResults
                            ? 'border-primary bg-primary/10'
                            : isSelected && showResults && !isCorrectOption
                            ? 'border-red-500 bg-red-50 dark:bg-red-900/20'
                            : 'border-border hover:border-primary'
                        } ${showResults ? 'cursor-default' : 'cursor-pointer'}`}
                      >
                        {option}
                      </button>
                    );
                  })}
                </div>

                {showResults && (
                  <div className="mt-3 p-3 bg-muted rounded text-sm">
                    <p className="font-medium mb-1">{t('explore.practiceCard.explanation')}</p>
                    <p className="definition-content text-muted-foreground">{exercise.explanation}</p>
                  </div>
                )}
              </Card>
            );
          })}
        </div>

        {!showResults && (
          <div>
            <Button
              onClick={handleCheckAnswers}
              disabled={Object.keys(answers).length === 0}
              className="w-full"
            >
              {t('explore.practiceCard.checkAnswers')}
            </Button>
          </div>
        )}
      </div>
    </Card>
  );
};
