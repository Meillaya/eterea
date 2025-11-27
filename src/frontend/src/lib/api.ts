// API layer for communicating with Tauri backend

import { invoke } from '@tauri-apps/api/core';
import type { Bookmark, BookmarkStats, PaginatedResponse, LinkPreview } from './types';
import { bookmarks, isLoading, stats, allTags, feedMeta } from './stores/bookmarks.svelte';

// Check if running in Tauri
const isTauri = typeof window !== 'undefined' && '__TAURI__' in window;

// Mock data for development without Tauri
const mockBookmarks: Bookmark[] = [
  {
    id: '1',
    tweet_url: 'https://twitter.com/rustlang/status/123',
    content: 'Rust 2024 edition is here! 🦀 Check out the new features including improved async support and better error messages.',
    note_text: null,
    tweeted_at: '2024-05-01T14:30:00Z',
    imported_at: '2024-05-02T10:00:00Z',
    author_handle: 'rustlang',
    author_name: 'Rust Language',
    author_profile_url: 'https://twitter.com/rustlang',
    author_profile_image: 'https://pbs.twimg.com/profile_images/1234/rust_normal.jpg',
    tags: ['technology', 'programming'],
    comments: null,
    media: [],
    is_favorite: true
  },
  {
    id: '2',
    tweet_url: 'https://twitter.com/svaborern/status/456',
    content: 'Svelte 5 runes are a game changer. The new reactivity system is so much simpler to understand.',
    note_text: null,
    tweeted_at: '2024-04-28T09:15:00Z',
    imported_at: '2024-05-02T10:00:00Z',
    author_handle: 'svaborern',
    author_name: 'Svelte Enthusiast',
    author_profile_url: null,
    author_profile_image: null,
    tags: ['technology', 'webdev'],
    comments: null,
    media: [],
    is_favorite: false
  }
];

const mockStats: BookmarkStats = {
  total_bookmarks: 2,
  unique_authors: 2,
  unique_tags: 3,
  earliest_date: '2024-04-28T09:15:00Z',
  latest_date: '2024-05-01T14:30:00Z',
  top_tags: [['technology', 2], ['programming', 1], ['webdev', 1]]
};

type LoadOptions = {
  offset?: number;
  limit?: number;
  reset?: boolean;
};

export async function loadBookmarks(options: LoadOptions = {}): Promise<void> {
  const limit = options.limit ?? feedMeta.value.limit ?? 50;
  const offset = options.reset ? 0 : (options.offset ?? feedMeta.value.offset ?? 0);
  if (options.reset) {
    bookmarks.clear();
    feedMeta.reset(limit);
  }

  if (offset === 0) {
    isLoading.set(true);
  }
  
  try {
    if (isTauri) {
      const response = await invoke<PaginatedResponse<Bookmark>>('get_bookmarks', { offset, limit });
      if (offset === 0) {
        bookmarks.set(response.items);
      } else {
        bookmarks.append(response.items);
      }
      feedMeta.update({
        offset: offset + response.items.length,
        limit,
        total: response.total,
        hasMore: response.has_more
      });
    } else {
      await new Promise(resolve => setTimeout(resolve, 300));
      if (offset === 0) {
        bookmarks.set(mockBookmarks);
      } else {
        bookmarks.append(mockBookmarks);
      }
      feedMeta.update({
        offset: offset + mockBookmarks.length,
        limit,
        total: mockBookmarks.length,
        hasMore: false
      });
    }
  } catch (error) {
    console.error('Failed to load bookmarks:', error);
    if (offset === 0) {
      bookmarks.set([]);
    }
  } finally {
    if (offset === 0) {
      isLoading.set(false);
    }
  }
}

export async function loadMoreBookmarks(): Promise<void> {
  if (!feedMeta.value.hasMore || isLoading.value) {
    return;
  }
  await loadBookmarks({
    offset: feedMeta.value.offset,
    limit: feedMeta.value.limit,
  });
}

export async function searchBookmarks(query: string, tag?: string | null): Promise<void> {
  isLoading.set(true);
  
  try {
    if (isTauri) {
      const response = await invoke<Bookmark[]>('search_bookmarks', { 
        query: query || undefined,
        tag: tag || undefined,
        limit: 100 
      });
      bookmarks.set(response);
      feedMeta.reset(100);
      feedMeta.update({
        offset: response.length,
        total: response.length,
        hasMore: false
      });
    } else {
      // Development mock - filter locally
      await new Promise(resolve => setTimeout(resolve, 200));
      let filtered = mockBookmarks;
      
      if (query) {
        const q = query.toLowerCase();
        filtered = filtered.filter(b => 
          b.content.toLowerCase().includes(q) ||
          b.author_handle.toLowerCase().includes(q) ||
          b.author_name.toLowerCase().includes(q)
        );
      }
      
      if (tag) {
        filtered = filtered.filter(b => b.tags.includes(tag));
      }
      
      bookmarks.set(filtered);
      feedMeta.reset(100);
      feedMeta.update({
        offset: filtered.length,
        total: filtered.length,
        hasMore: false
      });
    }
  } catch (error) {
    console.error('Search failed:', error);
  } finally {
    isLoading.set(false);
  }
}

