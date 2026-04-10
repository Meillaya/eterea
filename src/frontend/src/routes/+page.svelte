<script lang="ts">
  import { onMount } from 'svelte';
  import BookmarkList from '$lib/components/BookmarkList.svelte';
  import DateFilter from '$lib/components/DateFilter.svelte';
  import Header from '$lib/components/Header.svelte';
  import ImportModal from '$lib/components/ImportModal.svelte';
  import LayoutToggle from '$lib/components/LayoutToggle.svelte';
  import SearchBar from '$lib/components/SearchBar.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import { bookmarks, dateRange, feedMeta, isLoading, isLoadingMore, isRefreshing, searchQuery, selectedTag, viewMode } from '$lib/stores/bookmarks.svelte';
  import { hydrateCachedLibrarySnapshot, loadMoreBookmarks, loadStats, refreshBookmarks } from '$lib/api';

  let showImportModal = $state(false);
  let sidebarCollapsed = $state(false);
  let ready = $state(false);
  let lastSignature = $state('');
  let lastFocusRefreshAt = $state(0);
  const FOCUS_REFRESH_COOLDOWN_MS = 30_000;

  function showAllBookmarks() {
    selectedTag.clear();
    dateRange.clear();
    viewMode.set('all');
  }

  function showRecentBookmarks() {
    selectedTag.clear();
    const to = new Date();
    const from = new Date();
    from.setDate(from.getDate() - 30);
    dateRange.set(from.toISOString(), to.toISOString());
    viewMode.set('recent');
  }

  function showFavoriteBookmarks() {
    selectedTag.clear();
    viewMode.set('favorites');
  }

  function filterSignature(): string {
    return JSON.stringify({
      query: searchQuery.value,
      tag: selectedTag.value,
      from: dateRange.value.from,
      to: dateRange.value.to,
      view: viewMode.value
    });
  }

  onMount(async () => {
    hydrateCachedLibrarySnapshot();
    lastSignature = filterSignature();
    ready = true;
    lastFocusRefreshAt = Date.now();
    await refreshBookmarks();
    void loadStats({ suppressErrors: true });
  });

  async function refreshVisibleLibrary() {
    const now = Date.now();
    if (now - lastFocusRefreshAt < FOCUS_REFRESH_COOLDOWN_MS) return;
    lastFocusRefreshAt = now;
    await refreshBookmarks();
    void loadStats({ suppressErrors: true });
  }

  $effect(() => {
    const signature = filterSignature();
    if (!ready || signature === lastSignature) return;
    lastSignature = signature;
    void refreshBookmarks();
  });

  const activeFilterCount = $derived([
    searchQuery.value ? 1 : 0,
    selectedTag.value ? 1 : 0,
    dateRange.value.from ? 1 : 0,
    viewMode.value === 'favorites' ? 1 : 0,
    viewMode.value === 'recent' ? 1 : 0
  ].reduce((total, value) => total + value, 0));

  const mobileNavItems = [
    { id: 'all', label: 'Library', action: showAllBookmarks },
    { id: 'recent', label: 'Recent', action: showRecentBookmarks },
    { id: 'favorites', label: 'Favorites', action: showFavoriteBookmarks }
  ] as const;

  const activeChips = $derived.by(() => {
    const chips: string[] = [];
    if (searchQuery.value) {
      chips.push(`Search: ${searchQuery.value}`);
    }
    if (selectedTag.value) {
      chips.push(`#${selectedTag.value}`);
    }
    if (viewMode.value === 'favorites') {
      chips.push('Favorites only');
    }
    if (viewMode.value === 'recent') {
      chips.push('Recent 30 days');
    }
    if (dateRange.value.from && dateRange.value.to) {
      const formatOptions: Intl.DateTimeFormatOptions = {
        month: 'short',
        day: 'numeric'
      };
      chips.push(`${new Date(dateRange.value.from).toLocaleDateString('en-US', formatOptions)} – ${new Date(dateRange.value.to).toLocaleDateString('en-US', formatOptions)}`);
    }
    return chips;
  });

  const layoutSummary = $derived.by(() => {
    if (layoutMode.value === 'cards') return 'Grid layout';
    if (layoutMode.value === 'compact') return 'List layout';
    return 'Focus layout';
  });
</script>

<svelte:window onfocus={refreshVisibleLibrary} />

<svelte:head>
  <title>Eterea - Your Bookmarks</title>
</svelte:head>

