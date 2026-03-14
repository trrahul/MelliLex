import { useEffect, useRef, useState } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { Minus, Square, X } from 'lucide-react';
import { useTranslation } from 'react-i18next';

export function TitleBar() {
  const { t } = useTranslation();
  const [isMaximized, setIsMaximized] = useState(false);
  const appWindowRef = useRef(getCurrentWindow());
  const appWindow = appWindowRef.current;

  useEffect(() => {
    const checkMaximized = async () => {
      const maximized = await appWindow.isMaximized();
      setIsMaximized(maximized);
    };

    checkMaximized();

    const unlisten = appWindow.onResized(() => {
      checkMaximized();
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, []);

  const handleMinimize = () => appWindow.minimize();
  const handleMaximize = () => appWindow.toggleMaximize();
  const handleClose = () => appWindow.close();

  return (
    <div className="flex-shrink-0 relative z-[60] h-8 border-b select-none flex items-center justify-between chrome-panel chrome-panel--top">
      <div data-tauri-drag-region className="flex items-center gap-2 px-3 flex-1 h-full">
        <img src="/icon.png" alt="MelliLex" className="w-4 h-4" />
        <span className="text-sm font-semibold text-foreground">{t('common.appName')}</span>
      </div>

      <div className="flex items-center h-full">
        <button
          onClick={handleMinimize}
          className="h-full w-12 flex items-center justify-center hover:bg-muted/50 transition-colors cursor-default"
          aria-label={t('common.minimize')}
          type="button"
        >
          <Minus className="w-4 h-4" />
        </button>
        <button
          onClick={handleMaximize}
          className="h-full w-12 flex items-center justify-center hover:bg-muted/50 transition-colors cursor-default"
          aria-label={isMaximized ? t('common.restore') : t('common.maximize')}
          type="button"
        >
          <Square className="w-3.5 h-3.5" />
        </button>
        <button
          onClick={handleClose}
          className="h-full w-12 flex items-center justify-center hover:bg-destructive hover:text-destructive-foreground transition-colors cursor-default"
          aria-label={t('common.close')}
          type="button"
        >
          <X className="w-4 h-4" />
        </button>
      </div>
    </div>
  );
}
