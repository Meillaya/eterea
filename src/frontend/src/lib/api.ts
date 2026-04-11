import { invoke, isTauri as detectTauri } from '@tauri-apps/api/core';
import type {
  Bookmark,
  BookmarkStats,
  LinkPreview,
  PaginatedResponse,
  SearchFilters,
  XImportSummary,
  XSyncStatus,
} from '$lib/types';
import {
  DEFAULT_FEED_LIMIT,
  allTags,
  bookmarks,
  dateRange,
  feedMeta,
  isLoading,
  isLoadingMore,
  isRefreshing,
  invokeDiagnostics,
  libraryDiagnostics,
  loadError,
  searchQuery,
  selectedTag,
  stats,
  viewMode,
} from '$lib/stores/bookmarks.svelte';

const isTauri = detectTauri();
const LIBRARY_SNAPSHOT_KEY = 'eterea:library-snapshot';
const STATS_SNAPSHOT_KEY = 'eterea:stats-snapshot';
const DEFAULT_STATS_TIMEOUT_MS = 2_000;
const LINK_PREVIEW_TIMEOUT_MS = 1_500;
const LINK_PREVIEW_CONCURRENCY = 4;
const MAX_IMPORT_FILE_BYTES = 10 * 1024 * 1024;
const ALLOWED_IMPORT_EXTENSIONS = new Set(['csv', 'json', 'js']);

type LibrarySnapshot = {
  version: 1;
  items: Bookmark[];
  response: PaginatedResponse<Bookmark>;
  cachedAt: number;
};

type StatsSnapshot = {
  version: 1;
  stats: BookmarkStats;
  cachedAt: number;
};

const demoBookmarks: Bookmark[] = [
  {
    id: '1',
    tweet_url: 'https://twitter.com/rustlang/status/123',
    content:
      'Rust 2024 edition is here! 🦀 Faster async stories, cleaner diagnostics, and more room for careful tooling.',
    note_text: 'Worth revisiting for the release notes language.',
    tweeted_at: '2024-05-01T14:30:00Z',
    imported_at: '2024-05-02T10:00:00Z',
    author_handle: 'rustlang',
    author_name: 'Rust Language',
    author_profile_url: 'https://twitter.com/rustlang',
    author_profile_image: 'https://pbs.twimg.com/profile_images/1234/rust_normal.jpg',
    tags: ['technology', 'programming'],
    comments: null,
    media: [],
    is_favorite: true,
  },
  {
    id: '2',
    tweet_url: 'https://twitter.com/svaborern/status/456',
    content:
      'Svelte 5 runes make local state read like real code. The new mental model is smaller, and it feels faster.',
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
    is_favorite: false,
  },
];

const demoStats: BookmarkStats = {
  total_bookmarks: 2,
  unique_authors: 2,
  unique_tags: 3,
  favorite_bookmarks: 1,
  earliest_date: '2024-04-28T09:15:00Z',
  latest_date: '2024-05-01T14:30:00Z',
  top_tags: [
    ['technology', 2],
    ['programming', 1],
    ['webdev', 1],
  ],
};

let activePageRequestKey: string | null = null;
let activePageRequest: Promise<void> | null = null;
let activeStatsRequest: Promise<void> | null = null;
let activeStatsTransport: Promise<unknown> | null = null;
let latestLibraryRequestId = 0;
let invokeTraceCount = 0;
let blockingLoadDepth = 0;
let backgroundLoadDepth = 0;
let previewRequests = 0;
const previewWaiters: Array<() => void> = [];
const previewCache = new Map<string, Promise<LinkPreview | null>>();

function normalizeExternalUrl(
  url: string,
  options: { requirePublicHost?: boolean } = {},
): string | null {
  try {
    const parsed = new URL(url);
    if (parsed.protocol !== 'https:') {
      return null;
    }

    if (!parsed.hostname) {
      return null;
    }

    const hostname = parsed.hostname.toLowerCase();
    if (!options.requirePublicHost) {
      return parsed.toString();
    }

    if (
      hostname === 'localhost' ||
      hostname.endsWith('.local') ||
      hostname.endsWith('.internal') ||
      isPrivateHostname(hostname)
    ) {
      return null;
    }

    return parsed.toString();
  } catch {
    return null;
  }
}

