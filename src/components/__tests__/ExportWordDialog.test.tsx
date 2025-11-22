import { fireEvent, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi, beforeEach } from 'vitest';
import { ExportWordDialog } from '../ExportWordDialog';
import { renderWithProviders, createMockRootStore } from '../../test/test-utils';

const toastSuccess = vi.fn();
const toastError = vi.fn();

vi.mock('sonner', () => ({
  toast: {
    success: (...args: unknown[]) => toastSuccess(...args),
    error: (...args: unknown[]) => toastError(...args),
  },
}));

const exportToMarkdownFile = vi.fn();
const exportToCapacities = vi.fn();

vi.mock('../../services/ExportService', () => ({
  ExportService: {
    exportToMarkdownFile: (...args: unknown[]) => exportToMarkdownFile(...args),
    exportToCapacities: (...args: unknown[]) => exportToCapacities(...args),
  },
}));

describe('ExportWordDialog', () => {
  beforeEach(() => {
    exportToMarkdownFile.mockReset();
    exportToCapacities.mockReset();
    toastSuccess.mockReset();
    toastError.mockReset();
  });

  it('disables trigger button when header section is missing', () => {
    const store = createMockRootStore({
      progressiveWordStore: { headerSection: null },
    });

    renderWithProviders(<ExportWordDialog />, { store });

    expect(screen.getByRole('button', { name: /export/i })).toBeDisabled();
  });

  it('exports markdown with provider and includeTimestamp from settings', async () => {
    exportToMarkdownFile.mockResolvedValue('C:/tmp/eloquent.md');

    const store = createMockRootStore({
      progressiveWordStore: {
        headerSection: {
          word: 'eloquent',
          pronunciation: '/x/',
          syllables: 'el·o·quent',
          origin: 'Latin',
          formality: { level: 'Formal', percentage: 75 },
          domains: [],
          tldr: 'summary',
        },
      },
      settingsStore: {
        settings: {
          aiProvider: 'openai',
          theme: 'light',
          enableGlobalLookup: false,
          globalLookupShortcut: 'CTRL+ALT+D',
          exportSettings: {
            capacities: {
              apiToken: 'token',
              spaceId: 'space',
              defaultTags: ['vocab'],
              noTimestamp: true,
            },
          },
        },
      },
    });

    renderWithProviders(<ExportWordDialog />, { store });

    fireEvent.click(screen.getByRole('button', { name: /export/i }));
    fireEvent.click(await screen.findByRole('button', { name: /export as markdown/i }));

    await waitFor(() => {
      expect(exportToMarkdownFile).toHaveBeenCalledWith('eloquent', 'openai', false);
      expect(toastSuccess).toHaveBeenCalled();
    });
  });

  it('shows capacities hint and disabled capacities button when config is missing', async () => {
    const store = createMockRootStore({
      progressiveWordStore: {
        headerSection: {
          word: 'eloquent',
          pronunciation: '/x/',
          syllables: 'el·o·quent',
          origin: 'Latin',
          formality: { level: 'Formal', percentage: 75 },
          domains: [],
          tldr: 'summary',
        },
      },
      settingsStore: {
        settings: {
          aiProvider: 'openai',
          theme: 'light',
          enableGlobalLookup: false,
          globalLookupShortcut: 'CTRL+ALT+D',
          exportSettings: {},
        },
      },
    });

    renderWithProviders(<ExportWordDialog />, { store });

    fireEvent.click(screen.getByRole('button', { name: /export/i }));

    expect(
      await screen.findByText('Configure Capacities in Settings to enable one-click sharing')
    ).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /configure capacities first/i })).toBeDisabled();
  });

  it('exports to capacities with exploration only when enabled and available', async () => {
    exportToCapacities.mockResolvedValue(undefined);

    const store = createMockRootStore({
      progressiveWordStore: {
        headerSection: {
          word: 'eloquent',
          pronunciation: '/x/',
          syllables: 'el·o·quent',
          origin: 'Latin',
          formality: { level: 'Formal', percentage: 75 },
          domains: [],
          tldr: 'summary',
        },
        meaningsSection: { meanings: [] },
        relatedSection: { synonyms: [], antonyms: [], collocations: [] },
      },
      settingsStore: {
        settings: {
          aiProvider: 'openai',
          theme: 'light',
          enableGlobalLookup: false,
          globalLookupShortcut: 'CTRL+ALT+D',
          exportSettings: {
            includeExploration: true,
            capacities: {
              apiToken: 'token',
              spaceId: 'space',
              defaultTags: ['vocab'],
              noTimestamp: false,
            },
          },
        },
      },
      exploreStore: {
        formalityPercentage: 80,
        formalityAlternatives: [{ word: 'articulate', level: 'Formal', context: 'speech', explanation: 'more formal' }],
        domainExplorations: [],
        usagePatterns: [],
        commonMistakes: [],
        practiceExercises: [],
        customExamples: [],
        customContext: '',
      },
    });

    renderWithProviders(<ExportWordDialog />, { store });

    fireEvent.click(screen.getByRole('button', { name: /export/i }));
    fireEvent.click(await screen.findByRole('button', { name: /send to capacities/i }));

    await waitFor(() => {
      expect(exportToCapacities).toHaveBeenCalled();
    });

    const call = exportToCapacities.mock.calls[0];
    expect(call[2]).toEqual({ includeExploration: true, includeTimestamp: true });
  });

  it('shows error toast when markdown export fails', async () => {
    exportToMarkdownFile.mockRejectedValue(new Error('disk full'));

    const store = createMockRootStore({
      progressiveWordStore: {
        headerSection: {
          word: 'eloquent',
          pronunciation: '/x/',
          syllables: 'el·o·quent',
          origin: 'Latin',
          formality: { level: 'Formal', percentage: 75 },
          domains: [],
          tldr: 'summary',
        },
      },
    });

    renderWithProviders(<ExportWordDialog />, { store });

    fireEvent.click(screen.getByRole('button', { name: /export/i }));
    fireEvent.click(await screen.findByRole('button', { name: /export as markdown/i }));

    await waitFor(() => {
      expect(toastError).toHaveBeenCalled();
    });
  });
});
