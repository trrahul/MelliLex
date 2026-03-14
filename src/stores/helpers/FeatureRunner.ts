import { runInAction } from 'mobx';
import { TIMEOUTS } from '../../constants/events';
import type { ILogger } from '../../services/LoggerService';
import { getErrorMessage } from '../../utils/errorHandler';

export type FeatureKey = 'formality' | 'domains' | 'usage' | 'practice' | 'custom' | 'mistakes';
export type FeatureState = 'ungenerated' | 'generating' | 'generated' | 'error';

interface FeatureBlueprint {
  displayName: string;
  defaultTimeoutMs: number;
}

const DEFAULT_TIMEOUT_MS = TIMEOUTS.EXPLORATION;

const FEATURE_BLUEPRINTS: Record<FeatureKey, FeatureBlueprint> = {
  formality: { displayName: 'Formality analysis', defaultTimeoutMs: DEFAULT_TIMEOUT_MS },
  domains: { displayName: 'Domain exploration', defaultTimeoutMs: DEFAULT_TIMEOUT_MS },
  usage: { displayName: 'Usage patterns', defaultTimeoutMs: DEFAULT_TIMEOUT_MS },
  practice: { displayName: 'Practice exercises', defaultTimeoutMs: DEFAULT_TIMEOUT_MS },
  custom: { displayName: 'Custom examples', defaultTimeoutMs: DEFAULT_TIMEOUT_MS },
  mistakes: { displayName: 'Common mistakes', defaultTimeoutMs: DEFAULT_TIMEOUT_MS },
};

export const getFeatureBlueprint = (feature: FeatureKey): FeatureBlueprint => FEATURE_BLUEPRINTS[feature];

export interface FeatureRunnerConfig<T> {
  feature: FeatureKey;
  execute: (word: string) => Promise<T>;
  onSuccess: (result: T) => void;
  customTimeoutMs?: number;
}

export interface FeatureRunnerHooks {
  logger: ILogger;
  getCurrentWord(): string;
  getFeatureState(feature: FeatureKey): FeatureState;
  setFeatureState(feature: FeatureKey, state: FeatureState, error?: string | null): void;
  startTimeout(feature: FeatureKey, timeoutMs: number): void;
  clearTimeout(feature: FeatureKey): void;
}

export async function runFeatureTask<T>(
  hooks: FeatureRunnerHooks,
  config: FeatureRunnerConfig<T>,
): Promise<void> {
  const word = hooks.getCurrentWord().trim();
  const blueprint = getFeatureBlueprint(config.feature);
  const displayName = blueprint.displayName;
  const timeoutMs = config.customTimeoutMs ?? blueprint.defaultTimeoutMs;
  if (!word) {
    hooks.logger.info(`[ExploreStore] Cannot generate ${displayName}: no current word`);
    return;
  }

  if (hooks.getFeatureState(config.feature) === 'generating') {
    hooks.logger.info(
      `[ExploreStore] ${displayName} already in progress`,
      { feature: config.feature },
    );
    return;
  }

  hooks.logger.info(`[ExploreStore] Generating ${displayName} for: ${word}`);

  runInAction(() => {
    hooks.setFeatureState(config.feature, 'generating');
  });

  hooks.startTimeout(config.feature, timeoutMs);

  try {
    const result = await config.execute(word);

    runInAction(() => {
      if (hooks.getCurrentWord().trim() !== word) {
        hooks.logger.info(
          `[ExploreStore] Skipping ${displayName} result for stale word: ${word}`,
          { feature: config.feature },
        );
        return;
      }
      config.onSuccess(result);
      hooks.setFeatureState(config.feature, 'generated');
    });
  } catch (error) {
    const errorMsg = getErrorMessage(
      error,
      `Failed to generate ${displayName.toLowerCase()}`,
    );
    hooks.logger.error(
      `[ExploreStore] ${displayName} generation failed`,
      error as Error,
      { feature: config.feature },
    );

    runInAction(() => {
      hooks.setFeatureState(config.feature, 'error', errorMsg);
    });
  } finally {
    hooks.clearTimeout(config.feature);
  }
}
