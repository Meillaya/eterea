// Reactive stores for bookmark state using Svelte 5 runes

import type { Bookmark, BookmarkStats } from '$lib/types';

// Bookmarks list
function createBookmarksStore() {
  let items = $state<Bookmark[]>([]);
  
  return {
    get value() { return items; },
    set(newItems: Bookmark[]) { items = newItems; },
    append(newItems: Bookmark[]) { items = [...items, ...newItems]; },
    prepend(newItems: Bookmark[]) { items = [...newItems, ...items]; },
    remove(id: string) { items = items.filter(b => b.id !== id); },
    clear() { items = []; }
  };
}

// Search query
function createSearchStore() {
  let query = $state('');
  
  return {
    get value() { return query; },
    set(newQuery: string) { query = newQuery; }
  };
}

// Selected tag filter
function createTagStore() {
  let tag = $state<string | null>(null);
  
  return {
    get value() { return tag; },
    set(newTag: string | null) { tag = newTag; },
    clear() { tag = null; }
  };
}

// Loading state
function createLoadingStore() {
  let loading = $state(false);
  
  return {
    get value() { return loading; },
    set(newLoading: boolean) { loading = newLoading; }
  };
}

// Stats
function createStatsStore() {
  let data = $state<BookmarkStats | null>(null);
  
  return {
    get value() { return data; },
    set(newStats: BookmarkStats | null) { data = newStats; }
  };
}

// All tags
function createTagsStore() {
  let tags = $state<[string, number][]>([]);
  
  return {
    get value() { return tags; },
    set(newTags: [string, number][]) { tags = newTags; }
  };
}

// View mode (all, favorites, recent)
type ViewMode = 'all' | 'favorites' | 'recent';

function createViewModeStore() {
  let mode = $state<ViewMode>('all');
  
  return {
    get value() { return mode; },
    set(newMode: ViewMode) { mode = newMode; }
  };
}

// Date range filter
function createDateRangeStore() {
  let range = $state<{ from: string | null; to: string | null }>({ from: null, to: null });
  
  return {
    get value() { return range; },
    set(from: string | null, to: string | null) { range = { from, to }; },
    clear() { range = { from: null, to: null }; }
  };
}

// Export stores
export const bookmarks = createBookmarksStore();
export const searchQuery = createSearchStore();
export const selectedTag = createTagStore();
export const isLoading = createLoadingStore();
export const stats = createStatsStore();
export const allTags = createTagsStore();
export const viewMode = createViewModeStore();
// Layout mode for bookmark rendering
export type LayoutMode = 'default' | 'cards' | 'compact';

function resolveInitialLayout(): LayoutMode {
  if (typeof window === 'undefined') return 'default';
  const stored = window.localStorage.getItem('eterea:layout');
  if (stored === 'cards' || stored === 'compact' || stored === 'default') {
    return stored;
  }
  return 'default';
}

function createLayoutStore() {
  let layout = $state<LayoutMode>(resolveInitialLayout());
  
  return {
    get value() { return layout; },
    set(newLayout: LayoutMode) { 
      layout = newLayout; 
      if (typeof window !== 'undefined') {
        window.localStorage.setItem('eterea:layout', newLayout);
      }
    }
  };
}

// Feed meta (pagination)
type FeedState = {
  offset: number;
  limit: number;
  total: number;
  hasMore: boolean;
};

function createFeedMetaStore() {
  let state = $state<FeedState>({
    offset: 0,
    limit: 50,
    total: 0,
    hasMore: true
  });
  
  return {
    get value() { return state; },
    reset(limit = 50) {
      state = { offset: 0, limit, total: 0, hasMore: true };
    },
    update(partial: Partial<FeedState>) {
      state = { ...state, ...partial };
    }
  };
}

export const dateRange = createDateRangeStore();
export const layoutMode = createLayoutStore();
export const feedMeta = createFeedMetaStore();

