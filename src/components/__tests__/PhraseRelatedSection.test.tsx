import { screen, fireEvent } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { PhraseRelatedSection } from '../phrase/PhraseRelatedSection';
import { renderWithProviders, createMockRootStore } from '../../test/test-utils';
import type { PhraseSection3Related } from '../../types';

const sampleRelated: PhraseSection3Related = {
  variations: [
    { phrase: 'break the ice', region: 'universal', note: 'Most common form' },
    { phrase: 'crack the ice', region: 'british', note: 'British variant' },
  ],
  similarPhrases: [
    { phrase: 'warm up to', meaningHint: 'Gradually become friendlier' },
    { phrase: 'get the ball rolling', meaningHint: 'Start an activity or process' },
  ],
  oppositePhrases: [
    { phrase: 'cold shoulder', meaningHint: 'Deliberately ignore someone' },
  ],
  seeAlso: ['hit it off', 'make small talk'],
};

describe('PhraseRelatedSection', () => {
  it('shows region badge for non-universal variations', () => {
    renderWithProviders(<PhraseRelatedSection data={sampleRelated} />);
    expect(screen.getByText('UK')).toBeInTheDocument();
  });

  it('does not show region badge for universal variations', () => {
    const universalOnly: PhraseSection3Related = {
      ...sampleRelated,
      variations: [{ phrase: 'test phrase', region: 'universal' }],
    };
    renderWithProviders(<PhraseRelatedSection data={universalOnly} />);
    expect(screen.queryByText('Universal')).toBeNull();
  });

  it('renders all related phrase groups', () => {
    renderWithProviders(<PhraseRelatedSection data={sampleRelated} />);
    expect(screen.getByText('break the ice')).toBeInTheDocument();
    expect(screen.getByText(/Most common form/)).toBeInTheDocument();
    expect(screen.getByText('warm up to')).toBeInTheDocument();
    expect(screen.getByText('cold shoulder')).toBeInTheDocument();
    expect(screen.getByText('hit it off')).toBeInTheDocument();
  });

  it('calls onNavigate when a variation is clicked', () => {
    const onNavigate = vi.fn();
    renderWithProviders(
      <PhraseRelatedSection data={sampleRelated} onNavigate={onNavigate} />
    );
    fireEvent.click(screen.getByText('crack the ice'));
    expect(onNavigate).toHaveBeenCalledWith('crack the ice');
  });

  it('calls searchCoordinator.search when no onNavigate provided', () => {
    const store = createMockRootStore();
    renderWithProviders(<PhraseRelatedSection data={sampleRelated} />, { store });
    fireEvent.click(screen.getByText('cold shoulder'));
    expect(store.searchCoordinator.search).toHaveBeenCalledWith('cold shoulder', { source: 'related-phrase' });
  });

  it('calls onNavigate when a see-also item is clicked', () => {
    const onNavigate = vi.fn();
    renderWithProviders(
      <PhraseRelatedSection data={sampleRelated} onNavigate={onNavigate} />
    );
    fireEvent.click(screen.getByText('hit it off'));
    expect(onNavigate).toHaveBeenCalledWith('hit it off');
  });

  it('hides sections when data arrays are empty', () => {
    const emptyRelated: PhraseSection3Related = {
      variations: [],
      similarPhrases: [],
      oppositePhrases: [],
      seeAlso: [],
    };
    renderWithProviders(<PhraseRelatedSection data={emptyRelated} />);
    expect(screen.queryByText(/Variations/i)).toBeNull();
    expect(screen.queryByText(/Similar Meaning/i)).toBeNull();
    expect(screen.queryByText(/Opposite Meaning/i)).toBeNull();
    expect(screen.queryByText(/See Also/i)).toBeNull();
  });
});
