import type { TypographyOption } from '../types';

export const DEFAULT_TYPOGRAPHY_OPTION: TypographyOption = 'classic';

interface TypographyPreset {
  uiFont: string;
  contentFont: string;
}

export const TYPOGRAPHY_PRESETS: Record<TypographyOption, TypographyPreset> = {
  modern: {
    uiFont: 'Inter, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
    contentFont: 'Inter, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
  },
  classic: {
    uiFont: 'Inter, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
    contentFont: 'Crimson Pro, "Palatino Linotype", Palatino, Georgia, serif',
  },
  friendly: {
    uiFont: 'Plus Jakarta Sans, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
    contentFont: 'Merriweather, "Palatino Linotype", Palatino, Georgia, serif',
  },
};

export const applyTypographyOption = (option: TypographyOption | undefined): void => {
  if (typeof document === 'undefined') {
    return;
  }

  const preset = option ? TYPOGRAPHY_PRESETS[option] : null;
  const resolved = preset ?? TYPOGRAPHY_PRESETS[DEFAULT_TYPOGRAPHY_OPTION];
  const root = document.documentElement;

  root.style.setProperty('--font-ui', resolved.uiFont);
  root.style.setProperty('--font-content', resolved.contentFont);
  root.dataset.typography = option ?? DEFAULT_TYPOGRAPHY_OPTION;
};
