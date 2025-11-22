import React from 'react';
import { api } from '../services/api';

const variationsCache = new Map<string, string[]>();
const inflightRequests = new Map<string, Promise<string[]>>();

const normalizeWord = (word: string) => word.trim().toLowerCase();

export const __testing__ = {
  resetCaches() {
    variationsCache.clear();
    inflightRequests.clear();
  },
};

export const getWordVariationsWithCache = async (word: string): Promise<string[]> => {
  const normalizedWord = normalizeWord(word);
  if (!normalizedWord) {
    return [];
  }

  if (variationsCache.has(normalizedWord)) {
    return variationsCache.get(normalizedWord)!;
  }

  if (inflightRequests.has(normalizedWord)) {
    return inflightRequests.get(normalizedWord)!;
  }

  const request = (async () => {
    try {
      const fetchedVariations = await api.getWordVariations(word);
      variationsCache.set(normalizedWord, fetchedVariations);
      return fetchedVariations;
    } catch (error) {
      console.error('Failed to get word variations:', error);
      const fallbackWord = word.trim() || word;
      const fallback = [fallbackWord];
      variationsCache.set(normalizedWord, fallback);
      return fallback;
    } finally {
      inflightRequests.delete(normalizedWord);
    }
  })();

  inflightRequests.set(normalizedWord, request);
  return request;
};

const useWordVariations = (word: string) => {
  const normalizedWord = normalizeWord(word);
  const [variations, setVariations] = React.useState<string[] | null>(
    normalizedWord ? variationsCache.get(normalizedWord) || null : null
  );

  React.useEffect(() => {
    let isMounted = true;

    const fetchVariations = async () => {
      if (!normalizedWord) {
        setVariations(null);
        return;
      }

      const fetched = await getWordVariationsWithCache(word);
      if (isMounted) {
        setVariations(fetched);
      }
    };

    fetchVariations();

    return () => {
      isMounted = false;
    };
  }, [normalizedWord, word]);

  return variations;
};

export const HighlightedText: React.FC<{ text: string; word?: string }> = ({ text, word }) => {
  const variations = useWordVariations(word ?? '');

  if (!word || !variations) {
    return <>{text}</>;
  }

  const escapedVariations = variations.map(v => 
    v.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  );
  
  const pattern = `\\b(?:${escapedVariations.join('|')})\\b`;
  const regex = new RegExp(pattern, 'gi');
  
  const parts = text.split(regex);
  const matches = text.match(regex);

  if (!matches) return <>{text}</>;

  return (
    <>
      {parts.reduce<React.ReactNode[]>((acc, part, i) => {
        acc.push(part);
        if (i < matches.length) {
          acc.push(
            <span
              key={i}
              className="font-bold text-foreground [background:linear-gradient(180deg,transparent_60%,#fde68a_60%)] dark:[background:linear-gradient(180deg,transparent_60%,rgb(202_138_4/0.4)_60%)]"
            >
              {matches[i]}
            </span>
          );
        }
        return acc;
      }, [])}
    </>
  );
};