function isPrivateHostname(hostname: string): boolean {
  const ipv4Match = hostname.match(/^(\d{1,3}\.){3}\d{1,3}$/);
  if (ipv4Match) {
    const parts = hostname.split('.').map((part) => Number.parseInt(part, 10));
    if (parts.some((part) => Number.isNaN(part) || part < 0 || part > 255)) {
      return true;
    }

    const [a, b] = parts;
    if (a === 10 || a === 127 || a === 0) return true;
    if (a === 169 && b === 254) return true;
    if (a === 172 && b >= 16 && b <= 31) return true;
    if (a === 192 && b === 168) return true;
    if (a === 100 && b >= 64 && b <= 127) return true;
    return false;
  }

  return hostname === '::1' || hostname.startsWith('fc') || hostname.startsWith('fd') || hostname.startsWith('fe80:');
}

function validateImportFile(file: File): void {
  const extension = file.name.split('.').pop()?.toLowerCase() ?? '';
  if (!ALLOWED_IMPORT_EXTENSIONS.has(extension)) {
    throw new Error('Only CSV, JSON, and X archive JS files can be imported.');
  }

  if (file.size > MAX_IMPORT_FILE_BYTES) {
    throw new Error('Import files must be 10 MB or smaller.');
  }
}

function readSnapshot<T>(key: string): T | null {
  if (typeof window === 'undefined') return null;

  try {
    const raw = window.localStorage.getItem(key);
    return raw ? (JSON.parse(raw) as T) : null;
  } catch (error) {
    console.warn(`Failed to parse snapshot for ${key}`, error);
    return null;
  }
}

function writeSnapshot(key: string, value: unknown) {
  if (typeof window === 'undefined') return;

  try {
    window.localStorage.setItem(key, JSON.stringify(value));
  } catch (error) {
    console.warn(`Failed to persist snapshot for ${key}`, error);
  }
}

function clearSnapshot(key: string) {
  if (typeof window === 'undefined') return;
  window.localStorage.removeItem(key);
}

function nextTraceId(prefix: string): string {
  invokeTraceCount += 1;
  return `${prefix}-${invokeTraceCount}`;
}

function summarizeArgs(args: Record<string, unknown>): string {
  return Object.entries(args)
    .filter(([, value]) => value !== undefined)
    .map(([key, value]) => {
      if (typeof value === 'string') return `${key}=${value}`;
      if (typeof value === 'number' || typeof value === 'boolean') return `${key}=${value}`;
      if (value && typeof value === 'object' && 'length' in value) return `${key}=[len ${(value as { length: number }).length}]`;
      return key;
    })
    .join(' · ');
}

function summarizeResult(result: unknown): string {
  if (Array.isArray(result)) return `${result.length} item${result.length === 1 ? '' : 's'}`;
  if (result && typeof result === 'object') {
    if ('items' in result && Array.isArray((result as { items: unknown[] }).items)) {
      const response = result as PaginatedResponse<unknown>;
      return `${response.items.length}/${response.total} items @${response.offset}`;
    }
    if ('total_bookmarks' in result) {
      const statsResult = result as BookmarkStats;
      return `${statsResult.total_bookmarks} bookmarks · ${statsResult.unique_tags} tags`;
    }
  }
  return 'ok';
}

