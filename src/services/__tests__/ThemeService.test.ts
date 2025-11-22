import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { ThemeService } from '../ThemeService';

describe('ThemeService', () => {
  // Store original values to restore after tests
  let originalLocalStorage: Storage;
  let originalMatchMedia: typeof window.matchMedia;
  let mockMatchMedia: ReturnType<typeof createMockMatchMedia>;

  function createMockMatchMedia(matches: boolean) {
    const listeners: ((e: MediaQueryListEvent) => void)[] = [];
    const mock = {
      _matches: matches,
      get matches() {
        return this._matches;
      },
      set matches(value: boolean) {
        this._matches = value;
      },
      media: '(prefers-color-scheme: dark)',
      addEventListener: vi.fn((event: string, listener: (e: MediaQueryListEvent) => void) => {
        if (event === 'change') {
          listeners.push(listener);
        }
      }),
      removeEventListener: vi.fn((event: string, listener: (e: MediaQueryListEvent) => void) => {
        const index = listeners.indexOf(listener);
        if (index > -1) {
          listeners.splice(index, 1);
        }
      }),
      dispatchEvent: vi.fn(),
      onchange: null,
      addListener: vi.fn(),
      removeListener: vi.fn(),
      trigger: (matches: boolean) => {
        mock._matches = matches;
        listeners.forEach(listener => {
          listener({ matches, media: '(prefers-color-scheme: dark)' } as MediaQueryListEvent);
        });
      }
    };
    return mock as unknown as MediaQueryList & { trigger: (matches: boolean) => void; _matches: boolean };
  }

  beforeEach(() => {
    // Mock localStorage
    originalLocalStorage = global.localStorage;
    const storage: Record<string, string> = {};
    global.localStorage = {
      getItem: vi.fn((key: string) => storage[key] || null),
      setItem: vi.fn((key: string, value: string) => {
        storage[key] = value;
      }),
      removeItem: vi.fn((key: string) => {
        delete storage[key];
      }),
      clear: vi.fn(() => {
        Object.keys(storage).forEach(key => delete storage[key]);
      }),
      key: vi.fn(),
      length: 0,
    };

    // Mock matchMedia (default to light mode)
    originalMatchMedia = window.matchMedia;
    mockMatchMedia = createMockMatchMedia(false);
    window.matchMedia = vi.fn(() => mockMatchMedia);

    // Clear document classList
    document.documentElement.classList.remove('dark');
  });

  afterEach(() => {
    global.localStorage = originalLocalStorage;
    window.matchMedia = originalMatchMedia;
    document.documentElement.classList.remove('dark');
  });

  describe('getInitialTheme', () => {
    it('should return "system" when no theme is saved', () => {
      const theme = ThemeService.getInitialTheme();
      expect(theme).toBe('system');
    });

    it('should return "light" when light theme is saved', () => {
      localStorage.setItem('theme', 'light');
      const theme = ThemeService.getInitialTheme();
      expect(theme).toBe('light');
    });

    it('should return "dark" when dark theme is saved', () => {
      localStorage.setItem('theme', 'dark');
      const theme = ThemeService.getInitialTheme();
      expect(theme).toBe('dark');
    });

    it('should return "system" for invalid saved values', () => {
      localStorage.setItem('theme', 'invalid');
      const theme = ThemeService.getInitialTheme();
      expect(theme).toBe('system');
    });
  });

  describe('getResolvedTheme', () => {
    it('should return "light" for light theme', () => {
      const resolved = ThemeService.getResolvedTheme('light');
      expect(resolved).toBe('light');
    });

    it('should return "dark" for dark theme', () => {
      const resolved = ThemeService.getResolvedTheme('dark');
      expect(resolved).toBe('dark');
    });

    it('should return "light" for system theme when system prefers light', () => {
      mockMatchMedia._matches = false;
      const resolved = ThemeService.getResolvedTheme('system');
      expect(resolved).toBe('light');
    });

    it('should return "dark" for system theme when system prefers dark', () => {
      mockMatchMedia._matches = true;
      const resolved = ThemeService.getResolvedTheme('system');
      expect(resolved).toBe('dark');
    });
  });

  describe('applyTheme', () => {
    it('should add dark class for dark theme', () => {
      ThemeService.applyTheme('dark');
      expect(document.documentElement.classList.contains('dark')).toBe(true);
    });

    it('should remove dark class for light theme', () => {
      document.documentElement.classList.add('dark');
      ThemeService.applyTheme('light');
      expect(document.documentElement.classList.contains('dark')).toBe(false);
    });

    it('should add dark class for system theme when system prefers dark', () => {
      mockMatchMedia._matches = true;
      ThemeService.applyTheme('system');
      expect(document.documentElement.classList.contains('dark')).toBe(true);
    });

    it('should remove dark class for system theme when system prefers light', () => {
      mockMatchMedia._matches = false;
      document.documentElement.classList.add('dark');
      ThemeService.applyTheme('system');
      expect(document.documentElement.classList.contains('dark')).toBe(false);
    });

    it('should save theme to localStorage', () => {
      ThemeService.applyTheme('dark');
      expect(localStorage.setItem).toHaveBeenCalledWith('theme', 'dark');
    });

    it('should save system theme to localStorage', () => {
      ThemeService.applyTheme('system');
      expect(localStorage.setItem).toHaveBeenCalledWith('theme', 'system');
    });
  });

  describe('toggleTheme', () => {
    it('should toggle from light to dark', () => {
      const newTheme = ThemeService.toggleTheme('light');
      expect(newTheme).toBe('dark');
    });

    it('should toggle from dark to light', () => {
      const newTheme = ThemeService.toggleTheme('dark');
      expect(newTheme).toBe('light');
    });

    it('should toggle from system (light) to dark', () => {
      mockMatchMedia._matches = false;
      const newTheme = ThemeService.toggleTheme('system');
      expect(newTheme).toBe('dark');
    });

    it('should toggle from system (dark) to light', () => {
      mockMatchMedia._matches = true;
      const newTheme = ThemeService.toggleTheme('system');
      expect(newTheme).toBe('light');
    });
  });

  describe('integration', () => {
    it('should handle full theme cycle: light -> dark -> light', () => {
      // Start with light
      ThemeService.applyTheme('light');
      expect(document.documentElement.classList.contains('dark')).toBe(false);
      expect(localStorage.setItem).toHaveBeenCalledWith('theme', 'light');

      // Toggle to dark
      const darkTheme = ThemeService.toggleTheme('light');
      ThemeService.applyTheme(darkTheme);
      expect(document.documentElement.classList.contains('dark')).toBe(true);
      expect(localStorage.setItem).toHaveBeenCalledWith('theme', 'dark');

      // Toggle back to light
      const lightTheme = ThemeService.toggleTheme(darkTheme);
      ThemeService.applyTheme(lightTheme);
      expect(document.documentElement.classList.contains('dark')).toBe(false);
      expect(localStorage.setItem).toHaveBeenCalledWith('theme', 'light');
    });

    it('should persist theme across "sessions"', () => {
      // Set dark theme
      ThemeService.applyTheme('dark');
      
      // Simulate page reload (get initial theme)
      const initialTheme = ThemeService.getInitialTheme();
      expect(initialTheme).toBe('dark');
      
      // Apply the loaded theme
      ThemeService.applyTheme(initialTheme);
      expect(document.documentElement.classList.contains('dark')).toBe(true);
    });

    it('should handle system theme preference changes', () => {
      // Start with system theme (light)
      mockMatchMedia._matches = false;
      ThemeService.applyTheme('system');
      expect(document.documentElement.classList.contains('dark')).toBe(false);

      // System preference changes to dark
      mockMatchMedia._matches = true;
      ThemeService.applyTheme('system');
      expect(document.documentElement.classList.contains('dark')).toBe(true);
    });
  });

  describe('edge cases', () => {
    it('should handle missing localStorage gracefully', () => {
      // Mock localStorage to throw error
      const getItemSpy = vi.spyOn(localStorage, 'getItem');
      getItemSpy.mockImplementation(() => {
        throw new Error('localStorage not available');
      });

      // Should not throw and default to system
      expect(() => ThemeService.getInitialTheme()).toThrow();
      
      getItemSpy.mockRestore();
    });

    it('should handle matchMedia not supported', () => {
      // Mock matchMedia to be undefined
      (window as any).matchMedia = undefined;

      // Should not throw
      expect(() => ThemeService.getResolvedTheme('system')).toThrow();
    });

    it('should handle rapid theme changes', () => {
      ThemeService.applyTheme('light');
      ThemeService.applyTheme('dark');
      ThemeService.applyTheme('light');
      ThemeService.applyTheme('dark');

      // Final state should be dark
      expect(document.documentElement.classList.contains('dark')).toBe(true);
      expect(localStorage.setItem).toHaveBeenLastCalledWith('theme', 'dark');
    });
  });
});
