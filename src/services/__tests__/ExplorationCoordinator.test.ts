import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { NavigationStore } from '../../stores/NavigationStore';

describe('NavigationStore', () => {
  let store: NavigationStore;

  beforeEach(() => {
    store = new NavigationStore();
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2024-01-01T00:00:00Z'));
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  const travel = (ms: number) => {
    vi.advanceTimersByTime(ms);
    vi.setSystemTime(Date.now() + ms);
  };

  it('adds navigation entries and updates current word', () => {
    store.navigateTo('spark');
    store.navigateTo('ember', 'related-word');

    expect(store.currentWord).toBe('ember');
    expect(store.history).toHaveLength(2);
    expect(store.history[1].source).toBe('related-word');
  });

  it('truncates forward history when navigating mid-stack', () => {
    store.navigateTo('spark');
    store.navigateTo('ember');
    store.navigateTo('glow');

    expect(store.history.map(entry => entry.word)).toEqual(['spark', 'ember', 'glow']);

    store.goBack();
    store.goBack();
    expect(store.currentWord).toBe('spark');

    store.navigateTo('flare');
    expect(store.history.map(entry => entry.word)).toEqual(['spark', 'flare']);
    expect(store.canGoForward).toBe(false);
  });

  it('supports goBack and goForward navigation', () => {
    store.navigateTo('spark');
    store.navigateTo('ember');
    store.navigateTo('flare');

    expect(store.goBack()).toBe('ember');
    expect(store.goBack()).toBe('spark');
    expect(store.canGoBack).toBe(false);
    expect(store.goForward()).toBe('ember');
    expect(store.currentWord).toBe('ember');
  });

  it('clears history completely', () => {
    store.navigateTo('spark');
    store.navigateTo('ember');

    store.clear();

    expect(store.history).toHaveLength(0);
    expect(store.currentWord).toBeNull();
    expect(store.canGoBack).toBe(false);
  });

  it('returns breadcrumb trail and derived metrics', () => {
    store.navigateTo('spark');
    travel(10);
    store.navigateTo('ember');

    expect(store.breadcrumbTrail.map(entry => entry.word)).toEqual(['spark', 'ember']);
    expect(store.pathDepth).toBe(2);
    expect(store.uniqueWordsExplored).toBe(2);
    expect(store.getNavigationPath()).toEqual(['spark', 'ember']);

    const edges = store.getNavigationEdges();
    expect(edges).toHaveLength(1);
    expect(edges[0]).toEqual({ from: 'spark', to: 'ember', timestamp: store.history[1].timestamp });
  });

  it('limits history to configured maximum size', () => {
    (store as any).maxHistorySize = 3;

    store.navigateTo('one');
    store.navigateTo('two');
    store.navigateTo('three');
    store.navigateTo('four');

    expect(store.history.map(entry => entry.word)).toEqual(['two', 'three', 'four']);
    expect(store.currentIndex).toBe(2);
  });
});