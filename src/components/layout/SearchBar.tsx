import { useState, useEffect, useMemo } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';
import { Search, X } from 'lucide-react';
import { useStores } from '../../stores/RootStore';
import { SpellCheckDialog } from '../SpellCheckDialog';
import type { SpellCheckResponse } from '../../types';
import { createSearchStrategy } from '../../services/search/SearchContextStrategy';

export const SearchBar = observer(() => {
  const { t } = useTranslation();
  const location = useLocation();
  const navigate = useNavigate();
  const { searchCoordinator, progressiveWordStore, historyStore } = useStores();
  const [searchQuery, setSearchQuery] = useState('');
  const [spellCheckData, setSpellCheckData] = useState<SpellCheckResponse | null>(null);
  const [showSpellCheckDialog, setShowSpellCheckDialog] = useState(false);

  const pathname = location.pathname;
  const isSettingsPage = pathname === '/settings';

  const searchStrategy = useMemo(
    () =>
      createSearchStrategy({
        pathname,
        navigate,
        searchCoordinator,
        progressiveWordStore,
        historyStore,
      }),
    [
      pathname,
      navigate,
      searchCoordinator,
      progressiveWordStore,
      historyStore,
    ]
  );

  // Sync search query with current word when it changes
  // Keeps text input aligned with active word coming from other screens without overriding manual edits
  useEffect(() => {
    if (searchStrategy.type !== 'word') {
      return;
    }

    setSearchQuery(progressiveWordStore.currentWord || '');
  }, [searchStrategy.type, progressiveWordStore.currentWord]);

  useEffect(() => {
    if (searchStrategy.type !== 'history') {
      return;
    }

    setSearchQuery(historyStore.searchQuery);
  }, [searchStrategy.type, historyStore.searchQuery]);

  useEffect(() => {
    if (!searchStrategy.requiresSpellCheck) {
      setShowSpellCheckDialog(false);
      setSpellCheckData(null);
    }
  }, [searchStrategy.requiresSpellCheck]);

  const getPlaceholder = () => {
    return t(searchStrategy.placeholderKey);
  };

  const handleInputChange = (value: string) => {
    setSearchQuery(value);
    searchStrategy.handleInputChange?.(value);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    const trimmed = searchQuery.trim();
    if (!trimmed || searchStrategy.isBusy()) return;

    if (searchStrategy.requiresSpellCheck) {
      const spellCheck = await searchCoordinator.checkSpelling(trimmed);
      if (spellCheck) {
        setSpellCheckData(spellCheck);
        setShowSpellCheckDialog(true);
        return;
      }
    }

    await searchStrategy.performSearch(trimmed);
    if (searchStrategy.clearOnSubmit) {
      setSearchQuery('');
    }
  };

  const handleSelectWord = async (selectedWord: string) => {
    setShowSpellCheckDialog(false);
    setSpellCheckData(null);
    if (searchStrategy.isBusy()) {
      return;
    }

    if (searchStrategy.performSuggestion) {
      await searchStrategy.performSuggestion(selectedWord);
    } else {
      await searchStrategy.performSearch(selectedWord);
    }

    if (searchStrategy.clearOnSubmit) {
      setSearchQuery('');
    }
  };

  const handleCancelSpellCheck = () => {
    setShowSpellCheckDialog(false);
    setSpellCheckData(null);
  };

  const handleClear = () => {
    setSearchQuery('');
    searchStrategy.handleInputChange?.('');
  };

  const canClearSearch = !isSettingsPage && searchQuery.length > 0;

  return (
    <>
      <form 
        onSubmit={handleSubmit} 
        className={`relative flex-1 transition-all duration-300 ${
          isSettingsPage 
            ? 'max-w-0 opacity-0 -translate-x-4 pointer-events-none' 
            : 'max-w-lg opacity-100 translate-x-0'
        }`}
      >
        <input
          type="text"
          value={searchQuery}
          onChange={(e) => handleInputChange(e.target.value)}
          placeholder={getPlaceholder()}
          className="w-full px-4 py-2 pr-16 border-2 border-input bg-background rounded-lg text-sm outline-none focus:border-ring transition-colors"
          disabled={isSettingsPage}
        />
        {canClearSearch && (
          <button
            type="button"
            onClick={handleClear}
            className="absolute right-10 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground transition-colors"
            aria-label={t('search.clear')}
          >
            <X className="w-4 h-4" />
          </button>
        )}
        <button
          type="submit"
          className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground transition-colors"
          disabled={isSettingsPage || searchStrategy.isBusy()}
          aria-label={t('search.submit')}
        >
          <Search className="w-4 h-4" />
        </button>
      </form>

      <SpellCheckDialog
        open={searchStrategy.requiresSpellCheck && showSpellCheckDialog}
        spellCheckData={spellCheckData}
        onSelectWord={handleSelectWord}
        onCancel={handleCancelSpellCheck}
      />
    </>
  );
});
