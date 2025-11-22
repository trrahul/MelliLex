import { runInAction } from 'mobx';
import type { ITimeoutService } from '../../services/TimeoutService';
import type { ILogger } from '../../services/LoggerService';

export interface ProgressiveSearchConfig {
  timeoutService: ITimeoutService;
  timeoutMs: number;
  isLoading: () => boolean;
  onTimeout: () => void;
  onError: (error: unknown) => void;
  invoke: () => Promise<void>;
  logger: ILogger;
  logPrefix: string;
}

export async function runProgressiveSearch(config: ProgressiveSearchConfig): Promise<void> {
  const {
    timeoutService,
    timeoutMs,
    isLoading,
    onTimeout,
    onError,
    invoke,
    logger,
    logPrefix,
  } = config;

  timeoutService.setTimeout(() => {
    if (isLoading()) {
      runInAction(onTimeout);
    }
  }, timeoutMs);

  try {
    logger.info(`${logPrefix} Invoking progressive search`);
    await invoke();
    logger.info(`${logPrefix} Progressive search command completed`);
  } catch (error) {
    runInAction(() => onError(error));
  } finally {
    timeoutService.clearTimeout();
  }
}
