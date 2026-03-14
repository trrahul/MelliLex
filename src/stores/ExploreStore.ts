import { makeObservable, observable, action, runInAction, computed, reaction, type IReactionDisposer } from 'mobx';
import { api } from '../services/api';
import type {
  FormalityAlternative,
  DomainExploration,
  UsagePattern,
  PracticeExercise,
  MistakeItem,
} from '../types';
import type { ILogger } from '../services/LoggerService';
import { BaseStore } from './BaseStore';
import type { LoadingState } from '../types';
import {
  runFeatureTask,
  type FeatureKey,
  type FeatureRunnerHooks,
  type FeatureState,
  getFeatureBlueprint,
} from './helpers/FeatureRunner';
export type { FeatureState } from './helpers/FeatureRunner';

interface WordSource {
  readonly currentWord: string;
  readonly loadingState: LoadingState;
}

const COMPACT_DOMAIN_LIMIT = 4;

interface FeatureRunnerConfig<T> {
  feature: FeatureKey;
  execute: (word: string) => Promise<T>;
  onSuccess: (result: T) => void;
  customTimeoutMs?: number;
}

export class ExploreStore extends BaseStore {
  private wordSyncDisposer?: IReactionDisposer;
  private searchResetDisposer?: IReactionDisposer;

  // Formality Analysis
  formalityState: FeatureState = 'ungenerated';
  formalityPercentage: number | null = null;
  formalityAlternatives: FormalityAlternative[] = [];
  formalityError: string | null = null;

  // Domain Exploration
  domainsState: FeatureState = 'ungenerated';
  domainExplorations: DomainExploration[] = [];
  domainsError: string | null = null;

  // Usage Patterns
  usageState: FeatureState = 'ungenerated';
  usagePatterns: UsagePattern[] = [];
  usageError: string | null = null;

  // Related Words & Practice
  practiceState: FeatureState = 'ungenerated';
  practiceExercises: PracticeExercise[] = [];
  practiceError: string | null = null;

  // Common Mistakes
  mistakesState: FeatureState = 'ungenerated';
  commonMistakes: MistakeItem[] = [];
  mistakesError: string | null = null;

  // Custom Context Examples
  customContextState: FeatureState = 'ungenerated';
  customContext: string = '';
  customExamples: string[] = [];
  customContextError: string | null = null;

  private featureTimeouts = new Map<FeatureKey, number>();
  private cacheHydrationToken: string | null = null;
  /** @internal MobX-tracked — use `currentWord` getter instead */
  exploreWord: string = '';

  constructor(private logger: ILogger, private wordSource: WordSource) {
    super();
    makeObservable(this, {
      // Observables
      exploreWord: observable,
      formalityState: observable,
      formalityPercentage: observable,
      formalityAlternatives: observable,
      formalityError: observable,
      domainsState: observable,
      domainExplorations: observable,
      domainsError: observable,
      usageState: observable,
      usagePatterns: observable,
      usageError: observable,
      practiceState: observable,
      practiceExercises: observable,
      practiceError: observable,
      mistakesState: observable,
      commonMistakes: observable,
      mistakesError: observable,
      customContextState: observable,
      customContext: observable,
      customExamples: observable,
      customContextError: observable,

      // Actions
      resetAll: action,
      generateFormality: action,
      generateDomains: action,
      generateUsage: action,
      generatePractice: action,
      generateMistakes: action,
      generateCustomExamples: action,

      // Computed
      currentWord: computed,
      hasAnyGenerated: computed,
      allFeaturesGenerated: computed,
    });

    this.setupWordSync();
  }

  get currentWord(): string {
    return this.exploreWord;
  }


  resetAll(): void {
    this.clearAllFeatureTimeouts();

    this.formalityState = 'ungenerated';
    this.formalityPercentage = null;
    this.formalityAlternatives = [];
    this.formalityError = null;

    this.domainsState = 'ungenerated';
    this.domainExplorations = [];
    this.domainsError = null;

    this.usageState = 'ungenerated';
    this.usagePatterns = [];
    this.usageError = null;

    this.practiceState = 'ungenerated';
    this.practiceExercises = [];
    this.practiceError = null;

    this.mistakesState = 'ungenerated';
    this.commonMistakes = [];
    this.mistakesError = null;

    this.customContextState = 'ungenerated';
    this.customContext = '';
    this.customExamples = [];
    this.customContextError = null;

    this.logger.info('[ExploreStore] Reset all features');
  }

  async generateFormality(): Promise<void> {
    await this.runFeature({
      feature: 'formality',
      execute: (word) => api.generateFormalityAnalysis(word),
      onSuccess: (result) => {
        this.formalityPercentage = result.formalityPercentage;
        this.formalityAlternatives = result.formalityAlternatives;
        this.logger.info(
          `[ExploreStore] Formality generated: ${result.formalityPercentage}% with ${result.formalityAlternatives.length} alternatives`
        );
      },
    });
  }

