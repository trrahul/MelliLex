/**
 * Event names used for progressive loading.
 * Centralized to avoid typos and enable refactoring.
 */
export const EVENTS = {
  // Word Definition Progressive Events (3-section)
  WORD_SECTION_1_HEADER: 'word-section-1-header',
  WORD_SECTION_2_MEANINGS: 'word-section-2-meanings',
  WORD_SECTION_3_RELATED: 'word-section-3-related',

  // Phrase Definition Progressive Events (3-section)
  PHRASE_SECTION_1_OVERVIEW: 'phrase-section-1-overview',
  PHRASE_SECTION_2_CONTEXT: 'phrase-section-2-context',
  PHRASE_SECTION_3_RELATED: 'phrase-section-3-related',

  // Word Exploration Progressive Events
  EXPLORATION_FORMALITY: 'exploration-formality',
  EXPLORATION_USAGE: 'exploration-usage',
  EXPLORATION_DOMAINS: 'exploration-domains',
  EXPLORATION_PRACTICE: 'exploration-practice',
} as const;

/**
 * Timeout configurations for various operations.
 */
export const TIMEOUTS = {
  WORD_SEARCH: 30000, // 30 seconds
  EXPLORATION: 30000, // 30 seconds
} as const;

/**
 * UI configuration constants.
 */
export const UI_CONFIG = {
  MAX_HISTORY_SIZE: 100,
} as const;
