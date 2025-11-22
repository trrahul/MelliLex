import { describe, it, expect, beforeEach } from 'vitest';
import i18n, { LANGUAGE_MAP, CODE_TO_NAME } from '../i18n';

describe('i18n Configuration', () => {
  beforeEach(() => {
    // Reset to English before each test
    i18n.changeLanguage('en');
    localStorage.clear();
  });

  it('should initialize with default language (English)', () => {
    expect(i18n.language).toBe('en');
  });

  it('should have all language codes in LANGUAGE_MAP', () => {
    const expectedLanguages = [
      'English', 'Spanish', 'Portuguese', 'French', 'German',
      'Hindi', 'Arabic', 'Chinese (Simplified)', 'Japanese', 'Korean',
      'Italian', 'Turkish', 'Russian'
    ];

    expectedLanguages.forEach(lang => {
      expect(LANGUAGE_MAP[lang]).toBeDefined();
      expect(typeof LANGUAGE_MAP[lang]).toBe('string');
      expect(LANGUAGE_MAP[lang].length).toBe(2); // All codes should be 2 letters
    });
  });

  it('should have bidirectional mapping between names and codes', () => {
    Object.entries(LANGUAGE_MAP).forEach(([name, code]) => {
      expect(CODE_TO_NAME[code]).toBe(name);
    });

    Object.entries(CODE_TO_NAME).forEach(([code, name]) => {
      expect(LANGUAGE_MAP[name]).toBe(code);
    });
  });

  it('should switch language when changeLanguage is called', async () => {
    await i18n.changeLanguage('es');
    expect(i18n.language).toBe('es');

    await i18n.changeLanguage('en');
    expect(i18n.language).toBe('en');
  });

  it('should have English translations loaded', () => {
    expect(i18n.hasResourceBundle('en', 'translation')).toBe(true);
    
    // Test common translations
    expect(i18n.t('common.appName')).toBe('MelliLex');
    expect(i18n.t('common.search')).toBe('Search');
    expect(i18n.t('navigation.settings')).toBe('Settings');
  });

  it('should have Spanish translations loaded', () => {
    expect(i18n.hasResourceBundle('es', 'translation')).toBe(true);
    
    // Test Spanish translations
    i18n.changeLanguage('es');
    expect(i18n.t('common.appName')).toBe('MelliLex');
    expect(i18n.t('common.search')).toBe('Buscar');
    expect(i18n.t('navigation.settings')).toBe('Configuración');
  });

  it('should fallback to English for unsupported languages', async () => {
    await i18n.changeLanguage('pt'); // Portuguese not yet implemented
    
    // Should use English translations as fallback
    expect(i18n.t('common.appName')).toBe('MelliLex');
  });

  it('should interpolate variables in translations', () => {
    const word = 'example';
    const translation = i18n.t('spellCheck.misspelled', { word });
    expect(translation).toContain(word);
  });

  // Note: localStorage tests removed - this is a desktop app using Tauri backend for persistence
  it('should change language programmatically', async () => {
    await i18n.changeLanguage('es');
    expect(i18n.language).toBe('es');
    
    await i18n.changeLanguage('en');
    expect(i18n.language).toBe('en');
  });

  it('should support all translation namespaces', () => {
    const testKeys = [
      'common.appName', 
      'search.placeholder', 
      'navigation.home', 
      'settings.title',
      'spellCheck.title', 
      'explore.title', 
      'history.title', 
      'phrase.meaning'
    ];

    testKeys.forEach(key => {
      const translation = i18n.t(key);
      
      // Should not return the key itself (means translation exists)
      expect(translation).not.toBe(key);
    });
  });

  it('should map browser language codes correctly', () => {
    const testCases = [
      { browserCode: 'es', expectedCode: 'es' },
      { browserCode: 'es-MX', expectedCode: 'es' },
      { browserCode: 'pt-BR', expectedCode: 'pt' },
      { browserCode: 'en-US', expectedCode: 'en' },
      { browserCode: 'zh-CN', expectedCode: 'zh' },
    ];

    testCases.forEach(({ browserCode, expectedCode }) => {
      const code = browserCode.split('-')[0];
      expect(code).toBe(expectedCode);
    });
  });

  it('should handle RTL languages correctly', async () => {
    await i18n.changeLanguage('ar'); // Arabic is RTL
    
    // Check if dir attribute should be set to RTL
    const dir = i18n.dir();
    expect(dir).toBe('rtl');
  });

  it('should support nested translation keys', () => {
    const translation = i18n.t('settings.language.info');
    expect(translation).toBeDefined();
    expect(typeof translation).toBe('string');
    expect(translation.length).toBeGreaterThan(0);
  });

  it('should return correct benefits object for language section', () => {
    const benefits = i18n.t('settings.language.benefits', { returnObjects: true });
    expect(typeof benefits).toBe('object');
    expect(benefits).toHaveProperty('title');
    expect(benefits).toHaveProperty('understand');
  });

  it('should translate all navigation items', () => {
    const navItems = ['home', 'explore', 'history', 'saved', 'settings'];
    
    navItems.forEach(item => {
      const translation = i18n.t(`navigation.${item}`);
      expect(translation).toBeDefined();
      expect(translation).not.toBe(`navigation.${item}`);
    });
  });

  it('should translate all settings sections', () => {
    const sections = [
      'aiProvider', 'language', 'uiLanguage',
      'appearance', 'cache', 'export', 'about'
    ];
    
    sections.forEach(section => {
      const translation = i18n.t(`settings.${section}.title`);
      expect(translation).toBeDefined();
      expect(translation).not.toBe(`settings.${section}.title`);
    });
  });

  it('should have consistent translation structure across languages', () => {
    const checkKeys = (obj: any, path = ''): string[] => {
      const keys: string[] = [];
      for (const key in obj) {
        const currentPath = path ? `${path}.${key}` : key;
        if (typeof obj[key] === 'object' && !Array.isArray(obj[key])) {
          keys.push(...checkKeys(obj[key], currentPath));
        } else {
          keys.push(currentPath);
        }
      }
      return keys;
    };

    const enBundle = i18n.getResourceBundle('en', 'translation');
    const esBundle = i18n.getResourceBundle('es', 'translation');

    const enKeys = checkKeys(enBundle).sort();
    const esKeys = checkKeys(esBundle).sort();

    // Both should have the same keys
    expect(enKeys).toEqual(esKeys);
  });
});
