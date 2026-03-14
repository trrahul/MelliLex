import { describe, it, expect, vi, beforeEach } from 'vitest';
import { api } from '../../services/api';
import { getWordVariationsWithCache, __testing__ } from '../textHighlight';

vi.mock('../../services/api', () => ({
  api: {
    getWordVariations: vi.fn(),
  },
}));

describe('getWordVariationsWithCache', () => {
  beforeEach(() => {
    __testing__.resetCaches();
    vi.resetAllMocks();
  });

  it('deduplicates concurrent requests for the same word', async () => {
    const mockedGetWordVariations = vi.mocked(api.getWordVariations);
    mockedGetWordVariations.mockImplementation(async () => {
      await new Promise((resolve) => setTimeout(resolve, 10));
      return ['coverage', 'covering'];
    });

    const [first, second] = await Promise.all([
      getWordVariationsWithCache('Coverage'),
      getWordVariationsWithCache('coverage'),
    ]);

    expect(api.getWordVariations).toHaveBeenCalledTimes(1);
    expect(first).toEqual(['coverage', 'covering']);
    expect(second).toEqual(['coverage', 'covering']);
  });

  it('reuses cached results on subsequent calls', async () => {
    const mockedGetWordVariations = vi.mocked(api.getWordVariations);
    mockedGetWordVariations.mockResolvedValue(['coverage']);

    await getWordVariationsWithCache('Coverage');
    await getWordVariationsWithCache('coverage');

    expect(api.getWordVariations).toHaveBeenCalledTimes(1);
  });

  it('returns fallback when API call fails', async () => {
    const mockedGetWordVariations = vi.mocked(api.getWordVariations);
    mockedGetWordVariations.mockRejectedValue(new Error('fail'));

    const result = await getWordVariationsWithCache('Coverage');

    expect(api.getWordVariations).toHaveBeenCalledTimes(1);
    expect(result).toEqual(['Coverage']);
  });
});
