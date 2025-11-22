import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';

import en from './locales/en.json';
import es from './locales/es.json';
import de from './locales/de.json';
import fr from './locales/fr.json';
import it from './locales/it.json';
import ja from './locales/ja.json';
import ko from './locales/ko.json';
import pt from './locales/pt.json';
import ru from './locales/ru.json';
import tr from './locales/tr.json';
import zh from './locales/zh.json';

// Language code mapping for supported languages
const LANGUAGE_MAP: Record<string, string> = {
  'English': 'en',
  'Spanish': 'es',
  'Portuguese': 'pt',
  'French': 'fr',
  'German': 'de',
  'Hindi': 'hi',
  'Arabic': 'ar',
  'Chinese (Simplified)': 'zh',
  'Japanese': 'ja',
  'Korean': 'ko',
  'Italian': 'it',
  'Turkish': 'tr',
  'Russian': 'ru',
};

// Reverse mapping
const CODE_TO_NAME: Record<string, string> = Object.fromEntries(
  Object.entries(LANGUAGE_MAP).map(([name, code]) => [code, name])
);

// Language resources
const resources = {
  en: { translation: en },
  es: { translation: es },
  pt: { translation: pt },
  fr: { translation: fr },
  de: { translation: de },
  hi: { translation: en }, // Hindi not yet translated
  ar: { translation: en }, // Arabic not yet translated
  zh: { translation: zh },
  ja: { translation: ja },
  ko: { translation: ko },
  it: { translation: it },
  tr: { translation: tr },
  ru: { translation: ru },
};

i18n
  .use(initReactI18next)
  .init({
    resources,
    fallbackLng: 'en',
    lng: 'en', // Default to English, will be overridden by App.tsx from backend settings
    interpolation: {
      escapeValue: false, // React already escapes
    },
    react: {
      useSuspense: false, // Disable suspense to avoid potential issues with language switching
    },
  });

export { LANGUAGE_MAP, CODE_TO_NAME };
export default i18n;