async function invokeWithTrace<T>(
  command: string,
  args: Record<string, unknown>,
  phase: string,
  options: { forwardTraceId?: boolean } = {},
): Promise<T> {
  const traceId = nextTraceId(command);
  const payload = options.forwardTraceId ? { ...args, traceId } : args;
  const startedAt = Date.now();
  const detail = summarizeArgs(options.forwardTraceId ? payload : { ...args, traceId });

  invokeDiagnostics.start({
    id: traceId,
    command,
    phase,
    startedAt,
    detail,
  });
  console.info(`[eterea][invoke:start] ${traceId} ${command} ${detail}`);

  try {
    const result = await invoke<T>(command, payload);
    const resultDetail = `${summarizeResult(result)} · ${Date.now() - startedAt}ms`;
    console.info(`[eterea][invoke:success] ${traceId} ${command} ${resultDetail}`);
    invokeDiagnostics.finish({
      id: traceId,
      command,
      phase,
      status: 'success',
      detail: resultDetail,
    });
    return result;
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    const resultDetail = `${message} · ${Date.now() - startedAt}ms`;
    console.error(`[eterea][invoke:error] ${traceId} ${command} ${resultDetail}`);
    invokeDiagnostics.finish({
      id: traceId,
      command,
      phase,
      status: 'error',
      detail: resultDetail,
    });
    throw error;
  }
}

function persistLibrarySnapshot(response: PaginatedResponse<Bookmark>) {
  if (response.offset !== 0) return;

  writeSnapshot(
    LIBRARY_SNAPSHOT_KEY,
    {
      version: 1,
      items: response.items,
      response,
      cachedAt: Date.now(),
    } satisfies LibrarySnapshot,
  );
}

function persistStatsSnapshot(nextStats: BookmarkStats) {
  writeSnapshot(
    STATS_SNAPSHOT_KEY,
    {
      version: 1,
      stats: nextStats,
      cachedAt: Date.now(),
    } satisfies StatsSnapshot,
  );
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
      offset: librarySnapshot.response.offset + librarySnapshot.response.items.length,
      limit: librarySnapshot.response.limit,
      total: librarySnapshot.response.total,
      hasMore: librarySnapshot.response.has_more,
    });
  }

  const statsSnapshot = readSnapshot<StatsSnapshot>(STATS_SNAPSHOT_KEY);
  if (statsSnapshot?.version === 1 && !stats.value) {
    stats.set(statsSnapshot.stats);
    allTags.set(statsSnapshot.stats.top_tags);
  }
}

function beginPrimaryLoad(): 'blocking' | 'background' {
  const mode = bookmarks.value.length > 0 ? 'background' : 'blocking';
  libraryDiagnostics.set(
    mode === 'background' ? 'background-refresh' : 'blocking-load',
    `bookmarks=${bookmarks.value.length} total=${feedMeta.value.total}`,
  );

  if (mode === 'background') {
    backgroundLoadDepth += 1;
    isRefreshing.set(true);
  } else {
    blockingLoadDepth += 1;
    isLoading.set(true);
  }

  return mode;
}

function endPrimaryLoad(mode: 'blocking' | 'background') {
  if (mode === 'background') {
    backgroundLoadDepth = Math.max(0, backgroundLoadDepth - 1);
    isRefreshing.set(backgroundLoadDepth > 0);
    libraryDiagnostics.set(
      backgroundLoadDepth > 0 ? 'background-refresh' : 'idle',
      `refresh-finished bookmarks=${bookmarks.value.length} total=${feedMeta.value.total}`,
    );
    return;
  }

  blockingLoadDepth = Math.max(0, blockingLoadDepth - 1);
  isLoading.set(blockingLoadDepth > 0);
  libraryDiagnostics.set(
    blockingLoadDepth > 0 ? 'blocking-load' : 'idle',
    `load-finished bookmarks=${bookmarks.value.length} total=${feedMeta.value.total}`,
  );
}

async function withTimeout<T>(task: Promise<T>, timeoutMs: number, message: string): Promise<T> {
  let timeoutId: ReturnType<typeof setTimeout> | undefined;

  const timeout = new Promise<never>((_, reject) => {
    timeoutId = setTimeout(() => reject(new Error(message)), timeoutMs);
  });

  try {
    return await Promise.race([task, timeout]);
  } finally {
    if (timeoutId !== undefined) clearTimeout(timeoutId);
  }
}

