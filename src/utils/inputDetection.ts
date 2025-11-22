import type { InputType } from '../types';

export function detectInputType(input: string): InputType {
  const normalized = normalizeInput(input);
  const wordCount = countWords(normalized);
  
  return wordCount > 1 ? 'phrase' : 'word';
}

function normalizeInput(input: string): string {
  return input.trim().split(/\s+/).join(' ');
}

function countWords(normalized: string): number {
  if (!normalized) return 0;
  return normalized.split(/\s+/).length;
}

export function normalizePhrase(input: string): string {
  return input.trim().split(/\s+/).join(' ').toLowerCase();
}

/**
 * Sanitize text for display by trimming whitespace and removing
 * leading/trailing non-alphanumeric characters.
 * Used by both word and phrase stores for consistent display text.
 */
export function sanitizeDisplayText(text: string | undefined): string {
  const trimmed = (text ?? '').trim();
  return trimmed
    .replace(/^[^a-zA-Z0-9]+/, '')
    .replace(/[^a-zA-Z0-9]+$/, '');
}
