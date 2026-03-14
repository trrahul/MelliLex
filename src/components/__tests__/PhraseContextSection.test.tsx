import { screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { PhraseContextSection } from '../phrase/PhraseContextSection';
import { renderWithProviders } from '../../test/test-utils';
import type { PhraseSection2Context } from '../../types';

// Mock HighlightedText to avoid async API calls
vi.mock('../../utils/textHighlight', () => ({
  HighlightedText: ({ text }: { text: string }) => <span>{text}</span>,
}));

const sampleContext: PhraseSection2Context = {
  origin: {
    story: 'Originally referred to breaking ice on trade routes for ships.',
    era: '17th century',
    source: 'Maritime trade',
    evolution: 'Later adopted metaphorically for social situations.',
  },
  usageNotes: [
    {
      context: 'Social gathering',
      example: 'He told a joke to break the ice at the party.',
      tone: 'casual',
    },
    {
      context: 'Business meeting',
      example: 'She used small talk to break the ice before the presentation.',
      tone: 'professional',
    },
  ],
  commonMistakes: [
    {
      mistakeType: 'Wrong preposition',
      incorrect: 'break the ice to someone',
      correct: 'break the ice with someone',
      explanation: 'Use "with" not "to" when referring to the person.',
    },
  ],
};

describe('PhraseContextSection', () => {
  it('renders origin details including metadata and evolution', () => {
    renderWithProviders(
      <PhraseContextSection data={sampleContext} phrase="break the ice" />
    );
    expect(screen.getByText(/breaking ice on trade routes/)).toBeInTheDocument();
    expect(screen.getByText('17th century')).toBeInTheDocument();
    expect(screen.getByText('Maritime trade')).toBeInTheDocument();
    expect(screen.getByText(/adopted metaphorically/)).toBeInTheDocument();
  });

  it('renders usage notes and tone labels', () => {
    renderWithProviders(
      <PhraseContextSection data={sampleContext} phrase="break the ice" />
    );
    expect(screen.getByText('Social gathering')).toBeInTheDocument();
    expect(screen.getByText('Business meeting')).toBeInTheDocument();
    expect(screen.getByText('casual')).toBeInTheDocument();
    expect(screen.getByText('professional')).toBeInTheDocument();
    expect(screen.getByText(/told a joke to break the ice/)).toBeInTheDocument();
  });

  it('renders common mistakes with correction details', () => {
    renderWithProviders(
      <PhraseContextSection data={sampleContext} phrase="break the ice" />
    );
    expect(screen.getByText('Wrong preposition')).toBeInTheDocument();
    expect(screen.getByText(/break the ice to someone/)).toBeInTheDocument();
    expect(screen.getByText(/break the ice with someone/)).toBeInTheDocument();
    expect(screen.getByText(/Use "with" not "to"/)).toBeInTheDocument();
  });

  it('hides usage notes section when empty', () => {
    renderWithProviders(
      <PhraseContextSection
        data={{ ...sampleContext, usageNotes: [] }}
        phrase="break the ice"
      />
    );
    expect(screen.queryByText(/Usage Notes/i)).toBeNull();
  });

  it('hides common mistakes section when empty', () => {
    renderWithProviders(
      <PhraseContextSection
        data={{ ...sampleContext, commonMistakes: [] }}
        phrase="break the ice"
      />
    );
    expect(screen.queryByText(/Common Mistakes/i)).toBeNull();
  });

  it('hides era/source badges when not present', () => {
    const minimal: PhraseSection2Context = {
      origin: { story: 'Unknown origin.' },
      usageNotes: [],
      commonMistakes: [],
    };
    renderWithProviders(
      <PhraseContextSection data={minimal} phrase="test" />
    );
    expect(screen.getByText('Unknown origin.')).toBeInTheDocument();
    // No badges should render
    expect(screen.queryByText('17th century')).toBeNull();
  });
});
