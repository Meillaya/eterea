// API layer for communicating with Tauri backend

import { invoke } from "@tauri-apps/api/core";
import type {
  Bookmark,
  BookmarkStats,
  LinkPreview,
  PaginatedResponse,
  XImportSummary,
  XSyncStatus,
} from "./types";
import {
  DEFAULT_FEED_LIMIT,
  allTags,
  bookmarks,
  dateRange,
  feedMeta,
  isLoading,
  isLoadingMore,
  isRefreshing,
  searchQuery,
  selectedTag,
  stats,
  viewMode,
} from "./stores/bookmarks.svelte";

// Check if running in Tauri
const isTauri = typeof window !== "undefined" && "__TAURI__" in window;

// Mock data for development without Tauri
const mockBookmarks: Bookmark[] = [
  {
    id: "1",
    tweet_url: "https://twitter.com/rustlang/status/123",
    content:
      "Rust 2024 edition is here! 🦀 Check out the new features including improved async support and better error messages.",
    note_text: null,
    tweeted_at: "2024-05-01T14:30:00Z",
    imported_at: "2024-05-02T10:00:00Z",
    author_handle: "rustlang",
    author_name: "Rust Language",
    author_profile_url: "https://twitter.com/rustlang",
    author_profile_image:
      "https://pbs.twimg.com/profile_images/1234/rust_normal.jpg",
    tags: ["technology", "programming"],
    comments: null,
    media: [],
    is_favorite: true,
  },
  {
    id: "2",
    tweet_url: "https://twitter.com/svaborern/status/456",
    content:
      "Svelte 5 runes are a game changer. The new reactivity system is so much simpler to understand.",
    note_text: null,
    tweeted_at: "2024-04-28T09:15:00Z",
    imported_at: "2024-05-02T10:00:00Z",
    author_handle: "svaborern",
    author_name: "Svelte Enthusiast",
    author_profile_url: null,
    author_profile_image: null,
    tags: ["technology", "webdev"],
    comments: null,
    media: [],
    is_favorite: false,
  },
];

const mockStats: BookmarkStats = {
  total_bookmarks: 2,
  unique_authors: 2,
  unique_tags: 3,
  favorite_bookmarks: 1,
  earliest_date: "2024-04-28T09:15:00Z",
  latest_date: "2024-05-01T14:30:00Z",
  top_tags: [
    ["technology", 2],
    ["programming", 1],
    ["webdev", 1],
  ],
};

type LibrarySnapshot = {
  version: 1;
  items: Bookmark[];
  feedMeta: PaginatedResponse<Bookmark>;
  cachedAt: number;
};

type StatsSnapshot = {
  version: 1;
  stats: BookmarkStats;
  cachedAt: number;
};

const LIBRARY_SNAPSHOT_KEY = "eterea:library-snapshot";
const STATS_SNAPSHOT_KEY = "eterea:stats-snapshot";
const DEFAULT_STATS_TIMEOUT_MS = 2_000;
const LINK_PREVIEW_TIMEOUT_MS = 1_500;
const LINK_PREVIEW_CONCURRENCY = 4;

let activeBookmarkRequestKey: string | null = null;
let activeBookmarkRequest: Promise<void> | null = null;
let activeStatsRequest: Promise<void> | null = null;
let primaryLoadDepth = 0;
let backgroundLoadDepth = 0;
let activePreviewRequests = 0;
const previewQueue: Array<() => void> = [];

function beginPrimaryLoad(): "blocking" | "background" {
  const mode = bookmarks.value.length > 0 ? "background" : "blocking";
  if (mode === "background") {
    backgroundLoadDepth += 1;
    isRefreshing.set(true);
  } else {
    primaryLoadDepth += 1;
    isLoading.set(true);
  }
  return mode;
}

function endPrimaryLoad(mode: "blocking" | "background") {
  if (mode === "background") {
    backgroundLoadDepth = Math.max(0, backgroundLoadDepth - 1);
    isRefreshing.set(backgroundLoadDepth > 0);
    return;
  }

  primaryLoadDepth = Math.max(0, primaryLoadDepth - 1);
  isLoading.set(primaryLoadDepth > 0);
}

