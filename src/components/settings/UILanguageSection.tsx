import { useEffect } from 'react';
import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';
import { useStores } from '../../stores/RootStore';
import { SUPPORTED_LANGUAGES } from '../../types';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '../ui/select';

interface UILanguageSectionProps {
  value: string;
  onChange: (value: string) => void;
}

export const UILanguageSection = observer(({ value, onChange }: UILanguageSectionProps) => {
  const { t } = useTranslation();
  const { settingsStore } = useStores();
  
  const wasAutoDetected = !settingsStore.settings.uiLanguage;

  useEffect(() => {
    // Auto-detect language on first run if not set
    if (!settingsStore.settings.uiLanguage) {
      const browserLang = navigator.language || navigator.languages?.[0] || 'en';
      const detectedLangCode = browserLang.split('-')[0]; // e.g., 'es-MX' -> 'es'
      
      // Map browser language code to our language names
      const languageMapping: Record<string, string> = {
        'en': 'English',
        'es': 'Spanish',
        'pt': 'Portuguese',
        'fr': 'French',
        'de': 'German',
        'hi': 'Hindi',
        'ar': 'Arabic',
        'zh': 'Chinese (Simplified)',
        'ja': 'Japanese',
        'ko': 'Korean',
        'it': 'Italian',
        'tr': 'Turkish',
        'ru': 'Russian',
      };
      
      const detectedLanguage = languageMapping[detectedLangCode] || 'English';
      
      // Auto-set if different from English
      if (detectedLanguage !== 'English') {
        settingsStore.updateSettings({
          uiLanguage: detectedLanguage,
        });
      }
    }
  }, [settingsStore]);

  return (
    <div>
      <label className="text-sm font-medium text-foreground mb-2 block">
        {t('settings.general.uiLanguage')}
      </label>
      <Select
        value={value}
        onValueChange={onChange}
      >
        <SelectTrigger>
          <SelectValue placeholder={t('settings.general.selectUiLanguage')} />
        </SelectTrigger>
        <SelectContent>
          {SUPPORTED_LANGUAGES.map((lang) => (
            <SelectItem key={lang.code} value={lang.name}>
              {lang.nativeName} ({lang.name})
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      <p className="text-xs text-muted-foreground mt-1.5">
        {t('settings.general.uiLanguageDesc')}
        {wasAutoDetected && <span> · {t('settings.general.autoDetected')}</span>}
      </p>
    </div>
  );
});
