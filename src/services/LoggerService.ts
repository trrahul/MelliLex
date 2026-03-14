import * as log from '@tauri-apps/plugin-log';

/**
 * Interface for logging operations.
 * Enables dependency inversion and allows swapping logging implementations.
 */
export interface ILogger {
  info(message: string, context?: Record<string, any>): void;

  error(message: string, error?: Error, context?: Record<string, any>): void;

  debug(message: string, context?: Record<string, any>): void;
}

/**
 * Production implementation using Tauri's logging plugin.
 * Formats messages with context for structured logging.
 */
export class TauriLoggerService implements ILogger {
  info(message: string, context?: Record<string, any>): void {
    log.info(context ? `${message} ${JSON.stringify(context)}` : message);
  }

  error(message: string, error?: Error, context?: Record<string, any>): void {
    const errorMsg = error ? `${message}: ${error.message}` : message;
    log.error(context ? `${errorMsg} ${JSON.stringify(context)}` : errorMsg);
  }

  debug(message: string, context?: Record<string, any>): void {
    log.debug(context ? `${message} ${JSON.stringify(context)}` : message);
  }
}
