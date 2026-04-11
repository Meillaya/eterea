<script lang="ts">
  import { bookmarks, dateRange, feedMeta, hasMediaFilter, invokeDiagnostics, isLoading, isLoadingMore, isRefreshing, libraryDiagnostics, loadError, runtimeDiagnostics, searchQuery, selectedAuthor, selectedTag, viewMode } from '$lib/stores/bookmarks.svelte';
  import { loadMoreBookmarks } from '$lib/api';
  import BookmarkFeed from './BookmarkFeed.svelte';
  import DateRangePicker from './DateRangePicker.svelte';
  import LayoutSwitcher from './LayoutSwitcher.svelte';
  import LibrarySidebar from './LibrarySidebar.svelte';
  import SearchInput from './SearchInput.svelte';

  interface Props {
    onopenimport: () => void;
  }

  let { onopenimport }: Props = $props();

  function showLibrary() {
    searchQuery.set('');
    selectedTag.clear();
    selectedAuthor.clear();
    hasMediaFilter.set(false);
    dateRange.clear();
    viewMode.set('all');
  }

  function showRecent() {
    searchQuery.set('');
    selectedTag.clear();
    selectedAuthor.clear();
    hasMediaFilter.set(false);
    const to = new Date();
    const from = new Date();
    from.setDate(from.getDate() - 30);
    dateRange.set(from.toISOString(), to.toISOString());
    viewMode.set('recent');
  }

  function showFavorites() {
    searchQuery.set('');
    selectedTag.clear();
    selectedAuthor.clear();
    hasMediaFilter.set(false);
    dateRange.clear();
    viewMode.set('favorites');
  }

  function clearAllFilters() {
    searchQuery.set('');
    selectedTag.clear();
    selectedAuthor.clear();
    dateRange.clear();
    hasMediaFilter.set(false);
    viewMode.set('all');
  }

  function clearChip(id: string) {
    if (id === 'search') searchQuery.set('');
    if (id === 'tag') selectedTag.clear();
    if (id === 'author') selectedAuthor.clear();
    if (id === 'hasMedia') hasMediaFilter.set(false);
    if (id === 'recent') {
      dateRange.clear();
      viewMode.set('all');
    }
    if (id === 'favorites') viewMode.set('all');
    if (id === 'date') dateRange.clear();
  }

  function mobileNavClass(id: string) {
    return `rounded-full px-3 py-1.5 transition-colors ${viewMode.value === id ? 'bg-bg-elevated text-text-primary' : 'text-text-secondary'}`;
  }

  const activeFilterCount = $derived([
    searchQuery.value ? 1 : 0,
    selectedTag.value ? 1 : 0,
    selectedAuthor.value ? 1 : 0,
    dateRange.value.from ? 1 : 0,
    hasMediaFilter.value ? 1 : 0,
    viewMode.value === 'favorites' ? 1 : 0,
    viewMode.value === 'recent' ? 1 : 0,
  ].reduce((total, current) => total + current, 0));

  const modeLabel = $derived.by(() => {
    if (viewMode.value === 'favorites') return 'Favorites';
    if (viewMode.value === 'recent') return 'Recent';
    return 'Library';
  });

  const modeDescription = $derived.by(() => {
    if (viewMode.value === 'favorites') return 'Everything you starred to keep close.';
    if (viewMode.value === 'recent') return 'A fast cut through the last 30 days.';
    return 'Saved tweets, kept quiet and easy to read.';
  });

  const activeChips = $derived.by(() => {
    const chips: { id: string; label: string }[] = [];
    if (searchQuery.value) chips.push({ id: 'search', label: `Search: ${searchQuery.value}` });
    if (selectedTag.value) chips.push({ id: 'tag', label: `#${selectedTag.value}` });
    if (selectedAuthor.value) chips.push({ id: 'author', label: `@${selectedAuthor.value}` });
    if (hasMediaFilter.value) chips.push({ id: 'hasMedia', label: 'Has media' });
    if (viewMode.value === 'favorites') chips.push({ id: 'favorites', label: 'Favorites' });
    if (viewMode.value === 'recent') chips.push({ id: 'recent', label: 'Recent 30 days' });
    if (viewMode.value !== 'recent' && dateRange.value.from && dateRange.value.to) {
      const options: Intl.DateTimeFormatOptions = { month: 'short', day: 'numeric' };
      chips.push({
        id: 'date',
        label: `${new Date(dateRange.value.from).toLocaleDateString('en-US', options)} – ${new Date(dateRange.value.to).toLocaleDateString('en-US', options)}`,
      });
    }
    return chips;
  });

  const mobileNav = [
    { id: 'all', label: 'Library', action: showLibrary },
    { id: 'recent', label: 'Recent', action: showRecent },
    { id: 'favorites', label: 'Favorites', action: showFavorites },
  ] as const;

  let sentinelEl = $state<HTMLDivElement | undefined>(undefined);

  $effect(() => {
    const el = sentinelEl;
    if (!el) return;

    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0]?.isIntersecting && !isLoadingMore.value && !isLoading.value && feedMeta.value.hasMore) {
          void loadMoreBookmarks();
        }
      },
      { rootMargin: '400px' },
    );

    observer.observe(el);
    return () => observer.disconnect();
  });

  const currentTrace = $derived(invokeDiagnostics.value.current);
  const recentTraceHistory = $derived(invokeDiagnostics.value.history.slice(0, 4));
  const loadState = $derived(libraryDiagnostics.value);
  const runtimeState = $derived(runtimeDiagnostics.value);