function readSnapshot<T>(key: string): T | null {
  if (typeof window === "undefined") {
    return null;
  }

  try {
    const raw = window.localStorage.getItem(key);
    return raw ? (JSON.parse(raw) as T) : null;
  } catch (error) {
    console.warn(`Failed to read cache snapshot for ${key}:`, error);
    return null;
  }
}

function writeSnapshot(key: string, value: unknown) {
  if (typeof window === "undefined") {
    return;
  }

  try {
    window.localStorage.setItem(key, JSON.stringify(value));
  } catch (error) {
    console.warn(`Failed to persist cache snapshot for ${key}:`, error);
  }
}

function clearSnapshot(key: string) {
  if (typeof window === "undefined") {
    return;
  }

  window.localStorage.removeItem(key);
}

function persistLibrarySnapshot(response: PaginatedResponse<Bookmark>) {
  if (response.offset !== 0) {
    return;
  }

  writeSnapshot(LIBRARY_SNAPSHOT_KEY, {
    version: 1,
    items: response.items,
    feedMeta: response,
    cachedAt: Date.now(),
  } satisfies LibrarySnapshot);
}

function persistStatsSnapshot(data: BookmarkStats) {
  writeSnapshot(STATS_SNAPSHOT_KEY, {
    version: 1,
    stats: data,
    cachedAt: Date.now(),
  } satisfies StatsSnapshot);
}

export function hydrateCachedLibrarySnapshot() {
  const librarySnapshot = readSnapshot<LibrarySnapshot>(LIBRARY_SNAPSHOT_KEY);
  if (
    librarySnapshot?.version === 1 &&
    bookmarks.value.length === 0 &&
    librarySnapshot.items.length > 0
  ) {
    bookmarks.set(librarySnapshot.items);
    feedMeta.set({
      offset: librarySnapshot.feedMeta.offset + librarySnapshot.feedMeta.items.length,
      limit: librarySnapshot.feedMeta.limit,
      total: librarySnapshot.feedMeta.total,
      hasMore: librarySnapshot.feedMeta.has_more,
    });
  }

  const statsSnapshot = readSnapshot<StatsSnapshot>(STATS_SNAPSHOT_KEY);
  if (statsSnapshot?.version === 1 && !stats.value) {
    stats.set(statsSnapshot.stats);
    allTags.set(statsSnapshot.stats.top_tags);
  }
}

type LoadOptions = {
  offset?: number;
  limit?: number;
  reset?: boolean;
};

type LoadStatsOptions = {
  suppressErrors?: boolean;
  throwOnError?: boolean;
  timeoutMs?: number;
};

async function withTimeout<T>(
  task: Promise<T>,
  timeoutMs: number,
  message: string,
): Promise<T> {
  let timeoutId: ReturnType<typeof setTimeout> | undefined;

  const timeoutPromise = new Promise<never>((_, reject) => {
    timeoutId = setTimeout(() => {
      reject(new Error(message));
    }, timeoutMs);
  });

  try {
    return await Promise.race([task, timeoutPromise]);
  } finally {
    if (timeoutId !== undefined) {
      clearTimeout(timeoutId);
    }
  }
}

