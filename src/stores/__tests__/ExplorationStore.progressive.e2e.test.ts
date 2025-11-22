import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { ExploreStore } from '../ExploreStore';
import type { ILogger } from '../../services/LoggerService';
import { TIMEOUTS } from '../../constants/events';
import { WordFrequency } from '../../types';
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

describe('ExploreStore feature runner (integration)', () => {
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

  it('generates all primary features sequentially', async () => {
    apiMocks.generateFormalityAnalysis.mockResolvedValue({ formalityPercentage: 40, formalityAlternatives: [] });
    apiMocks.generateDomainExploration.mockResolvedValue([
      { domain: 'science', usageFrequency: WordFrequency.Uncommon, commonCollocations: ['spark curiosity'], examples: ['spark curiosity in class'], isExpanded: false },
    ]);
    apiMocks.generateUsagePatterns.mockResolvedValue([
      { template: 'spark + noun', patternType: 'collocation', description: 'initiate something', examples: ['spark debate'] },
    ]);
    apiMocks.generatePracticeExercisesOnly.mockResolvedValue({
      practiceExercises: [
        {
          question: 'Pick the correct collocation',
          exerciseType: 'multiple-choice',
          options: ['spark change', 'sparkly change'],
          correctAnswer: 'spark change',
          explanation: 'spark + noun',
          isAnswered: false,
          userAnswer: '',
        },
      ],
    });
    apiMocks.generateCommonMistakes.mockResolvedValue([]);

    await store.generateFormality();
    await store.generateDomains();
    await store.generateUsage();
    await store.generatePractice();
    await store.generateMistakes();

    expect(store.formalityState).toBe('generated');
    expect(store.domainExplorations).toHaveLength(1);
    expect(store.usagePatterns[0].template).toBe('spark + noun');
    expect(store.practiceExercises).toHaveLength(1);
    expect(store.allFeaturesGenerated).toBe(true);
  });

  it('marks feature as timed out when API does not resolve', async () => {
    let resolveDomains: (() => void) | null = null;
    const pendingDomains = new Promise<any>((resolve) => {
      resolveDomains = () => resolve([]);
    });
    apiMocks.generateDomainExploration.mockReturnValue(pendingDomains);

    const generationPromise = store.generateDomains();

    expect(store.domainsState).toBe('generating');

    vi.advanceTimersByTime(TIMEOUTS.EXPLORATION + 50);
    await Promise.resolve();

    expect(store.domainsState).toBe('error');
    expect(store.domainsError).toContain('timed out');

    resolveDomains?.();
    await generationPromise;
  });
});
