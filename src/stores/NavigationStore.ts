import { makeAutoObservable } from 'mobx';
import { UI_CONFIG } from '../constants/events';

export interface NavigationEntry {
  word: string;
  timestamp: number;
  source?: string;
}

export class NavigationStore {
  history: NavigationEntry[] = [];
  currentIndex: number = -1;
  private maxHistorySize = UI_CONFIG.MAX_HISTORY_SIZE;

  constructor() {
    makeAutoObservable(this);
  }

 
  navigateTo(word: string, source?: string): void {
    if (this.currentIndex < this.history.length - 1) {
      this.history = this.history.slice(0, this.currentIndex + 1);
    }

    const entry: NavigationEntry = {
      word,
      timestamp: Date.now(),
      source,
    };

    this.history.push(entry);
    this.currentIndex = this.history.length - 1;

    if (this.history.length > this.maxHistorySize) {
      this.history = this.history.slice(-this.maxHistorySize);
      this.currentIndex = this.history.length - 1;
    }
  }

  goBack(): string | null {
    if (!this.canGoBack) return null;
    
    this.currentIndex--;
    return this.currentWord;
  }

  goForward(): string | null {
    if (!this.canGoForward) return null;
    
    this.currentIndex++;
    return this.currentWord;
  }

  get currentWord(): string | null {
    if (this.currentIndex < 0 || this.currentIndex >= this.history.length) {
      return null;
    }
    return this.history[this.currentIndex].word;
  }

  get breadcrumbTrail(): NavigationEntry[] {
    if (this.currentIndex < 0) return [];
    return this.history.slice(0, this.currentIndex + 1);
  }

  get pathDepth(): number {
    return this.breadcrumbTrail.length;
  }

  get uniqueWordsExplored(): number {
    const uniqueWords = new Set(this.history.map(entry => entry.word.toLowerCase()));
    return uniqueWords.size;
  }

  get canGoBack(): boolean {
    return this.currentIndex > 0;
  }

  get canGoForward(): boolean {
    return this.currentIndex < this.history.length - 1;
  }

  clear(): void {
    this.history = [];
    this.currentIndex = -1;
  }

  getNavigationPath(): string[] {
    return this.breadcrumbTrail.map(entry => entry.word);
  }

  getNavigationEdges(): Array<{ from: string; to: string; timestamp: number }> {
    const edges: Array<{ from: string; to: string; timestamp: number }> = [];
    
    for (let i = 1; i < this.breadcrumbTrail.length; i++) {
      const prev = this.breadcrumbTrail[i - 1];
      const current = this.breadcrumbTrail[i];
      
      edges.push({
        from: prev.word,
        to: current.word,
        timestamp: current.timestamp,
      });
    }
    
    return edges;
  }
}
