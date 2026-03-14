import { makeAutoObservable } from 'mobx';

/**
 * Store to track the last content page (Define or Explore)
 * so that navigation from History/Saved goes back to the correct page.
 */
export class LastPageStore {
  lastContentPage: '/' | '/explore' = '/';

  constructor() {
    makeAutoObservable(this);
    // Try to restore from sessionStorage
    const saved = sessionStorage.getItem('lastContentPage');
    if (saved === '/' || saved === '/explore') {
      this.lastContentPage = saved;
    }
  }

  setLastPage(page: '/' | '/explore'): void {
    this.lastContentPage = page;
    sessionStorage.setItem('lastContentPage', page);
  }

  get page(): '/' | '/explore' {
    return this.lastContentPage;
  }
}