function applyFilteredPage(
  response: PaginatedResponse<Bookmark>,
  options: { append?: boolean } = {},
) {
  if (options.append && response.offset > 0) {
    bookmarks.append(response.items);
    feedMeta.update({
      offset: response.offset + response.items.length,
      limit: response.limit,
      total: response.total,
      hasMore: response.has_more,
    });
    libraryDiagnostics.set(
      'apply-results',
      `append filtered items=${response.items.length} total=${response.total} offset=${response.offset}`,
    );
    console.info(
      `[eterea][library] append filtered items=${response.items.length} total=${response.total} offset=${response.offset}`,
    );
    return;
  }

  bookmarks.set(response.items);
  feedMeta.set({
    offset: response.offset + response.items.length,
    limit: response.limit,
    total: response.total,
    hasMore: response.has_more,
  });
  libraryDiagnostics.set(
    'apply-results',
    `replace items=${response.items.length} total=${response.total} offset=${response.offset}`,
  );
  console.info(
    `[eterea][library] replace items=${response.items.length} total=${response.total} offset=${response.offset}`,
  );
}

function normalizeDemoFilters(filters: SearchFilters): Bookmark[] {
  let filtered = [...demoBookmarks];

  if (filters.query) {
    const query = filters.query.toLowerCase();
    filtered = filtered.filter(
      (bookmark) =>
        bookmark.content.toLowerCase().includes(query) ||
        bookmark.author_handle.toLowerCase().includes(query) ||
        bookmark.author_name.toLowerCase().includes(query),
    );
  }

  if (filters.tag) {
    filtered = filtered.filter((bookmark) => bookmark.tags.includes(filters.tag!));
  }

  if (filters.fromDate) {
    const from = new Date(filters.fromDate).getTime();
    filtered = filtered.filter((bookmark) => new Date(bookmark.tweeted_at).getTime() >= from);
  }

  if (filters.toDate) {
    const to = new Date(filters.toDate).getTime();
    filtered = filtered.filter((bookmark) => new Date(bookmark.tweeted_at).getTime() <= to);
  }

  if (filters.favoritesOnly) {
    filtered = filtered.filter((bookmark) => bookmark.is_favorite);
  }

  if (filters.hasMedia) {
    filtered = filtered.filter((bookmark) => bookmark.media.length > 0);
  }

  return filtered;
}

export async function loadBookmarks(options: { offset?: number; limit?: number; reset?: boolean } = {}) {
  const limit = options.limit ?? feedMeta.value.limit ?? DEFAULT_FEED_LIMIT;
  const offset = options.reset ? 0 : (options.offset ?? feedMeta.value.offset ?? 0);
  const requestKey = JSON.stringify({ offset, limit });

  if (activePageRequest && activePageRequestKey === requestKey) {
    console.info(`[eterea][library] dedupe get_bookmarks ${requestKey}`);
    return activePageRequest;
  }

  const requestId = ++latestLibraryRequestId;
  const loadMode = offset === 0 ? beginPrimaryLoad() : null;
  const hadVisibleItems = bookmarks.value.length > 0;
  if (offset === 0) loadError.set(null);

  let request!: Promise<void>;
  request = (async () => {
    try {
      libraryDiagnostics.set('dispatch-get-bookmarks', `offset=${offset} limit=${limit}`);
      if (isTauri) {
        const response = await invokeWithTrace<PaginatedResponse<Bookmark>>('get_bookmarks', {
          offset,
          limit,
        }, 'library-load', { forwardTraceId: true });

        if (requestId !== latestLibraryRequestId) {
          console.info(`[eterea][library] stale get_bookmarks ignored requestId=${requestId}`);
          return;
        }
        
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
            limit: response.limit,
            total: response.total,
            hasMore: response.has_more,
          });
        }
      } else {
        await new Promise((resolve) => setTimeout(resolve, 120));
        const response: PaginatedResponse<Bookmark> = {
          items: demoBookmarks.slice(offset, offset + limit),
          total: demoBookmarks.length,
          offset,
          limit,
          has_more: offset + limit < demoBookmarks.length,
        };
        if (requestId !== latestLibraryRequestId) {
          console.info(`[eterea][library] stale demo get_bookmarks ignored requestId=${requestId}`);
          return;
        }
        if (offset === 0) {
          bookmarks.set(response.items);
          feedMeta.set({
            offset: response.items.length,
            limit,
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
      }
    } catch (error) {
      console.error('Failed to load bookmarks', error);
      if (requestId !== latestLibraryRequestId) {
        return;
      }
      if (offset === 0 && !hadVisibleItems) {
        bookmarks.clear();
        feedMeta.reset(limit);
        loadError.set(error instanceof Error ? error.message : 'Unable to load bookmarks right now.');
      }
      libraryDiagnostics.set(
        'load-error',
        error instanceof Error ? error.message : 'Unable to load bookmarks right now.',
      );
    } finally {
      if (loadMode) endPrimaryLoad(loadMode);
      activePageRequest = null;
      if (activePageRequestKey === requestKey) {
        activePageRequestKey = null;
      }
    }
  })();

  activePageRequestKey = requestKey;
  activePageRequest = request;
  return request;
}

