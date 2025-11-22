import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import '@testing-library/jest-dom/vitest';
import { CustomContextCard } from '../explore/CustomContextCard';
import '../../i18n';

describe('CustomContextCard', () => {
  it('renders generated examples using quote-block styling', () => {
    const examples = [
      'Outline the core platform benefits for enterprise clients.',
      'Emphasize how the service de-risks global rollouts.',
    ];

    const { container } = render(
      <CustomContextCard
        state="generated"
        examples={examples}
        contextLabel="Executive memo"
        error={null}
        onGenerate={vi.fn()}
      />
    );

    const quoteBlocks = container.querySelectorAll('.quote-block');
    expect(quoteBlocks).toHaveLength(examples.length);
    expect(quoteBlocks[0]).toHaveTextContent(examples[0]);
    expect(quoteBlocks[0].className).not.toContain('italic');
    expect(screen.getByText(/Custom Context Examples/i)).toBeInTheDocument();
  });
});