export async function loadStats(): Promise<void> {
  try {
    if (isTauri) {
      const data = await invoke<BookmarkStats>('get_stats');
      stats.set(data);
      allTags.set(data.top_tags);
    } else {
      stats.set(mockStats);
      allTags.set(mockStats.top_tags);
    }
  } catch (error) {
    console.error('Failed to load stats:', error);
  }
}

export async function importFile(path: string): Promise<number> {
  if (isTauri) {
    const count = await invoke<number>('import_file', { path });
    // Refresh data after import
    await Promise.all([loadBookmarks({ reset: true }), loadStats()]);
    return count;
  } else {
    // Mock import
    await new Promise(resolve => setTimeout(resolve, 1000));
    return 42;
  }
}

export async function deleteBookmark(id: string): Promise<void> {
  if (isTauri) {
    await invoke('delete_bookmark', { id });
    bookmarks.remove(id);
  } else {
    bookmarks.remove(id);
  }
}

export async function openInBrowser(url: string): Promise<void> {
  if (isTauri) {
    await invoke('open_url', { url });
  } else {
    window.open(url, '_blank');
  }
}

export async function toggleFavorite(id: string): Promise<boolean> {
  if (isTauri) {
    const newStatus = await invoke<boolean>('toggle_favorite', { id });
    // Update local state
    const items = bookmarks.value;
    const idx = items.findIndex(b => b.id === id);
    if (idx !== -1) {
      items[idx].is_favorite = newStatus;
      bookmarks.set([...items]);
    }
    return newStatus;
  } else {
    // Mock toggle
    const items = bookmarks.value;
    const idx = items.findIndex(b => b.id === id);
    if (idx !== -1) {
      items[idx].is_favorite = !items[idx].is_favorite;
      bookmarks.set([...items]);
      return items[idx].is_favorite;
    }
    return false;
  }
}

export async function loadFavorites(): Promise<void> {
  isLoading.set(true);
  
  try {
    if (isTauri) {
      const response = await invoke<Bookmark[]>('get_favorites', { limit: 100 });
      bookmarks.set(response);
      feedMeta.reset(100);
      feedMeta.update({
        offset: response.length,
        total: response.length,
        hasMore: false
      });
    } else {
      // Mock favorites
      await new Promise(resolve => setTimeout(resolve, 200));
      const favorites = mockBookmarks.filter(b => b.is_favorite);
      bookmarks.set(favorites);
      feedMeta.reset(100);
      feedMeta.update({
        offset: favorites.length,
        total: favorites.length,
        hasMore: false
      });
    }
  } catch (error) {
    console.error('Failed to load favorites:', error);
  } finally {
    isLoading.set(false);
  }
}

export async function searchWithFilters(filters: {
  query?: string;
  tag?: string;
  author?: string;
  fromDate?: string;
  toDate?: string;
  favoritesOnly?: boolean;
  hasMedia?: boolean;
}): Promise<void> {
  isLoading.set(true);
  
  try {
    if (isTauri) {
      const response = await invoke<Bookmark[]>('search_with_filters', {
        query: filters.query || undefined,
        tag: filters.tag || undefined,
        author: filters.author || undefined,
        fromDate: filters.fromDate || undefined,
        toDate: filters.toDate || undefined,
        favoritesOnly: filters.favoritesOnly || false,
        hasMedia: filters.hasMedia,
        limit: 100,
      });
      bookmarks.set(response);
      feedMeta.reset(100);
      feedMeta.update({
        offset: response.length,
        total: response.length,
        hasMore: false
      });
    } else {
      // Mock filter
      await new Promise(resolve => setTimeout(resolve, 200));
      let filtered = [...mockBookmarks];
      
      if (filters.query) {
        const q = filters.query.toLowerCase();
        filtered = filtered.filter(b => 
          b.content.toLowerCase().includes(q) ||
          b.author_handle.toLowerCase().includes(q)
        );
      }
      
      if (filters.favoritesOnly) {
        filtered = filtered.filter(b => b.is_favorite);
      }
      
      bookmarks.set(filtered);
      feedMeta.reset(100);
      feedMeta.update({
        offset: filtered.length,
        total: filtered.length,
        hasMore: false
      });
    }
  } catch (error) {
    console.error('Search with filters failed:', error);
  } finally {
    isLoading.set(false);
  }
}

const previewCache = new Map<string, Promise<LinkPreview | null>>();

export function fetchLinkPreview(url: string): Promise<LinkPreview | null> {
  if (previewCache.has(url)) {
    return previewCache.get(url)!;
  }
  
  const task = (async () => {
    try {
      if (isTauri) {
        return await invoke<LinkPreview>('fetch_link_preview', { url });
      }
      // Non-tauri fallback
      return null;
    } catch (error) {
      console.error('Failed to fetch link preview:', error);
      return null;
    }
  })();
  
  previewCache.set(url, task);
  return task;
}