</script>

<div class="flex min-h-screen bg-bg-primary text-text-primary">
  <LibrarySidebar {onopenimport} />

  <main class="min-w-0 flex-1">
    <div class="mx-auto flex w-full max-w-[1600px] flex-col gap-4 px-4 py-4 sm:px-6 xl:px-8">
      <section class="surface-panel rounded-[2.25rem] p-5 sm:p-7">
        <div class="flex flex-col gap-6 xl:flex-row xl:items-end xl:justify-between">
          <div class="space-y-4">
            <div class="flex flex-wrap items-center gap-2 text-xs">
              <span class="pill">{modeLabel}</span>
              <span class="pill">local-first</span>
              <span class="pill">{feedMeta.value.total.toLocaleString()} saved</span>
              {#if isRefreshing.value}
                <span class="pill text-accent">Refreshing…</span>
              {/if}
            </div>

            <div class="space-y-3">
              <h1 class="max-w-4xl font-display text-[2.5rem] leading-none italic text-text-primary sm:text-[3.45rem]">
                Read what you saved without the rest of the internet shouting over it.
              </h1>
              <p class="max-w-2xl text-sm leading-7 text-text-secondary sm:text-base">
                {modeDescription} Everything stays tuned for fast open, quick filtering, and a calmer archive that keeps tweet content first.
              </p>
            </div>
          </div>

          <div class="flex flex-wrap gap-3">
            <a class="ghost-button" href="/settings">Tune layout</a>
            <button class="accent-button" onclick={onopenimport}>Import bookmarks</button>
          </div>
        </div>

        <div class="mt-6 grid gap-3 lg:grid-cols-[minmax(0,1fr),auto] lg:items-center">
          <SearchInput />
          <div class="flex flex-wrap items-center gap-2 lg:justify-end">
            <div class="inline-flex rounded-full border border-border bg-bg-secondary/70 p-1 text-xs shadow-[var(--shadow-soft)] xl:hidden">
              {#each mobileNav as item}
                <button class={mobileNavClass(item.id)} onclick={item.action}>{item.label}</button>
              {/each}
            </div>
            <DateRangePicker />
            <button
              class={`ghost-button transition-colors ${hasMediaFilter.value ? 'border-border-accent text-accent' : ''}`}
              onclick={() => hasMediaFilter.set(!hasMediaFilter.value)}
              title="Show only bookmarks with media"
            >
              <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
              </svg>
              <span>Has media</span>
            </button>
            <LayoutSwitcher />
          </div>
        </div>

        {#if activeChips.length > 0}
          <div class="mt-4 flex flex-wrap gap-2">
            {#each activeChips as chip}
              <button class="pill transition-colors hover:border-border-accent hover:text-accent" onclick={() => clearChip(chip.id)}>
                <span>{chip.label}</span>
                <span aria-hidden="true">×</span>
              </button>
            {/each}
          </div>
        {/if}
      </section>

      <section class="soft-panel rounded-[2rem] p-5 sm:p-6">
        <div class="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
          <div>
            <p class="section-label">Reading feed</p>
            <h2 class="mt-2 text-xl font-medium text-text-primary">{modeLabel}</h2>
            <p class="mt-2 text-sm text-text-secondary">
              Showing {bookmarks.value.length.toLocaleString()} of {feedMeta.value.total.toLocaleString()} bookmarks.
              {#if activeFilterCount > 0}
                {activeFilterCount} filter{activeFilterCount === 1 ? '' : 's'} active.
              {/if}
            </p>
          </div>

          <div class="flex flex-wrap gap-2">
            <button class="ghost-button text-sm" onclick={clearAllFilters}>Reset view</button>
            <button class="ghost-button text-sm xl:hidden" onclick={onopenimport}>Import</button>
          </div>
        </div>

        {#if loadError.value && bookmarks.value.length === 0}
          <div class="mt-6 rounded-[1.5rem] border border-red-400/20 bg-red-500/8 p-5 text-sm text-red-100">
            <p class="font-medium">Couldn’t load the archive.</p>
            <p class="mt-2 text-red-100/80">{loadError.value}</p>
          </div>
        {:else if isLoading.value && bookmarks.value.length === 0}
          <div class="mt-6 space-y-4">
            {#each Array.from({ length: 4 }) as _}
              <div class="surface-panel rounded-[1.8rem] p-5" aria-hidden="true">
                <div class="skeleton h-4 w-36 rounded-full"></div>
                <div class="mt-5 space-y-3">
                  <div class="skeleton h-4 w-full rounded-full"></div>
                  <div class="skeleton h-4 w-[92%] rounded-full"></div>
                  <div class="skeleton h-4 w-[68%] rounded-full"></div>
                </div>
              </div>
            {/each}
          </div>
        {:else if bookmarks.value.length === 0}
          <div class="mt-6 rounded-[1.8rem] border border-border-subtle bg-bg-secondary/45 p-8 text-center">
            <p class="section-label">Nothing here yet</p>
            <h3 class="mt-3 text-2xl font-medium text-text-primary">The archive is quiet.</h3>
            <p class="mx-auto mt-3 max-w-xl text-sm leading-7 text-text-secondary">
              Clear the current filters or import another export to fill the library back in.
            </p>
            <div class="mt-6 flex flex-wrap justify-center gap-3">
              <button class="ghost-button" onclick={clearAllFilters}>Clear filters</button>
              <button class="accent-button" onclick={onopenimport}>Import bookmarks</button>
            </div>
          </div>
        {:else}
          <div class="mt-6">
            <BookmarkFeed items={bookmarks.value} />
          </div>
        {/if}

        {#if runtimeState.message}
          <div class="mt-6 rounded-[1.5rem] border border-amber-400/20 bg-amber-500/8 p-5 text-sm text-amber-100">
            <p class="font-medium">Runtime error captured.</p>
            <p class="mt-2 text-amber-100/80">{runtimeState.message}</p>
            <p class="mt-2 text-xs text-amber-200/80">source: {runtimeState.source}</p>
          </div>
        {/if}

        {#if currentTrace || recentTraceHistory.length > 0}
          <div class="mt-6 rounded-[1.5rem] border border-border-subtle bg-bg-primary/30 p-4 text-xs text-text-secondary">
            <div class="flex flex-wrap items-center justify-between gap-2">
              <p class="section-label">Live invoke trace</p>
              <span class="pill">check Tauri terminal for matching trace ids</span>
            </div>

            <div class="mt-3 rounded-[1rem] border border-border-subtle px-3 py-3">
              <div class="flex flex-wrap items-center justify-between gap-2">
                <span class="font-medium text-text-primary">frontend-state</span>
                <span class="pill">{loadState.phase}</span>
              </div>
              <p class="mt-1 text-text-muted">{loadState.detail}</p>
              <p class="mt-1 text-text-muted">
                bookmarks={bookmarks.value.length} · loading={isLoading.value ? 'yes' : 'no'} · refreshing={isRefreshing.value ? 'yes' : 'no'} · total={feedMeta.value.total}
              </p>
            </div>

            {#if currentTrace}
              <div class="mt-3 rounded-[1rem] border border-border-accent bg-[color:var(--accent-soft)] px-3 py-3">
                <p class="font-mono text-text-primary">{currentTrace.id}</p>
                <p class="mt-1 text-text-primary">{currentTrace.command} · {currentTrace.phase}</p>
                <p class="mt-1 text-text-muted">{currentTrace.detail}</p>
              </div>
            {/if}

            {#if recentTraceHistory.length > 0}
              <div class="mt-3 space-y-2">
                {#each recentTraceHistory as trace}
                  <div class="rounded-[1rem] border border-border-subtle px-3 py-2">
                    <div class="flex flex-wrap items-center justify-between gap-2">
                      <span class="font-mono text-text-primary">{trace.id}</span>
                      <span class="pill">{trace.status}</span>
                    </div>
                    <p class="mt-1">{trace.command} · {trace.phase}</p>
                    <p class="mt-1 text-text-muted">{trace.detail}</p>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        {/if}

        {#if feedMeta.value.hasMore}
          <div bind:this={sentinelEl} class="mt-6 h-1" aria-hidden="true"></div>
          {#if isLoadingMore.value}
            <div class="mt-4 flex justify-center">
              <div class="h-5 w-5 animate-spin rounded-full border-2 border-accent border-t-transparent"></div>
            </div>
          {/if}
        {/if}
      </section>
    </div>
  </main>
</div>