<div class="flex min-h-screen bg-bg-primary text-text-primary">
  <Sidebar collapsed={sidebarCollapsed} ontoggle={() => (sidebarCollapsed = !sidebarCollapsed)} />

  <div class="min-w-0 flex-1">
    <Header onimport={() => (showImportModal = true)} />

    <main class="mx-auto flex max-w-7xl flex-col gap-6 px-4 py-6 sm:px-6 lg:px-8 xl:px-10">
      <section class="panel rounded-[2rem] px-6 py-6">
        <div class="flex flex-col gap-6 xl:flex-row xl:items-end xl:justify-between">
          <div class="max-w-3xl">
            <p class="eyebrow">Bookmark library</p>
            <h2 class="mt-3 font-display text-[2.5rem] leading-none italic text-text-primary sm:text-[3.2rem]">A calmer place to read what you saved.</h2>
            <p class="mt-3 max-w-2xl text-sm leading-7 text-text-secondary sm:text-base">Keep the archive local, searchable, and easy to revisit. Search fast, move between layouts, and pull in new bookmarks without turning the app into a dashboard.</p>
          </div>

          <div class="grid gap-3 sm:grid-cols-3 xl:min-w-[420px]">
            <div class="rounded-[1.4rem] border border-border-subtle bg-bg-secondary/60 px-4 py-4">
              <p class="text-xs uppercase tracking-wide text-text-muted">Visible now</p>
              <p class="mt-2 text-2xl font-mono text-text-primary">{bookmarks.value.length.toLocaleString()}</p>
            </div>
            <div class="rounded-[1.4rem] border border-border-subtle bg-bg-secondary/60 px-4 py-4">
              <p class="text-xs uppercase tracking-wide text-text-muted">Archive total</p>
              <p class="mt-2 text-2xl font-mono text-text-primary">{feedMeta.value.total.toLocaleString()}</p>
            </div>
            <div class="rounded-[1.4rem] border border-border-subtle bg-bg-secondary/60 px-4 py-4">
              <p class="text-xs uppercase tracking-wide text-text-muted">Active filters</p>
              <p class="mt-2 text-2xl font-mono text-text-primary">{activeFilterCount}</p>
            </div>
          </div>
        </div>

        <div class="mt-5 flex flex-wrap items-center gap-3 text-sm text-text-secondary">
          <span class="rounded-full border border-border bg-bg-secondary/70 px-4 py-2">Press <kbd class="rounded-full border border-border-subtle bg-bg-tertiary px-2 py-0.5 font-mono text-[11px] text-text-muted">/</kbd> to jump into search</span>
          {#if viewMode.value === 'favorites'}
            <span class="rounded-full bg-yellow-500/15 px-3 py-2 text-yellow-400">Favorites view</span>
          {:else if viewMode.value === 'recent'}
            <span class="rounded-full bg-accent/15 px-3 py-2 text-accent">Recent 30 days</span>
          {/if}
          <a href="/settings" class="rounded-full border border-border bg-bg-secondary/70 px-4 py-2 transition-colors hover:border-accent hover:text-text-primary">Open settings</a>
          <button onclick={() => (showImportModal = true)} class="rounded-full border border-border bg-bg-secondary/70 px-4 py-2 text-text-primary transition-colors hover:border-accent hover:text-accent">Import bookmarks</button>
        </div>
      </section>

      <section class="panel rounded-[2rem] px-5 py-5">
        <div class="grid gap-4 xl:grid-cols-[minmax(0,1fr),auto] xl:items-center">
          <SearchBar />
          <div class="flex flex-wrap items-center justify-start gap-2 xl:justify-end">
            <LayoutToggle />
            <DateFilter />
          </div>
        </div>

        {#if activeChips.length > 0}
          <div class="mt-4 flex flex-wrap gap-2">
            {#each activeChips as chip}
              <span class="rounded-full border border-border-subtle bg-bg-secondary/60 px-3 py-1.5 text-xs text-text-secondary">{chip}</span>
            {/each}
          </div>
        {/if}
      </section>

      <section class="min-h-0 flex-1">
        <div class="mb-4 flex items-center justify-between gap-4">
          <div>
            <p class="eyebrow">Reading feed</p>
            <h3 class="mt-1 text-xl font-medium text-text-primary">Saved posts</h3>
          </div>
          <div class="flex items-center gap-3">
            {#if isRefreshing.value}
              <span class="rounded-full border border-accent/30 bg-accent/10 px-3 py-1 text-xs font-medium text-accent">Refreshing…</span>
            {/if}
            {#if feedMeta.value.total > 0}
              <p class="text-sm text-text-muted">{feedMeta.value.total.toLocaleString()} bookmarks in your archive</p>
            {/if}
          </div>
        </div>

        {#if isLoading.value && bookmarks.value.length === 0}
          <div class="panel flex h-72 items-center justify-center rounded-[2rem]">
            <div class="h-10 w-10 animate-spin rounded-full border-2 border-accent border-t-transparent"></div>
          </div>
        {:else if bookmarks.value.length === 0}
          <div class="panel flex h-80 flex-col items-center justify-center rounded-[2rem] px-6 text-center">
            <div class="flex h-16 w-16 items-center justify-center rounded-full border border-border bg-bg-secondary/70 text-text-muted">
              <svg class="h-8 w-8" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z" /></svg>
            </div>
            <h3 class="mt-5 text-2xl font-medium text-text-primary">Nothing to read yet</h3>
            <p class="mt-3 max-w-lg text-sm leading-7 text-text-secondary">Import a file, connect X, or clear filters to refill the library. Your archive stays local, searchable, and ready to revisit.</p>
            <div class="mt-6 flex flex-wrap justify-center gap-3">
              <button onclick={() => (showImportModal = true)} class="rounded-full bg-accent px-5 py-2.5 text-sm font-medium text-white transition-opacity hover:opacity-90">Import bookmarks</button>
              <span class="rounded-full border border-border px-4 py-2.5 text-sm text-text-muted">CSV · JSON · X archive JS</span>
            </div>
          </div>
        {:else}
          <BookmarkList items={bookmarks.value} />

          {#if feedMeta.value.hasMore}
            <div class="py-7 text-center">
              <button
                onclick={loadMoreBookmarks}
                disabled={isLoadingMore.value}
                class="rounded-full border border-border bg-bg-secondary/70 px-5 py-2.5 text-sm text-text-primary transition-colors hover:border-accent hover:text-accent disabled:cursor-wait disabled:opacity-70"
              >
                {isLoadingMore.value ? 'Loading more…' : 'Load more bookmarks'}
              </button>
            </div>
          {:else}
            <p class="py-6 text-center text-sm text-text-muted">You’re all caught up.</p>
          {/if}
        {/if}
      </section>
    </main>
  </div>
</div>

{#if showImportModal}
  <ImportModal onclose={() => (showImportModal = false)} />
{/if}