export async function loadBookmarks(options: LoadOptions = {}): Promise<void> {
  const limit = options.limit ?? feedMeta.value.limit ?? DEFAULT_FEED_LIMIT;
  const offset = options.reset
    ? 0
    : (options.offset ?? feedMeta.value.offset ?? 0);
  const requestKey = JSON.stringify({ offset, limit });
  if (activeBookmarkRequest && activeBookmarkRequestKey === requestKey) {
    return activeBookmarkRequest;
  }

  const loadMode = offset === 0 ? beginPrimaryLoad() : null;
  const hadItemsBeforeRequest = bookmarks.value.length > 0;

  let request!: Promise<void>;
  request = (async () => {
    try {
      if (isTauri) {
        const response = await invoke<PaginatedResponse<Bookmark>>(
          "get_bookmarks",
          { offset, limit },
        );
        if (offset === 0) {
          bookmarks.set(response.items);
          feedMeta.set({
            offset: response.offset + response.items.length,
            limit: response.limit,
            total: response.total,
            hasMore: response.has_more,
          });
          persistLibrarySnapshot(response);
        } else {
          bookmarks.append(response.items);
          feedMeta.update({
            offset: offset + response.items.length,
            limit,
            total: response.total,
            hasMore: response.has_more,
          });
        }
      } else {
        await new Promise((resolve) => setTimeout(resolve, 120));
        if (offset === 0) {
          bookmarks.set(mockBookmarks);
          feedMeta.set({
            offset: mockBookmarks.length,
            limit,
            total: mockBookmarks.length,
            hasMore: false,
          });
          persistLibrarySnapshot({
            items: mockBookmarks,
            total: mockBookmarks.length,
            offset: 0,
            limit,
            has_more: false,
          });
        } else {
          bookmarks.append(mockBookmarks);
          feedMeta.update({
            offset: offset + mockBookmarks.length,
            limit,
            total: mockBookmarks.length,
            hasMore: false,
          });
        }
      }
    } catch (error) {
      console.error("Failed to load bookmarks:", error);
      if (offset === 0 && !hadItemsBeforeRequest) {
        bookmarks.set([]);
        feedMeta.reset(limit);
      }
    } finally {
      if (loadMode) {
        endPrimaryLoad(loadMode);
      }
      if (activeBookmarkRequestKey === requestKey) {
        activeBookmarkRequest = null;
        activeBookmarkRequestKey = null;
      }
    }
  })();

  activeBookmarkRequestKey = requestKey;
  activeBookmarkRequest = request;
  return request;
}

export async function loadMoreBookmarks(): Promise<void> {
  if (!feedMeta.value.hasMore || isLoading.value || isLoadingMore.value) {
    return;
  }
  isLoadingMore.set(true);
  try {
    await loadBookmarks({
      offset: feedMeta.value.offset,
      limit: feedMeta.value.limit,
    });
  } finally {
    isLoadingMore.set(false);
  }
}

export async function searchBookmarks(
  query: string,
  tag?: string | null,
): Promise<void> {
  const loadMode = beginPrimaryLoad();

  try {
    if (isTauri) {
      const response = await invoke<Bookmark[]>("search_bookmarks", {
        query: query || undefined,
        tag: tag || undefined,
        limit: 100,
      });
      bookmarks.set(response);
      feedMeta.reset(100);
      feedMeta.update({
        offset: response.length,
        total: response.length,
        hasMore: false,
      });
    } else {
      await new Promise((resolve) => setTimeout(resolve, 200));
      let filtered = mockBookmarks;

      if (query) {
        const q = query.toLowerCase();
        filtered = filtered.filter(
          (bookmark) =>
            bookmark.content.toLowerCase().includes(q) ||
            bookmark.author_handle.toLowerCase().includes(q) ||
            bookmark.author_name.toLowerCase().includes(q),
        );
      }

      if (tag) {
        filtered = filtered.filter((bookmark) => bookmark.tags.includes(tag));
      }

      bookmarks.set(filtered);
      feedMeta.reset(100);
      feedMeta.update({
        offset: filtered.length,
        total: filtered.length,
        hasMore: false,
      });
    }
  } catch (error) {
    console.error("Search failed:", error);
  } finally {
    endPrimaryLoad(loadMode);
  }
}

export async function loadStats(options: LoadStatsOptions = {}): Promise<void> {
  if (activeStatsRequest) {
    return activeStatsRequest;
  }

  let request!: Promise<void>;
  request = (async () => {
  try {
    if (isTauri) {
      const data = await withTimeout(
        invoke<BookmarkStats>("get_stats"),
        options.timeoutMs ?? DEFAULT_STATS_TIMEOUT_MS,
        "Stats request timed out. Please retry.",
      );
      stats.set(data);
      allTags.set(data.top_tags);
      persistStatsSnapshot(data);
    } else {
      stats.set(mockStats);
      allTags.set(mockStats.top_tags);
      persistStatsSnapshot(mockStats);
    }
  } catch (error) {
    console.error("Failed to load stats:", error);
    if (options.throwOnError || !options.suppressErrors) {
      throw error instanceof Error ? error : new Error(String(error));
    }
  } finally {
    activeStatsRequest = null;
  }
  })();

  activeStatsRequest = request;
  return request;
}

