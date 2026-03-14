import { describe, it, expect, beforeEach, vi } from 'vitest';
import { HistoryStore } from '../../stores/HistoryStore';
import type { WordHistory } from '../../types';

const apiMocks = vi.hoisted(() => ({
  getHistory: vi.fn(),
  clearHistory: vi.fn(),
  deleteHistoryItem: vi.fn(),
}));

vi.mock('../../services/api', () => ({
  api: apiMocks,
}));

describe('HistoryStore', () => {
  let store: HistoryStore;

  const sampleHistory = (): WordHistory[] => [
    { id: '1', word: 'spark', aiProvider: 'openai', searchedAt: '2024-01-01T00:00:00Z' },
    { id: '2', word: 'glow', aiProvider: 'anthropic', searchedAt: '2024-01-02T00:00:00Z' },
  ];

  beforeEach(() => {
    vi.clearAllMocks();
    store = new HistoryStore();
  });

  describe('loadHistory', () => {
    it('loads items and marks success state', async () => {
      apiMocks.getHistory.mockResolvedValue(sampleHistory());

      await store.loadHistory();

      expect(apiMocks.getHistory).toHaveBeenCalled();
      expect(store.items).toHaveLength(2);
      expect(store.isSuccess).toBe(true);
      expect(store.error).toBeNull();
    });

    it('stores error message when load fails', async () => {
      apiMocks.getHistory.mockRejectedValue(new Error('db down'));

      await store.loadHistory();

      expect(store.hasError).toBe(true);
      expect(store.error).toBe('Failed to load history');
    });
  });

  describe('clearHistory', () => {
    it('clears items after API success', async () => {
      store.items = sampleHistory();

      await store.clearHistory();

      expect(apiMocks.clearHistory).toHaveBeenCalled();
      expect(store.items).toHaveLength(0);
    });

    it('captures errors from API failure', async () => {
      apiMocks.clearHistory.mockRejectedValue(new Error('permission denied'));

      await store.clearHistory();

      expect(store.error).toBe('permission denied');
    });
  });

  describe('deleteItem', () => {
    beforeEach(() => {
      store.items = sampleHistory();
    });

    it('removes item locally after API delete', async () => {
      await store.deleteItem('1');

      expect(apiMocks.deleteHistoryItem).toHaveBeenCalledWith('1');
      expect(store.items).toHaveLength(1);
      expect(store.items[0].id).toBe('2');
    });

    it('sets error when deletion fails', async () => {
      apiMocks.deleteHistoryItem.mockRejectedValue(new Error('network'));

      await store.deleteItem('1');

      expect(store.error).toBe('network');
      expect(store.items).toHaveLength(2);
    });
  });

  describe('computed helpers', () => {
    beforeEach(() => {
      const extendedHistory = Array.from({ length: 12 }).map((_, idx) => ({
        id: String(idx + 1),
        word: `word-${idx + 1}`,
        aiProvider: idx % 2 === 0 ? 'openai' : 'anthropic',
        searchedAt: new Date(2024, 0, idx + 1).toISOString(),
      }));
      store.items = extendedHistory;
    });

    it('returns most recent 10 words', () => {
      expect(store.recentWords).toHaveLength(10);
      expect(store.recentWords[0].word).toBe('word-1');
    });

    it('filters items by search query', () => {
      store.setSearchQuery('word-12');
      expect(store.filteredItems).toHaveLength(1);

      store.setSearchQuery('anthropic');
      expect(store.filteredItems.every(item => item.aiProvider === 'anthropic')).toBe(true);
    });

    it('returns all items when query empty', () => {
      store.setSearchQuery('');
      expect(store.filteredItems).toHaveLength(12);
    });
  });
});
