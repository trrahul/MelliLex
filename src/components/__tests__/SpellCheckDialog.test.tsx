import { render, screen, fireEvent } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { SpellCheckDialog } from '../SpellCheckDialog';
import '../../i18n';

describe('SpellCheckDialog', () => {
  const defaultProps = {
    open: true,
    onSelectWord: vi.fn(),
    onCancel: vi.fn(),
  };

  const mockSpellCheckData = {
    originalWord: 'tset',
    isCorrect: false,
    suggestedWord: 'test',
    alternatives: ['test', 'set', 'teat'],
  };

  it('renders nothing when spellCheckData is null', () => {
    const { container } = render(
      <SpellCheckDialog {...defaultProps} spellCheckData={null} />
    );
    expect(container.innerHTML).toBe('');
  });

  it('renders nothing when word is correct', () => {
    const { container } = render(
      <SpellCheckDialog
        {...defaultProps}
        spellCheckData={{ ...mockSpellCheckData, isCorrect: true }}
      />
    );
    expect(container.innerHTML).toBe('');
  });

  it('renders suggested word first with recommended label', () => {
    render(
      <SpellCheckDialog {...defaultProps} spellCheckData={mockSpellCheckData} />
    );
    const buttons = screen.getAllByRole('button');
    // First alternative should be "test" (the suggested word)
    const testButton = buttons.find(b => b.textContent?.includes('test') && b.textContent?.includes('recommended'));
    expect(testButton).toBeDefined();
  });

  it('calls onSelectWord when an alternative is clicked', () => {
    const onSelectWord = vi.fn();
    render(
      <SpellCheckDialog
        {...defaultProps}
        onSelectWord={onSelectWord}
        spellCheckData={mockSpellCheckData}
      />
    );
    fireEvent.click(screen.getByRole('button', { name: /^set$/ }));
    expect(onSelectWord).toHaveBeenCalledWith('set');
  });

  it('calls onSelectWord with original word when "use anyway" is clicked', () => {
    const onSelectWord = vi.fn();
    render(
      <SpellCheckDialog
        {...defaultProps}
        onSelectWord={onSelectWord}
        spellCheckData={mockSpellCheckData}
      />
    );
    fireEvent.click(screen.getByRole('button', { name: /tset/i }));
    expect(onSelectWord).toHaveBeenCalledWith('tset');
  });

  it('shows no suggestions message when alternatives are empty', () => {
    render(
      <SpellCheckDialog
        {...defaultProps}
        spellCheckData={{
          originalWord: 'xyz',
          isCorrect: false,
          suggestedWord: null,
          alternatives: [],
        }}
      />
    );
    expect(screen.getByText(/no suggestions/i)).toBeInTheDocument();
  });

  it('deduplicates suggested word from alternatives list', () => {
    render(
      <SpellCheckDialog
        {...defaultProps}
        spellCheckData={{
          originalWord: 'tset',
          isCorrect: false,
          suggestedWord: 'test',
          alternatives: ['test', 'test', 'set'],
        }}
      />
    );
    // "test" should appear only once as a clickable button (+ once in "use anyway")
    const testButtons = screen.getAllByRole('button').filter(
      b => b.textContent?.trim().startsWith('test')
    );
    // 1 for the recommended "test" button
    expect(testButtons.length).toBe(1);
  });
});