export async function importFile(path: string): Promise<number> {
  if (isTauri) {
    const count = await invoke<number>("import_file", { path });
    await Promise.all([
      loadBookmarks({ reset: true }),
      loadStats({ suppressErrors: true }),
    ]);
    return count;
  }

  await new Promise((resolve) => setTimeout(resolve, 1000));
  return 42;
}

export async function importFileContent(file: File): Promise<number> {
  const content = await file.text();
  if (isTauri) {
    const count = await invoke<number>("import_bookmarks_content", {
      filename: file.name,
      content,
    });
    await Promise.all([
      loadBookmarks({ reset: true }),
      loadStats({ suppressErrors: true }),
    ]);
    return count;
  }

  await new Promise((resolve) => setTimeout(resolve, 1000));
  return 42;
}

export async function getXSyncStatus(): Promise<XSyncStatus> {
  if (isTauri) {
    return await invoke<XSyncStatus>("get_x_sync_status");
  }

  return {
    configured: false,
    connected: false,
    last_attempted_at: null,
    last_synced_at: null,
    last_status: null,
    last_error: null,
    last_imported_count: null,
    last_skipped_count: null,
    total_fetched: null,
  };
}

export async function importBookmarksFromX(): Promise<XImportSummary> {
  if (isTauri) {
    const summary = await invoke<XImportSummary>("import_bookmarks_from_x");
    await Promise.all([
      loadBookmarks({ reset: true }),
      loadStats({ suppressErrors: true }),
    ]);
    return summary;
  }

  await new Promise((resolve) => setTimeout(resolve, 1000));
  return {
    imported_count: mockBookmarks.length,
    skipped_count: 0,
    total_fetched: mockBookmarks.length,
    last_synced_at: new Date().toISOString(),
    reauthenticated: true,
    status_message: "Mock X import completed.",
  };
}

export async function deleteBookmark(id: string): Promise<void> {
  if (isTauri) {
    await invoke("delete_bookmark", { id });
  }
  bookmarks.remove(id);
  clearSnapshot(LIBRARY_SNAPSHOT_KEY);
  await loadStats({ suppressErrors: true });
}

export async function openInBrowser(url: string): Promise<void> {
  if (isTauri) {
    await invoke("open_url", { url });
  } else {
    window.open(url, "_blank");
  }
}

export async function toggleFavorite(id: string): Promise<boolean> {
  if (isTauri) {
    const newStatus = await invoke<boolean>("toggle_favorite", { id });
    const items = bookmarks.value;
    const index = items.findIndex((bookmark) => bookmark.id === id);
    if (index !== -1) {
      items[index].is_favorite = newStatus;
      bookmarks.set([...items]);
      clearSnapshot(LIBRARY_SNAPSHOT_KEY);
      if (stats.value) {
        stats.set({
          ...stats.value,
          favorite_bookmarks: Math.max(
            0,
            stats.value.favorite_bookmarks + (newStatus ? 1 : -1),
          ),
        });
      }
    }
    return newStatus;
  }

  const items = bookmarks.value;
  const index = items.findIndex((bookmark) => bookmark.id === id);
  if (index !== -1) {
    items[index].is_favorite = !items[index].is_favorite;
    bookmarks.set([...items]);
    clearSnapshot(LIBRARY_SNAPSHOT_KEY);
    if (stats.value) {
      stats.set({
        ...stats.value,
        favorite_bookmarks: Math.max(
          0,
          stats.value.favorite_bookmarks + (items[index].is_favorite ? 1 : -1),
        ),
      });
    }
    return items[index].is_favorite;
  }
  return false;
}

