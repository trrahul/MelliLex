import { makeObservable, observable, action, computed, runInAction } from 'mobx';
import { api } from '../services/api';
import type { WordHistory } from '../types';
import { getErrorMessage, ERROR_MESSAGES } from '../utils/errorHandler';
import { BaseStore } from './BaseStore';

export class HistoryStore extends BaseStore {
  items: WordHistory[] = [];
  searchQuery: string = '';
  private readonly historyLimit = 50;

  constructor() {
    super();
    makeObservable(this, {
      items: observable,
      searchQuery: observable,
      setSearchQuery: action,
      recentWords: computed,
      filteredItems: computed,
    });
  }

  setSearchQuery(query: string) {
    this.searchQuery = query;
  }

  async addToHistory(_word: string): Promise<void> {
    try {
      const updatedHistory = await api.getHistory(this.historyLimit);
      runInAction(() => {
        this.items = updatedHistory;
      });
    } catch (err) {
      runInAction(() => {
        this.error = getErrorMessage(err, ERROR_MESSAGES.LOAD_HISTORY_FAILED);
      });
    }
  }

  async loadHistory(limit?: number) {
    await this.executeAsync(
      () => api.getHistory(limit),
      ERROR_MESSAGES.LOAD_HISTORY_FAILED,
      (history) => {
        this.items = history;
      }
    );
  }

  async clearHistory() {
    try {
      await api.clearHistory();
      
      runInAction(() => {
        this.items = [];
      });
    } catch (err) {
      runInAction(() => {
        this.error = getErrorMessage(err, ERROR_MESSAGES.CLEAR_HISTORY_FAILED);
      });
    }
  }

  async deleteItem(id: string) {
    try {
      await api.deleteHistoryItem(id);
      
      runInAction(() => {
        this.items = this.items.filter(item => item.id !== id);
      });
    } catch (err) {
      runInAction(() => {
        this.error = getErrorMessage(err, ERROR_MESSAGES.DELETE_ITEM_FAILED);
      });
    }
  }

  get recentWords() {
    return this.items.slice(0, 10);
  }

  get filteredItems() {
    const query = this.searchQuery.trim().toLowerCase();
    if (!query) {
      return this.items;
    }

    return this.items.filter(item =>
      item.word.toLowerCase().includes(query) ||
      item.aiProvider.toLowerCase().includes(query)
    );
  }
}
