import { makeObservable, observable, computed, runInAction, action } from 'mobx';
import type { LoadingState } from '../types';

export abstract class BaseStore {
  loadingState: LoadingState = 'idle';
  error: string | null = null;

  constructor() {
    makeObservable<this, 'setLoading' | 'setSuccess' | 'setError' | 'setIdle'>(this, {
      loadingState: observable,
      error: observable,
      setLoading: action,
      setSuccess: action,
      setError: action,
      setIdle: action,
      isLoading: computed,
      hasError: computed,
      isSuccess: computed,
      isIdle: computed,
    });
  }

  protected setLoading(): void {
    this.loadingState = 'loading';
    this.error = null;
  }

  protected setSuccess(): void {
    this.loadingState = 'success';
  }

  protected setError(message: string): void {
    this.loadingState = 'error';
    this.error = message;
  }

  protected setIdle(): void {
    this.loadingState = 'idle';
    this.error = null;
  }

  protected async executeAsync<T>(
    operation: () => Promise<T>,
    errorMessage: string,
    onSuccess?: (result: T) => void
  ): Promise<T | null> {
    this.setLoading();

    try {
      const result = await operation();
      
      runInAction(() => {
        this.setSuccess();
        if (onSuccess) {
          onSuccess(result);
        }
      });

      return result;
    } catch (err) {
      runInAction(() => {
        this.setError(errorMessage);
      });
      return null;
    }
  }

  // Computed properties
  get isLoading(): boolean {
    return this.loadingState === 'loading';
  }

  get hasError(): boolean {
    return this.loadingState === 'error';
  }

  get isSuccess(): boolean {
    return this.loadingState === 'success';
  }

  get isIdle(): boolean {
    return this.loadingState === 'idle';
  }
}