export async function loadMoreBookmarks() {
  if (!feedMeta.value.hasMore || isLoading.value || isLoadingMore.value) return;

  isLoadingMore.set(true);
  try {
    const recent = viewMode.value === 'recent' ? recentRange() : null;
    const activeFilters: SearchFilters = {
      query: searchQuery.value || undefined,
      tag: selectedTag.value || undefined,
      fromDate: dateRange.value.from || recent?.from,
      toDate: dateRange.value.to || recent?.to,
      favoritesOnly: viewMode.value === 'favorites',
    };
    const hasActiveFilters = Boolean(
      activeFilters.query ||
        activeFilters.tag ||
        activeFilters.fromDate ||
        activeFilters.toDate ||
        activeFilters.favoritesOnly,
    );

    if (hasActiveFilters) {
      await searchWithFilters(activeFilters, {
        append: true,
        offset: feedMeta.value.offset,
        limit: feedMeta.value.limit,
      });
      return;
    }

    await loadBookmarks({
      offset: feedMeta.value.offset,
      limit: feedMeta.value.limit,
    });
  } finally {
    isLoadingMore.set(false);
  }
}

export async function searchBookmarks(query: string, tag?: string | null) {
  return searchWithFilters({ query, tag: tag ?? undefined });
}

export async function loadStats(options: {
  suppressErrors?: boolean;
  throwOnError?: boolean;
  timeoutMs?: number;
} = {}) {
  if (activeStatsRequest) return activeStatsRequest;

  let request!: Promise<void>;
  request = (async () => {
    try {
      if (isTauri) {
        const transport = invokeWithTrace<BookmarkStats>('get_stats', {}, 'stats-load', {
          forwardTraceId: true,
        });
        activeStatsTransport = transport.finally(() => {
          activeStatsTransport = null;
        });
        const nextStats = await withTimeout(
          transport,
          options.timeoutMs ?? DEFAULT_STATS_TIMEOUT_MS,
          'Stats request timed out. Please retry.',
        );
        stats.set(nextStats);
        allTags.set(nextStats.top_tags);
        persistStatsSnapshot(nextStats);
        libraryDiagnostics.set(
          'stats-loaded',
          `total=${nextStats.total_bookmarks} tags=${nextStats.unique_tags}`,
        );
      } else {
        stats.set(demoStats);
        allTags.set(demoStats.top_tags);
        persistStatsSnapshot(demoStats);
      }
    } catch (error) {
      console.error('Failed to load stats', error);
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
    const importedCount = await invokeWithTrace<number>('import_file', { path }, 'import-file');
    await Promise.all([loadBookmarks({ reset: true }), loadStats({ suppressErrors: true })]);
    return importedCount;
  }

  await new Promise((resolve) => setTimeout(resolve, 800));
  return 42;
}

export async function importFileContent(file: File): Promise<number> {
  validateImportFile(file);
  const content = await file.text();

  if (isTauri) {
    const importedCount = await invokeWithTrace<number>('import_bookmarks_content', {
      filename: file.name,
      content,
    }, 'import-content');
    await Promise.all([loadBookmarks({ reset: true }), loadStats({ suppressErrors: true })]);
    return importedCount;
  }

  await new Promise((resolve) => setTimeout(resolve, 800));
  return 42;
}

export async function getXSyncStatus(): Promise<XSyncStatus> {
  if (isTauri) {
    return invokeWithTrace<XSyncStatus>('get_x_sync_status', {}, 'x-sync-status');
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
    const summary = await invokeWithTrace<XImportSummary>('import_bookmarks_from_x', {}, 'x-import');
    await Promise.all([loadBookmarks({ reset: true }), loadStats({ suppressErrors: true })]);
    return summary;
  }

  await new Promise((resolve) => setTimeout(resolve, 1000));
  return {
    imported_count: demoBookmarks.length,
    skipped_count: 0,
    total_fetched: demoBookmarks.length,
    last_synced_at: new Date().toISOString(),
    reauthenticated: true,
    status_message: 'Mock X import completed.',
  };
}

export async function deleteBookmark(id: string): Promise<void> {
  if (isTauri) {
    await invokeWithTrace('delete_bookmark', { id }, 'delete-bookmark');
  }

  bookmarks.remove(id);
  feedMeta.update({
    total: Math.max(0, feedMeta.value.total - 1),
    offset: Math.max(0, feedMeta.value.offset - 1),
  });
  clearSnapshot(LIBRARY_SNAPSHOT_KEY);
  await loadStats({ suppressErrors: true });
}

export async function openInBrowser(url: string) {
  const safeUrl = normalizeExternalUrl(url, { requirePublicHost: true });
  if (!safeUrl) {
    console.warn('Blocked unsafe external URL:', url);
    return;
  }

  if (isTauri) {
    await invokeWithTrace('open_url', { url: safeUrl }, 'open-url');
    return;
  }

  window.open(safeUrl, '_blank', 'noopener,noreferrer');
}

export async function toggleFavorite(id: string): Promise<boolean> {
  const current = bookmarks.value.find((bookmark) => bookmark.id === id);
  if (!current) return false;

  const nextStatus = isTauri
    ? await invokeWithTrace<boolean>('toggle_favorite', { id }, 'toggle-favorite')
    : !current.is_favorite;

  if (viewMode.value === 'favorites' && !nextStatus) {
    bookmarks.remove(id);
    feedMeta.update({
      total: Math.max(0, feedMeta.value.total - 1),
      offset: Math.max(0, feedMeta.value.offset - 1),
    });
  } else {
    bookmarks.replace(id, { is_favorite: nextStatus });
  }

  clearSnapshot(LIBRARY_SNAPSHOT_KEY);

  if (stats.value) {
    stats.set({
      ...stats.value,
      favorite_bookmarks: Math.max(
        0,
        stats.value.favorite_bookmarks + (nextStatus === current.is_favorite ? 0 : nextStatus ? 1 : -1),
      ),
    });
  }

  return nextStatus;
}

export async function loadFavorites() {
  return searchWithFilters({ favoritesOnly: true });
}

export async function searchWithFilters(
  filters: SearchFilters,
  options: { append?: boolean; offset?: number; limit?: number } = {},
): Promise<void> {
  const loadMode = beginPrimaryLoad();
  const limit = options.limit ?? feedMeta.value.limit ?? DEFAULT_FEED_LIMIT;
  const offset = options.append ? (options.offset ?? feedMeta.value.offset ?? 0) : 0;
  const requestId = ++latestLibraryRequestId;
  loadError.set(null);

  try {
    libraryDiagnostics.set(
      'dispatch-search',
      `offset=${offset} limit=${limit} query=${filters.query ?? ''} tag=${filters.tag ?? ''} favorites=${filters.favoritesOnly ? 'yes' : 'no'}`,
    );
    if (isTauri) {
      const response = await invokeWithTrace<PaginatedResponse<Bookmark>>('search_with_filters', {
        query: filters.query || undefined,
        tag: filters.tag || undefined,
        author: filters.author || undefined,
        fromDate: filters.fromDate || undefined,
        toDate: filters.toDate || undefined,
        offset,
        favoritesOnly: filters.favoritesOnly || false,
        hasMedia: filters.hasMedia,
        limit,
      }, 'filtered-load', { forwardTraceId: true });
      if (requestId !== latestLibraryRequestId) {
        console.info(`[eterea][library] stale search_with_filters ignored requestId=${requestId}`);
        return;
      }
      applyFilteredPage(response, { append: options.append });
      return;
    }

    await new Promise((resolve) => setTimeout(resolve, 160));
    const matched = normalizeDemoFilters(filters);
    const items = matched.slice(offset, offset + limit);
    const response: PaginatedResponse<Bookmark> = {
      items,
      total: matched.length,
      offset,
      limit,
      has_more: offset + items.length < matched.length,
    };
    if (requestId !== latestLibraryRequestId) {
      console.info(`[eterea][library] stale demo search_with_filters ignored requestId=${requestId}`);
      return;
    }
    applyFilteredPage(response, { append: options.append });
  } catch (error) {
    console.error('Failed to search bookmarks', error);
    if (requestId !== latestLibraryRequestId) {
      return;
    }
    if (bookmarks.value.length === 0) {
      loadError.set(error instanceof Error ? error.message : 'Unable to search the archive right now.');
    }
    libraryDiagnostics.set(
      'search-error',
      error instanceof Error ? error.message : 'Unable to search the archive right now.',
    );
  } finally {
    endPrimaryLoad(loadMode);
  }
}

async function withPreviewSlot<T>(task: () => Promise<T>): Promise<T> {
  if (previewRequests >= LINK_PREVIEW_CONCURRENCY) {
    await new Promise<void>((resolve) => previewWaiters.push(resolve));
  }

  previewRequests += 1;
  try {
    return await task();
  } finally {
    previewRequests = Math.max(0, previewRequests - 1);
    previewWaiters.shift()?.();
  }
}

export function fetchLinkPreview(url: string): Promise<LinkPreview | null> {
  const safeUrl = normalizeExternalUrl(url, { requirePublicHost: true });
  if (!safeUrl) {
    return Promise.resolve(null);
  }

  if (previewCache.has(safeUrl)) {
    return previewCache.get(safeUrl)!;
  }

  const task = (async () => {
    try {
      if (!isTauri) return null;

      return await withPreviewSlot(() =>
        withTimeout(
          invokeWithTrace<LinkPreview>('fetch_link_preview', { url: safeUrl }, 'link-preview'),
          LINK_PREVIEW_TIMEOUT_MS,
          'Link preview timed out.',
        ),
      );
    } catch (error) {
      console.debug('Link preview unavailable', error);
      return null;
    }
  })();

  previewCache.set(safeUrl, task);
  return task;
}

function recentRange() {
  const to = new Date();
  const from = new Date();
  from.setDate(from.getDate() - 30);
  return {
    from: from.toISOString(),
    to: to.toISOString(),
  };
}

export async function refreshBookmarks(): Promise<void> {
  const recent = viewMode.value === 'recent' ? recentRange() : null;
  const filters: SearchFilters = {
    query: searchQuery.value || undefined,
    tag: selectedTag.value || undefined,
    fromDate: dateRange.value.from || recent?.from,
    toDate: dateRange.value.to || recent?.to,
    favoritesOnly: viewMode.value === 'favorites',
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
