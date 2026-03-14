import { screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { Section2MeaningsView } from '../sections/Section2MeaningsView';
import { renderWithProviders } from '../../test/test-utils';
import type { WordSection2Meanings } from '../../types';

// Mock the HighlightedText component to avoid the async word-variations API call
vi.mock('../../utils/textHighlight', () => ({
  HighlightedText: ({ text }: { text: string }) => <span>{text}</span>,
}));

const sampleMeanings: WordSection2Meanings = {
  meanings: [
    {
      number: 1,
      partOfSpeech: 'adjective',
      definition: 'Fluent or persuasive in speaking or writing.',
      memoryTip: 'Think of "eloquence" as elegant fluency.',
      examples: [
        'She gave an eloquent speech at the ceremony.',
        'His eloquent writing moved many readers.',
      ],
    },
    {
      number: 2,
      partOfSpeech: 'adjective',
      definition: 'Clearly expressing or indicating something.',
      memoryTip: '',
      examples: ['The statistics are eloquent testimony to the problem.'],
    },
  ],
};

describe('Section2MeaningsView', () => {
  it('renders core meaning content', () => {
    renderWithProviders(
      <Section2MeaningsView data={sampleMeanings} word="eloquent" />
    );
    expect(screen.getByText('Meanings')).toBeInTheDocument();
    expect(screen.getByText('1.')).toBeInTheDocument();
    expect(screen.getByText('2.')).toBeInTheDocument();
    expect(screen.getByText(/Fluent or persuasive/)).toBeInTheDocument();
    expect(screen.getByText(/Clearly expressing/)).toBeInTheDocument();
    expect(screen.getAllByText('adjective')).toHaveLength(2);
    expect(screen.getByText(/elegant fluency/)).toBeInTheDocument();
  });

  it('renders example sentences', () => {
    renderWithProviders(
      <Section2MeaningsView data={sampleMeanings} word="eloquent" />
    );
    expect(screen.getByText(/eloquent speech at the ceremony/)).toBeInTheDocument();
    expect(screen.getByText(/eloquent writing moved/)).toBeInTheDocument();
  });

  it('handles empty meanings array', () => {
    renderWithProviders(
      <Section2MeaningsView data={{ meanings: [] }} word="test" />
    );
    expect(screen.getByText('Meanings')).toBeInTheDocument();
  });
});
