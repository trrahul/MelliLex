import { makeAutoObservable } from 'mobx';
import type { LoadingState } from '../../../types';

export class MockWordSource {
  currentWord = '';
  loadingState: LoadingState = 'idle';

  constructor(initialWord = '') {
    makeAutoObservable(this);
    if (initialWord) {
      this.currentWord = initialWord;
    }
  }

  setWord(word: string) {
    this.currentWord = word;
  }

  startSearch(word: string) {
    this.currentWord = word;
    this.loadingState = 'loading';
  }

  finishSearch(state: LoadingState = 'success') {
    this.loadingState = state;
  }

  clear() {
    this.currentWord = '';
    this.loadingState = 'idle';
  }
}
