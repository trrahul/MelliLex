import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { ExploreStore } from '../ExploreStore';
import type { ILogger } from '../../services/LoggerService';
import type { PracticeExercise } from '../../types';
import { FormalityLevel, WordFrequency } from '../../types';
import { MockWordSource } from './helpers/MockWordSource';

const apiMocks = vi.hoisted(() => ({
  generateFormalityAnalysis: vi.fn(),
  generateDomainExploration: vi.fn(),
  generateUsagePatterns: vi.fn(),
  generatePracticeExercisesOnly: vi.fn(),
  generateContextualExamples: vi.fn(),
  generateCommonMistakes: vi.fn(),
  getCachedExplorationFeatures: vi.fn(),
}));

vi.mock('../../services/api', () => ({
  api: apiMocks,
}));

class TestLogger implements ILogger {
  info = vi.fn();
  error = vi.fn();
  debug = vi.fn();
}

describe('ExploreStore', () => {
  let store: ExploreStore;
  let logger: TestLogger;
  let wordSource: MockWordSource;

  beforeEach(() => {
    vi.useFakeTimers();
    vi.clearAllMocks();
    logger = new TestLogger();
    wordSource = new MockWordSource();
    store = new ExploreStore(logger, wordSource);
    apiMocks.getCachedExplorationFeatures.mockResolvedValue({});
    wordSource.setWord('spark');
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('resets feature state when current word changes', () => {
    store.formalityState = 'generated';
    store.formalityPercentage = 55;
    store.formalityAlternatives = [{ word: 'ember', level: FormalityLevel.Informal, context: 'poetry', explanation: 'desc' }];
    store.domainsState = 'error';
    store.domainsError = 'boom';
    store.practiceExercises = [{
      question: 'old',
      exerciseType: 'multiple-choice',
      options: ['a'],
      correctAnswer: 'a',
      explanation: 'x',
      isAnswered: false,
      userAnswer: '',
    }];

    wordSource.setWord('ember');

    expect(store.currentWord).toBe('ember');
    expect(store.formalityState).toBe('ungenerated');
    expect(store.formalityPercentage).toBeNull();
    expect(store.formalityAlternatives).toHaveLength(0);
    expect(store.domainsState).toBe('ungenerated');
    expect(store.domainsError).toBeNull();
    expect(store.practiceExercises).toHaveLength(0);
  });

  it('resets when progressive store restarts search for the same word', () => {
    store.formalityState = 'generated';
    store.domainsState = 'generated';
    wordSource.finishSearch();

    wordSource.startSearch('spark');

    expect(store.formalityState).toBe('ungenerated');
    expect(store.domainsState).toBe('ungenerated');
  });

  it('generates formality analysis and stores results', async () => {
    apiMocks.generateFormalityAnalysis.mockResolvedValue({
      formalityPercentage: 68,
      formalityAlternatives: [
        { word: 'glow', level: FormalityLevel.Informal, context: 'poetry', explanation: 'Softer tone' },
      ],
    });

    await store.generateFormality();

    expect(apiMocks.generateFormalityAnalysis).toHaveBeenCalledWith('spark');
    expect(store.formalityState).toBe('generated');
    expect(store.formalityPercentage).toBe(68);
    expect(store.formalityAlternatives).toHaveLength(1);
    expect(store.formalityError).toBeNull();
  });

  it('does not attempt feature generation without a current word', async () => {
    const freshWordSource = new MockWordSource();
    const freshStore = new ExploreStore(logger, freshWordSource);

    await freshStore.generateFormality();

    expect(apiMocks.generateFormalityAnalysis).not.toHaveBeenCalled();
    expect(freshStore.formalityState).toBe('ungenerated');
  });

  it('marks feature as error when API call fails', async () => {
    apiMocks.generateUsagePatterns.mockRejectedValue(new Error('network down'));

    await store.generateUsage();

    expect(apiMocks.generateUsagePatterns).toHaveBeenCalledWith('spark');
    expect(store.usageState).toBe('error');
    expect(store.usageError).toContain('network down');
  });

  it('prevents concurrent generation of the same feature', async () => {
    let resolveFormality: (value: any) => void = () => {};
    const pending = new Promise(resolve => {
      resolveFormality = resolve;
    });
    apiMocks.generateFormalityAnalysis.mockReturnValue(pending);

    const firstCall = store.generateFormality();
    const secondCall = store.generateFormality();

    expect(store.formalityState).toBe('generating');
    expect(apiMocks.generateFormalityAnalysis).toHaveBeenCalledTimes(1);

    resolveFormality({
      formalityPercentage: 50,
      formalityAlternatives: [],
    });

    await firstCall;
    await secondCall;

    expect(store.formalityState).toBe('generated');
    expect(apiMocks.generateFormalityAnalysis).toHaveBeenCalledTimes(1);
  });

  it('generates domain exploration data', async () => {
    apiMocks.generateDomainExploration.mockResolvedValue([
      {
        domain: 'engineering',
        usageFrequency: WordFrequency.Common,
        commonCollocations: ['spark plug'],
        examples: ['Spark timing'],
        isExpanded: false,
      },
    ]);

    await store.generateDomains();

    expect(apiMocks.generateDomainExploration).toHaveBeenCalledWith('spark');
    expect(store.domainsState).toBe('generated');
    expect(store.domainExplorations).toHaveLength(1);
    expect(store.domainsError).toBeNull();
  });

  it('generates usage patterns', async () => {
    apiMocks.generateUsagePatterns.mockResolvedValue([
      {
        template: '{word} debate',
        patternType: 'collocation',
        description: 'Use before noun',
        examples: ['spark debate'],
      },
    ]);

    await store.generateUsage();

    expect(store.usageState).toBe('generated');
    expect(store.usagePatterns[0].template).toBe('{word} debate');
  });

  it('generates practice exercises and related words', async () => {
    apiMocks.generatePracticeExercisesOnly.mockResolvedValue({
      practiceExercises: [
        {
          question: 'Choose the correct phrase',
          exerciseType: 'multiple-choice',
          options: ['spark change', 'sparkly change'],
          correctAnswer: 'spark change',
          explanation: 'spark + noun',
          isAnswered: false,
          userAnswer: '',
        },
      ],
    });

    await store.generatePractice();

    expect(store.practiceState).toBe('generated');
    expect(store.practiceExercises).toHaveLength(1);
  });

  it('generates custom contextual examples when context provided', async () => {
    apiMocks.generateContextualExamples.mockResolvedValue(['Spark innovation in science']);

    await store.generateCustomExamples('  science lab ');

    expect(apiMocks.generateContextualExamples).toHaveBeenCalledWith('spark', 'science lab');
    expect(store.customContextState).toBe('generated');
    expect(store.customContext).toBe('science lab');
    expect(store.customExamples).toEqual(['Spark innovation in science']);
  });

  it('skips custom examples when context is empty', async () => {
    await store.generateCustomExamples('   ');

    expect(apiMocks.generateContextualExamples).not.toHaveBeenCalled();
    expect(store.customContextState).toBe('ungenerated');
  });

  it('reports generation progress via computed helpers', async () => {
    expect(store.hasAnyGenerated).toBe(false);
    expect(store.allFeaturesGenerated).toBe(false);

    apiMocks.generateFormalityAnalysis.mockResolvedValue({ formalityPercentage: 50, formalityAlternatives: [] });
    apiMocks.generateDomainExploration.mockResolvedValue([]);
    apiMocks.generateUsagePatterns.mockResolvedValue([]);
    apiMocks.generatePracticeExercisesOnly.mockResolvedValue({ practiceExercises: [] });
    apiMocks.generateCommonMistakes.mockResolvedValue([]);

    await store.generateFormality();
    await store.generateDomains();
    await store.generateUsage();
    await store.generatePractice();
    await store.generateMistakes();

    expect(store.hasAnyGenerated).toBe(true);
    expect(store.allFeaturesGenerated).toBe(true);
  });
});