  async generateDomains(): Promise<void> {
    await this.runFeature({
      feature: 'domains',
      execute: (word) => api.generateDomainExploration(word),
      onSuccess: (result) => {
        this.domainExplorations = result;
        this.logger.info(`[ExploreStore] Domains generated: ${result.length} domains`);
      },
    });
  }

  async generateUsage(): Promise<void> {
    await this.runFeature({
      feature: 'usage',
      execute: (word) => api.generateUsagePatterns(word),
      onSuccess: (result) => {
        this.usagePatterns = result;
        this.logger.info(`[ExploreStore] Usage patterns generated: ${result.length} patterns`);
      },
    });
  }

  async generatePractice(force = false): Promise<void> {
    await this.runFeature({
      feature: 'practice',
      execute: (word) => api.generatePracticeExercisesOnly(word, 5, force),
      onSuccess: (result) => {
        this.practiceExercises = result.practiceExercises;
        this.logger.info(
          `[ExploreStore] Practice generated: ${result.practiceExercises.length} exercises`
        );
      },
    });
  }

  async generateMistakes(force = false): Promise<void> {
    await this.runFeature({
      feature: 'mistakes',
      execute: (word) => api.generateCommonMistakes(word, force),
      onSuccess: (mistakes) => {
        this.commonMistakes = mistakes;
        this.logger.info(`[ExploreStore] Mistakes generated: ${mistakes.length} items`);
      },
    });
  }

  async generateCustomExamples(context: string): Promise<void> {
    if (!context.trim()) {
      this.logger.info('[ExploreStore] Cannot generate custom examples: empty context');
      return;
    }

    this.customContext = context.trim();

    await this.runFeature({
      feature: 'custom',
      execute: (word) => api.generateContextualExamples(word, this.customContext),
      onSuccess: (examples) => {
        this.customExamples = examples;
        this.logger.info(`[ExploreStore] Custom examples generated: ${examples.length} examples`);
      },
    });
  }

  // Computed properties

  get hasAnyGenerated(): boolean {
    return (
      this.formalityState === 'generated' ||
      this.domainsState === 'generated' ||
      this.usageState === 'generated' ||
      this.practiceState === 'generated' ||
      this.mistakesState === 'generated' ||
      this.customContextState === 'generated'
    );
  }

  get allFeaturesGenerated(): boolean {
    return (
      this.formalityState === 'generated' &&
      this.domainsState === 'generated' &&
      this.usageState === 'generated' &&
      this.practiceState === 'generated' &&
      this.mistakesState === 'generated'
    );
  }

  private getFeatureRunnerHooks(): FeatureRunnerHooks {
    return {
      logger: this.logger,
      getCurrentWord: () => this.currentWord,
      getFeatureState: (feature) => this.getFeatureState(feature),
      setFeatureState: (feature, state, error) => this.setFeatureState(feature, state, error ?? null),
      startTimeout: (feature, timeoutMs) => this.startFeatureTimeout(feature, timeoutMs),
      clearTimeout: (feature) => this.clearFeatureTimeout(feature),
    };
  }

  private async runFeature<T>({ feature, execute, onSuccess, customTimeoutMs }: FeatureRunnerConfig<T>): Promise<void> {
    await runFeatureTask(this.getFeatureRunnerHooks(), {
      feature,
      execute,
      onSuccess,
      customTimeoutMs,
    });
  }

  private getFeatureState(feature: FeatureKey): FeatureState {
    switch (feature) {
      case 'formality':
        return this.formalityState;
      case 'domains':
        return this.domainsState;
      case 'usage':
        return this.usageState;
      case 'practice':
        return this.practiceState;
      case 'custom':
        return this.customContextState;
      case 'mistakes':
        return this.mistakesState;
    }
  }

  private setFeatureState(feature: FeatureKey, state: FeatureState, error: string | null = null): void {
    switch (feature) {
      case 'formality':
        this.formalityState = state;
        this.formalityError = error;
        break;
      case 'domains':
        this.domainsState = state;
        this.domainsError = error;
        break;
      case 'usage':
        this.usageState = state;
        this.usageError = error;
        break;
      case 'practice':
        this.practiceState = state;
        this.practiceError = error;
        break;
      case 'custom':
        this.customContextState = state;
        this.customContextError = error;
        break;
      case 'mistakes':
        this.mistakesState = state;
        this.mistakesError = error;
        break;
    }
  }

  private startFeatureTimeout(feature: FeatureKey, timeoutMs: number): void {
    this.clearFeatureTimeout(feature);

    const timeoutId = window.setTimeout(() => {
      runInAction(() => {
        const message = `${getFeatureBlueprint(feature).displayName} timed out. Please try again.`;
        this.setFeatureState(feature, 'error', message);
      });
      this.featureTimeouts.delete(feature);
    }, timeoutMs);

    this.featureTimeouts.set(feature, timeoutId);
  }

  private clearFeatureTimeout(feature: FeatureKey): void {
    const timeoutId = this.featureTimeouts.get(feature);
    if (timeoutId) {
      clearTimeout(timeoutId);
      this.featureTimeouts.delete(feature);
    }
  }

  private clearAllFeatureTimeouts(): void {
    this.featureTimeouts.forEach((timeoutId) => clearTimeout(timeoutId));
    this.featureTimeouts.clear();
  }