export async function loadFavorites(): Promise<void> {
  const loadMode = beginPrimaryLoad();

  try {
    if (isTauri) {
      const response = await invoke<Bookmark[]>("get_favorites", {
        limit: 100,
      });
      bookmarks.set(response);
      feedMeta.reset(100);
      feedMeta.update({
        offset: response.length,
        total: response.length,
        hasMore: false,
      });
    } else {
      await new Promise((resolve) => setTimeout(resolve, 200));
      const favorites = mockBookmarks.filter(
        (bookmark) => bookmark.is_favorite,
      );
      bookmarks.set(favorites);
      feedMeta.reset(100);
      feedMeta.update({
        offset: favorites.length,
        total: favorites.length,
        hasMore: false,
      });
    }
  } catch (error) {
    console.error("Failed to load favorites:", error);
  } finally {
    endPrimaryLoad(loadMode);
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
  const loadMode = beginPrimaryLoad();

  try {
    if (isTauri) {
      const response = await invoke<Bookmark[]>("search_with_filters", {
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
        hasMore: false,
      });
    } else {
      await new Promise((resolve) => setTimeout(resolve, 200));
      let filtered = [...mockBookmarks];

      if (filters.query) {
        const query = filters.query.toLowerCase();
        filtered = filtered.filter(
          (bookmark) =>
            bookmark.content.toLowerCase().includes(query) ||
            bookmark.author_handle.toLowerCase().includes(query),
        );
      }

      if (filters.tag) {
        filtered = filtered.filter((bookmark) =>
          bookmark.tags.includes(filters.tag!),
        );
      }

      if (filters.fromDate) {
        const from = new Date(filters.fromDate).getTime();
        filtered = filtered.filter(
          (bookmark) => new Date(bookmark.tweeted_at).getTime() >= from,
        );
      }

      if (filters.toDate) {
        const to = new Date(filters.toDate).getTime();
        filtered = filtered.filter(
          (bookmark) => new Date(bookmark.tweeted_at).getTime() <= to,
        );
      }

      if (filters.favoritesOnly) {
        filtered = filtered.filter((bookmark) => bookmark.is_favorite);
      }

      bookmarks.set(filtered);
      feedMeta.reset(100);
      feedMeta.update({
        offset: filtered.length,
        total: filtered.length,
        hasMore: false,
      });
    }
  } catch (error) {
    console.error("Search with filters failed:", error);
  } finally {
    endPrimaryLoad(loadMode);
  }
}

const previewCache = new Map<string, Promise<LinkPreview | null>>();

async function withPreviewSlot<T>(task: () => Promise<T>): Promise<T> {
  if (activePreviewRequests >= LINK_PREVIEW_CONCURRENCY) {
    await new Promise<void>((resolve) => previewQueue.push(resolve));
  }

  activePreviewRequests += 1;
  try {
    return await task();
  } finally {
    activePreviewRequests = Math.max(0, activePreviewRequests - 1);
    previewQueue.shift()?.();
  }
}

export function fetchLinkPreview(url: string): Promise<LinkPreview | null> {
  if (previewCache.has(url)) {
    return previewCache.get(url)!;
  }

  const task = (async () => {
    try {
      if (isTauri) {
        return await withPreviewSlot(() =>
          withTimeout(
            invoke<LinkPreview>("fetch_link_preview", { url }),
            LINK_PREVIEW_TIMEOUT_MS,
            "Link preview timed out.",
          ),
        );
      }
      return null;
    } catch (error) {
      console.debug("Link preview unavailable:", error);
      return null;
    }
  })();

  previewCache.set(url, task);
  return task;
}

export async function refreshBookmarks(): Promise<void> {
  const filters = {
    query: searchQuery.value || undefined,
    tag: selectedTag.value || undefined,
    fromDate: dateRange.value.from || undefined,
    toDate: dateRange.value.to || undefined,
    favoritesOnly: viewMode.value === "favorites",
  };

  const hasActiveFilters = Boolean(
    filters.query ||
    filters.tag ||
    filters.fromDate ||
    filters.toDate ||
    filters.favoritesOnly,
  );

  if (hasActiveFilters) {
    await searchWithFilters(filters);
    return;
  }

  await loadBookmarks({ reset: true });
}
