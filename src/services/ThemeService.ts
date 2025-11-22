import { useState, useEffect } from 'react';

export type Theme = 'light' | 'dark' | 'system';

/**
 * Service for managing application theme.
 * 
 * Responsibilities:
 * - Load theme from localStorage
 * - Apply theme to DOM
 * - Persist theme changes
 * - Handle system theme preference
 * 
 * Benefits:
 * - Single source of truth for theme logic
 * - Easy to test theme behavior
 * - Reusable across components
 */
export class ThemeService {
  private static readonly STORAGE_KEY = 'theme';

  static getInitialTheme(): Theme {
    const saved = localStorage.getItem(this.STORAGE_KEY);
    if (saved === 'light' || saved === 'dark') {
      return saved;
    }
    return 'system';
  }

  static getResolvedTheme(theme: Theme): 'light' | 'dark' {
    if (theme === 'system') {
      return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    }
    return theme;
  }

  static applyTheme(theme: Theme): void {
    const resolved = this.getResolvedTheme(theme);
    
    if (resolved === 'dark') {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
    
    localStorage.setItem(this.STORAGE_KEY, theme);
  }

  static toggleTheme(currentTheme: Theme): Theme {
    const resolved = this.getResolvedTheme(currentTheme);
    return resolved === 'dark' ? 'light' : 'dark';
  }
}

/**
 * React hook for theme management.
 * 
 * @returns Current theme, resolved theme, and toggle function
 */
export function useTheme() {
  const [theme, setTheme] = useState<Theme>(() => ThemeService.getInitialTheme());

  useEffect(() => {
    ThemeService.applyTheme(theme);
  }, [theme]);

  // Listen for system theme changes when theme is 'system'
  useEffect(() => {
    if (theme !== 'system') return;

    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const handleChange = () => {
      ThemeService.applyTheme('system');
    };

    mediaQuery.addEventListener('change', handleChange);
    return () => mediaQuery.removeEventListener('change', handleChange);
  }, [theme]);

  const toggleTheme = () => {
    setTheme((current) => ThemeService.toggleTheme(current));
  };

  const resolvedTheme = ThemeService.getResolvedTheme(theme);

  return { theme, resolvedTheme, toggleTheme, setTheme };
}
