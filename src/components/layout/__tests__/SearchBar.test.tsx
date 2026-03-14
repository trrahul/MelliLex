import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi, beforeEach } from 'vitest';
import { MemoryRouter } from 'react-router-dom';
import { SearchBar } from '../SearchBar';
import { RootStoreContext } from '../../../stores/RootStore';
import { createMockRootStore } from '../../../test/test-utils';
import '../../../i18n';

const mockNavigate = vi.fn();
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return { ...actual, useNavigate: () => mockNavigate };
});

vi.mock('../../SpellCheckDialog', () => ({
  SpellCheckDialog: ({ open, onSelectWord, onCancel, spellCheckData }: any) => (
    <div data-testid="spell-dialog" data-open={open ? 'yes' : 'no'}>
      <button type="button" onClick={() => onSelectWord(spellCheckData?.suggestedWord ?? 'fallback')}>
        choose-suggestion
      </button>
      <button type="button" onClick={onCancel}>cancel-spell</button>
    </div>
  ),
}));

const renderSearchBar = (route: string, storeOverrides: Record<string, unknown> = {}) => {
  const store = createMockRootStore(storeOverrides);
  const view = render(
    <MemoryRouter initialEntries={[route]}>
      <RootStoreContext.Provider value={store}>
        <SearchBar />
      </RootStoreContext.Provider>
    </MemoryRouter>
  );
  return { ...view, store };
};

describe('SearchBar', () => {
  beforeEach(() => {
    mockNavigate.mockReset();
  });

  it('submits word search with spell-check when on word routes', async () => {
    const search = vi.fn().mockResolvedValue(undefined);
    const checkSpelling = vi.fn().mockResolvedValue(null);
    const { store } = renderSearchBar('/explore', {
      progressiveWordStore: {
        currentWord: '',
        isLoading: false,
        clearCurrentWord: vi.fn(),
      },
      searchCoordinator: {
        search,
        checkSpelling,
      },
    });

    const input = screen.getByRole('textbox');
    fireEvent.change(input, { target: { value: '  eloquent  ' } });
    fireEvent.submit(input.closest('form')!);

    await waitFor(() => {
      expect(checkSpelling).toHaveBeenCalledWith('eloquent');
      expect(search).toHaveBeenCalledWith('eloquent', { source: 'search-bar' });
    });

    expect(mockNavigate).toHaveBeenCalledWith('/');
    expect((store.progressiveWordStore.clearCurrentWord as any)).not.toHaveBeenCalled();
  });

  it('opens spell-check dialog when suggestion is returned and searches selected suggestion', async () => {
    const search = vi.fn().mockResolvedValue(undefined);
    const checkSpelling = vi.fn().mockResolvedValue({
      originalWord: 'tset',
      isCorrect: false,
      suggestedWord: 'test',
      alternatives: ['test'],
    });

    renderSearchBar('/', {
      progressiveWordStore: {
        currentWord: '',
        isLoading: false,
        clearCurrentWord: vi.fn(),
      },
      searchCoordinator: {
        search,
        checkSpelling,
      },
    });

    const input = screen.getByRole('textbox');
    fireEvent.change(input, { target: { value: 'tset' } });
    fireEvent.submit(input.closest('form')!);

    await waitFor(() => {
      expect(screen.getByTestId('spell-dialog')).toHaveAttribute('data-open', 'yes');
    });

    fireEvent.click(screen.getByRole('button', { name: 'choose-suggestion' }));

    await waitFor(() => {
      expect(search).toHaveBeenCalledWith('test', { source: 'search-bar' });
    });
  });

  it('uses history strategy on history route', async () => {
    const setSearchQuery = vi.fn();
    const checkSpelling = vi.fn();

    renderSearchBar('/history', {
      historyStore: {
        searchQuery: '',
        loadingState: 'idle',
        setSearchQuery,
      },
      searchCoordinator: {
        checkSpelling,
      },
    });

    const input = screen.getByRole('textbox');
    fireEvent.change(input, { target: { value: '  hello  ' } });
    fireEvent.submit(input.closest('form')!);

    await waitFor(() => {
      expect(setSearchQuery).toHaveBeenCalledWith('hello');
    });

    expect(checkSpelling).not.toHaveBeenCalled();
  });

  it('clears current word when input becomes empty on word strategy', () => {
    const clearCurrentWord = vi.fn();

    renderSearchBar('/', {
      progressiveWordStore: {
        currentWord: 'existing',
        isLoading: false,
        clearCurrentWord,
      },
    });

    const input = screen.getByRole('textbox');
    fireEvent.change(input, { target: { value: '' } });

    expect(clearCurrentWord).toHaveBeenCalled();
  });

  it('disables search input on settings page', () => {
    renderSearchBar('/settings');
    expect(screen.getByRole('textbox')).toBeDisabled();
  });
});
