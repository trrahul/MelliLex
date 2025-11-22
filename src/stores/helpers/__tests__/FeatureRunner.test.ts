import { describe, expect, it, vi } from 'vitest';
import { runFeatureTask, type FeatureRunnerHooks } from '../FeatureRunner';

const createHooks = (overrides?: Partial<FeatureRunnerHooks>): FeatureRunnerHooks => {
  const stateMap = new Map<string, any>();
  const logger = {
    info: vi.fn(),
    error: vi.fn(),
    debug: vi.fn(),
  };

  const hooks: FeatureRunnerHooks = {
    logger,
    getCurrentWord: () => 'spark',
    getFeatureState: (feature) => stateMap.get(feature) ?? 'ungenerated',
    setFeatureState: (feature, state) => stateMap.set(feature, state),
    startTimeout: vi.fn(),
    clearTimeout: vi.fn(),
  };

  return { ...hooks, ...overrides };
};

describe('FeatureRunner', () => {
  it('skips execution when no word is available', async () => {
    const execute = vi.fn();
    const hooks = createHooks({ getCurrentWord: () => ' ' });

    await runFeatureTask(hooks, {
      feature: 'practice',
      execute,
      onSuccess: vi.fn(),
    });

    expect(execute).not.toHaveBeenCalled();
  });

  it('transitions through states on success', async () => {
    const hooks = createHooks();
    const onSuccess = vi.fn();

    await runFeatureTask(hooks, {
      feature: 'practice',
      execute: async () => ['exercise'],
      onSuccess,
    });

    expect(onSuccess).toHaveBeenCalledWith(['exercise']);
    expect(hooks.getFeatureState('practice')).toBe('generated');
  });

  it('propagates errors to state', async () => {
    const hooks = createHooks();

    await runFeatureTask(hooks, {
      feature: 'practice',
      execute: async () => {
        throw new Error('boom');
      },
      onSuccess: vi.fn(),
    });

    expect(hooks.getFeatureState('practice')).toBe('error');
  });
});
