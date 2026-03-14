export interface ITimeoutService {
  setTimeout(callback: () => void, delay: number): void;
  clearTimeout(): void;
}

export class TimeoutService implements ITimeoutService {
  private timerId: ReturnType<typeof globalThis.setTimeout> | null = null;

  setTimeout(callback: () => void, delay: number): void {
    this.clearTimeout(); // Clear any existing timeout
    this.timerId = globalThis.setTimeout(callback, delay);
  }

  clearTimeout(): void {
    if (this.timerId) {
      globalThis.clearTimeout(this.timerId);
      this.timerId = null;
    }
  }
}

export class MockTimeoutService implements ITimeoutService {
  private callback: (() => void) | null = null;
  private isActive: boolean = false;

  setTimeout(callback: () => void, _delay: number): void {
    this.callback = callback;
    this.isActive = true;
  }

  clearTimeout(): void {
    this.callback = null;
    this.isActive = false;
  }

  triggerTimeout(): void {
    if (this.isActive && this.callback) {
      this.callback();
      this.clearTimeout();
    }
  }

  isTimeoutActive(): boolean {
    return this.isActive;
  }
}
