import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';
import { SUPPORTED_LANGUAGES } from '../../types';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '../ui/select';

interface LanguageSectionProps {
  value: string;
  onChange: (value: string) => void;
}

export const LanguageSection = observer(({ value, onChange }: LanguageSectionProps) => {
  const { t } = useTranslation();
  
  return (
    <div>
      <label className="text-sm font-medium text-foreground mb-2 block">
        {t('settings.general.explanationLanguage')}
      </label>
      <Select
        value={value}
        onValueChange={onChange}
      >
        <SelectTrigger>
          <SelectValue placeholder={t('settings.general.selectExplanationLanguage')} />
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
        {t('settings.general.explanationLanguageDesc')}
      </p>
    </div>
  );
});
