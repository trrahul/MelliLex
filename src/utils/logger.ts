import { info, debug, error, warn } from "@tauri-apps/plugin-log";

export const logger = {
  info: (message: string, ...args: unknown[]) => {
    console.info(message, ...args);
    info(message).catch(() => {/* ignore */});
  },
  
  debug: (message: string, ...args: unknown[]) => {
    console.debug(message, ...args);
    debug(message).catch(() => {/* ignore */});
  },
  
  warn: (message: string, ...args: unknown[]) => {
    console.warn(message, ...args);
    warn(message).catch(() => {/* ignore */});
  },
  
  error: (message: string, ...args: unknown[]) => {
    console.error(message, ...args);
    error(message).catch(() => {/* ignore */});
  },
  
  log: (message: string, ...args: unknown[]) => {
    console.log(message, ...args);
    info(message).catch(() => {/* ignore */});
  }
};
