import type { Bookmark, BookmarkStats } from '$lib/types';

export const DEFAULT_FEED_LIMIT = 20;
const FEED_LIMIT_STORAGE_KEY = 'eterea:feed-limit';
const LAYOUT_STORAGE_KEY = 'eterea:layout';

export type ViewMode = 'all' | 'favorites' | 'recent';
export type LayoutMode = 'focus' | 'grid' | 'list';

export type FeedState = {
  offset: number;
  limit: number;
  total: number;
  hasMore: boolean;
};

export type InvokeTraceStatus = 'running' | 'success' | 'error';

export type InvokeTraceEntry = {
  id: string;
  command: string;
  phase: string;
  status: InvokeTraceStatus;
  startedAt: number;
  finishedAt: number | null;
  detail: string;
};

export type LibraryDiagnostics = {
  phase: string;
  detail: string;
  updatedAt: number;
};

export type RuntimeDiagnostics = {
  message: string | null;
  source: string;
  updatedAt: number;
};

function createValueStore<T>(initial: T) {
  let value = $state(initial);

  return {
    get value() {
      return value;
    },
    set(next: T) {
      value = next;
    },
  };
}

function createBookmarksStore() {
  let items = $state<Bookmark[]>([]);

  return {
    get value() {
      return items;
    },
    set(next: Bookmark[]) {
      items = next;
    },
    append(next: Bookmark[]) {
      items = [...items, ...next];
    },
    remove(id: string) {
      items = items.filter((bookmark) => bookmark.id !== id);
    },
    replace(id: string, patch: Partial<Bookmark>) {
      items = items.map((bookmark) =>
        bookmark.id === id ? { ...bookmark, ...patch } : bookmark,
      );
    },
    clear() {
      items = [];
    },
  };
}

function createTagStore() {
  let tag = $state<string | null>(null);

  return {
    get value() {
      return tag;
    },
    set(next: string | null) {
      tag = next;
    },
    clear() {
      tag = null;
    },
  };
}

function createAuthorStore() {
  let author = $state<string | null>(null);

  return {
    get value() {
      return author;
    },
    set(next: string | null) {
      author = next;
    },
    clear() {
      author = null;
    },
  };
}

function createDateRangeStore() {
  let range = $state<{ from: string | null; to: string | null }>({
    from: null,
    to: null,
  });

  return {
    get value() {
      return range;
    },
    set(from: string | null, to: string | null) {
      range = { from, to };
    },
    clear() {
      range = { from: null, to: null };
    },
  };
}

function resolveInitialLayout(): LayoutMode {
  if (typeof window === 'undefined') return 'focus';

  const stored = window.localStorage.getItem(LAYOUT_STORAGE_KEY);
  if (stored === 'focus' || stored === 'grid' || stored === 'list') {
    return stored;
  }

  if (stored === 'default') return 'focus';
  if (stored === 'cards') return 'grid';
  if (stored === 'compact') return 'list';
  return 'focus';
}

function createLayoutStore() {
  let layout = $state<LayoutMode>(resolveInitialLayout());

  return {
    get value() {
      return layout;
    },
    set(next: LayoutMode) {
      layout = next;
      if (typeof window !== 'undefined') {
        window.localStorage.setItem(LAYOUT_STORAGE_KEY, next);
      }
    },
  };
}

function resolveInitialFeedLimit(): number {
  if (typeof window === 'undefined') return DEFAULT_FEED_LIMIT;

  const stored = Number.parseInt(
    window.localStorage.getItem(FEED_LIMIT_STORAGE_KEY) ?? '',
    10,
  );

  if (Number.isFinite(stored) && stored >= 10 && stored <= 200) {
    return stored;
  }

  return DEFAULT_FEED_LIMIT;
}

function persistFeedLimit(limit: number) {
  if (typeof window === 'undefined') return;
  window.localStorage.setItem(FEED_LIMIT_STORAGE_KEY, String(limit));
}

function createFeedStore() {
  let feed = $state<FeedState>({
    offset: 0,
    limit: resolveInitialFeedLimit(),
    total: 0,
    hasMore: true,
  });

  return {
    get value() {
      return feed;
    },
    set(next: FeedState) {
      persistFeedLimit(next.limit);
      feed = next;
    },
    update(partial: Partial<FeedState>) {
      const next = { ...feed, ...partial };
      persistFeedLimit(next.limit);
      feed = next;
    },
    reset(limit = resolveInitialFeedLimit()) {
      persistFeedLimit(limit);
      feed = { offset: 0, limit, total: 0, hasMore: true };
    },
  };
}

function createInvokeDiagnosticsStore() {
  let current = $state<InvokeTraceEntry | null>(null);
  let history = $state<InvokeTraceEntry[]>([]);

  function push(entry: InvokeTraceEntry) {
    history = [entry, ...history].slice(0, 12);
  }

  return {
    get value() {
      return { current, history };
    },
    start(entry: Omit<InvokeTraceEntry, 'status' | 'finishedAt'>) {
      current = {
        ...entry,
        status: 'running',
        finishedAt: null,
      };
    },
    finish(update: {
      id: string;
      command: string;
      phase: string;
      detail: string;
      status: Exclude<InvokeTraceStatus, 'running'>;
    }) {
      const base =
        current && current.id === update.id
          ? current
          : {
              id: update.id,
              command: update.command,
              phase: update.phase,
              startedAt: Date.now(),
              finishedAt: null,
              detail: update.detail,
              status: 'running' as const,
            };

      const entry: InvokeTraceEntry = {
        ...base,
        status: update.status,
        finishedAt: Date.now(),
        detail: update.detail,
      };

      if (current?.id === update.id) {
        current = null;
      }

      push(entry);
    },
    clear() {
      current = null;
      history = [];
    },
  };
}

function createLibraryDiagnosticsStore() {
  let value = $state<LibraryDiagnostics>({
    phase: 'idle',
    detail: 'No load started yet.',
    updatedAt: Date.now(),
  });

  return {
    get value() {
      return value;
    },
    set(phase: string, detail: string) {
      value = {
        phase,
        detail,
        updatedAt: Date.now(),
      };
    },
  };
}

function createRuntimeDiagnosticsStore() {
  let value = $state<RuntimeDiagnostics>({
    message: null,
    source: 'none',
    updatedAt: Date.now(),
  });

  return {
    get value() {
      return value;
    },
    set(message: string, source: string) {
      value = {
        message,
        source,
        updatedAt: Date.now(),
      };
    },
    clear() {
      value = {
        message: null,
        source: 'none',
        updatedAt: Date.now(),
      };
    },
  };
}

export const bookmarks = createBookmarksStore();
export const searchQuery = createValueStore('');
export const selectedTag = createTagStore();
export const selectedAuthor = createAuthorStore();
export const hasMediaFilter = createValueStore(false);
export const dateRange = createDateRangeStore();
export const viewMode = createValueStore<ViewMode>('all');
export const layoutMode = createLayoutStore();
export const feedMeta = createFeedStore();
export const isLoading = createValueStore(false);
export const isRefreshing = createValueStore(false);
export const isLoadingMore = createValueStore(false);
export const stats = createValueStore<BookmarkStats | null>(null);
export const allTags = createValueStore<[string, number][]>([]);
export const loadError = createValueStore<string | null>(null);
export const invokeDiagnostics = createInvokeDiagnosticsStore();
export const libraryDiagnostics = createLibraryDiagnosticsStore();
export const runtimeDiagnostics = createRuntimeDiagnosticsStore();