  private setupWordSync(): void {
    this.wordSyncDisposer = reaction(
      () => this.wordSource.currentWord,
      (word, previousWord) => {
        if (word === previousWord) {
          return;
        }

        if (!word) {
          return;
        }

        this.logger.info(
          previousWord
            ? `[ExploreStore] Word changed from ${previousWord} to ${word} - resetting features`
            : `[ExploreStore] Word set to ${word}`
        );
        this.exploreWord = word;
        this.resetAll();
        void this.hydrateFromCache(word);
      }
    );

    this.searchResetDisposer = reaction(
      () => this.wordSource.loadingState,
      (state, previousState) => {
        if (state === 'loading' && previousState !== 'loading') {
          this.logger.info('[ExploreStore] Progressive search restarted - resetting features');
          this.resetAll();
        }

        if (state === 'success' && previousState === 'loading') {
          this.logger.info('[ExploreStore] Progressive search completed - hydrating cache');
          void this.hydrateFromCache(this.currentWord);
        }
      }
    );
  }

  cleanup(): void {
    this.wordSyncDisposer?.();
    this.searchResetDisposer?.();
    this.clearAllFeatureTimeouts();
  }

  private async hydrateFromCache(word: string): Promise<void> {
    const targetWord = word.trim();
    if (!targetWord) {
      this.logger.debug('[ExploreStore] hydrateFromCache skipped: empty word input');
      return;
    }

    this.logger.debug('[ExploreStore] hydrateFromCache invoked', { word: targetWord });
    this.cacheHydrationToken = targetWord;
    try {
      this.logger.debug('[ExploreStore] Requesting cached explore payload', { word: targetWord });
      const cached = await api.getCachedExplorationFeatures(targetWord);
      const payloadMetrics = {
        word: targetWord,
        hasFormality: Boolean(cached.formality),
        domainCount: cached.domains?.length ?? 0,
        usageCount: cached.usage?.length ?? 0,
        practiceExercises: cached.practice?.practiceExercises.length ?? 0,
      };
      this.logger.debug('[ExploreStore] Cached explore payload received', payloadMetrics);

      runInAction(() => {
        if (this.cacheHydrationToken !== targetWord || this.currentWord.trim() !== targetWord) {
          this.logger.info(
            `[ExploreStore] Ignoring cached exploration payload for stale word: ${targetWord}`,
            {
              cacheHydrationToken: this.cacheHydrationToken,
              currentWord: this.currentWord.trim(),
            }
          );
          return;
        }

        const hydratedFeatures: FeatureKey[] = [];

        if (cached.formality) {
          this.formalityPercentage = cached.formality.formalityPercentage;
          this.formalityAlternatives = cached.formality.formalityAlternatives;
          this.formalityState = 'generated';
          this.formalityError = null;
          hydratedFeatures.push('formality');
          this.logger.debug('[ExploreStore] Hydrated cached formality data', {
            word: targetWord,
            percentage: cached.formality.formalityPercentage,
            alternatives: cached.formality.formalityAlternatives.length,
          });
        }

        if (cached.domains && cached.domains.length > 0) {
          const domainsDetailLevel =
            cached.domains.length > COMPACT_DOMAIN_LIMIT ? 'extended' : 'compact';
          this.domainExplorations =
            domainsDetailLevel === 'compact'
              ? cached.domains.slice(0, COMPACT_DOMAIN_LIMIT)
              : cached.domains;
          this.domainsState = 'generated';
          this.domainsError = null;
          hydratedFeatures.push('domains');
          this.logger.debug('[ExploreStore] Hydrated cached domains data', {
            word: targetWord,
            count: this.domainExplorations.length,
          });
        }

        if (cached.usage && cached.usage.length > 0) {
          this.usagePatterns = cached.usage;
          this.usageState = 'generated';
          this.usageError = null;
          hydratedFeatures.push('usage');
          this.logger.debug('[ExploreStore] Hydrated cached usage data', {
            word: targetWord,
            count: this.usagePatterns.length,
          });
        }

        if (cached.practice) {
          this.practiceExercises = cached.practice.practiceExercises;
          if (this.practiceExercises.length > 0) {
            this.practiceState = 'generated';
            this.practiceError = null;
            hydratedFeatures.push('practice');
            this.logger.debug('[ExploreStore] Hydrated cached practice data', {
              word: targetWord,
              exercises: this.practiceExercises.length,
            });
          }
        }

        if (hydratedFeatures.length > 0) {
          this.logger.info('[ExploreStore] Hydrated explore features from cache', {
            word: targetWord,
            features: hydratedFeatures,
          });
        } else {
          this.logger.debug('[ExploreStore] No cached explore features applied', {
            word: targetWord,
          });
        }
      });
    } catch (error) {
      this.logger.error('[ExploreStore] Failed to hydrate cached explore data', error as Error, {
        word: targetWord,
      });
    } finally {
      if (this.cacheHydrationToken === targetWord) {
        this.cacheHydrationToken = null;
      }
    }
  }
}
