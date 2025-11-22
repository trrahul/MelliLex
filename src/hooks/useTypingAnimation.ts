import { useState, useEffect } from 'react';

export interface UseTypingAnimationResult {
  displayedWord: string;
  isTyping: boolean;
}

/**
 * Custom hook for typewriter animation effect.
 * 
 * Animates a word character-by-character with configurable speed.
 * Useful for creating engaging text reveal effects.
 * 
 * @param word - The word to animate (or undefined if not ready)
 * @param speed - Milliseconds per character (default: 50)
 * @returns Object with displayedWord and isTyping state
 * 
 * @example
 * ```tsx
 * const { displayedWord, isTyping } = useTypingAnimation(word, 50);
 * 
 * return (
 *   <h1 className={isTyping ? 'typing' : ''}>
 *     {displayedWord}
 *   </h1>
 * );
 * ```
 */
export function useTypingAnimation(
  word: string | undefined,
  speed: number = 50
): UseTypingAnimationResult {
  const [displayedWord, setDisplayedWord] = useState('');
  const [isTyping, setIsTyping] = useState(false);

  useEffect(() => {
    if (!word) {
      setDisplayedWord('');
      setIsTyping(false);
      return;
    }

    setIsTyping(true);
    setDisplayedWord('');

    let currentIndex = 0;
    const typingInterval = setInterval(() => {
      if (currentIndex <= word.length) {
        setDisplayedWord(word.slice(0, currentIndex));
        currentIndex++;
      } else {
        clearInterval(typingInterval);
        setIsTyping(false);
      }
    }, speed);

    return () => clearInterval(typingInterval);
  }, [word, speed]);

  return { displayedWord, isTyping };
}
