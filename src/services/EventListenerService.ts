import { listen, UnlistenFn } from '@tauri-apps/api/event';

/**
 * Interface for event listener management.
 * Allows dependency inversion - stores depend on this abstraction, not concrete Tauri API.
 */
export interface IEventListener {
  listen<T>(event: string, handler: (payload: T) => void): Promise<void>;

  cleanup(): void;
}

/**
 * Production implementation of IEventListener using Tauri's event system.
 * Manages event listener lifecycle and automatic cleanup.
 */
export class EventListenerService implements IEventListener {
  private unlistenFns: UnlistenFn[] = [];

  async listen<T>(event: string, handler: (payload: T) => void): Promise<void> {
    const unlisten = await listen<T>(event, (e) => handler(e.payload));
    this.unlistenFns.push(unlisten);
  }

  cleanup(): void {
    this.unlistenFns.forEach((fn) => fn());
    this.unlistenFns = [];
  }
}

/**
 * Mock implementation for testing.
 * Allows synchronous event emission for unit tests.
 */
export class MockEventListener implements IEventListener {
  private handlers: Map<string, Array<(payload: any) => void>> = new Map();

  async listen<T>(event: string, handler: (payload: T) => void): Promise<void> {
    if (!this.handlers.has(event)) {
      this.handlers.set(event, []);
    }
    this.handlers.get(event)!.push(handler);
  }

  cleanup(): void {
    this.handlers.clear();
  }

  /**
   * Emit an event to trigger handlers (for testing).
   * @param event Event name
   * @param payload Event payload
   */
  emit<T>(event: string, payload: T): void {
    const handlers = this.handlers.get(event);
    if (handlers) {
      handlers.forEach((handler) => handler(payload));
    }
  }
}
