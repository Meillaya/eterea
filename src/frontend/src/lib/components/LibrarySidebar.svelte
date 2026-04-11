<script lang="ts">
  import { allTags, dateRange, feedMeta, searchQuery, selectedAuthor, selectedTag, stats, viewMode } from '$lib/stores/bookmarks.svelte';

  interface Props {
    onopenimport: () => void;
  }

  let { onopenimport }: Props = $props();

  let tagSearch = $state('');
  const filteredTags = $derived(
    tagSearch.trim()
      ? allTags.value.filter(([tag]) => tag.toLowerCase().includes(tagSearch.toLowerCase()))
      : allTags.value,
  );

  function showLibrary() {
    searchQuery.set('');
    selectedTag.clear();
    selectedAuthor.clear();
    dateRange.clear();
    viewMode.set('all');
  }

  function showRecent() {
    searchQuery.set('');
    selectedTag.clear();
    selectedAuthor.clear();
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
    dateRange.clear();
    viewMode.set('favorites');
  }

  function activateTag(tag: string) {
    if (selectedTag.value === tag) {
      selectedTag.clear();
      return;
    }
    searchQuery.set('');
    selectedAuthor.clear();
    dateRange.clear();
    selectedTag.set(tag);
    viewMode.set('all');
  }

  function navClass(id: string) {
    const active = viewMode.value === id;
    return `w-full rounded-[1.35rem] border px-4 py-3 text-left transition-all ${active ? 'border-border-accent bg-[color:var(--accent-soft)] text-text-primary' : 'border-transparent bg-transparent text-text-secondary'}`;
  }

  const navItems = [
    { id: 'all', label: 'Library', description: 'Everything you saved', action: showLibrary },
    { id: 'recent', label: 'Recent', description: 'Last 30 days', action: showRecent },
    { id: 'favorites', label: 'Favorites', description: 'Pinned to revisit', action: showFavorites },
  ] as const;
</script>

<aside class="hidden w-[290px] shrink-0 xl:block">
  <div class="sticky top-4 space-y-4 px-4 pb-4 pt-4">
    <div class="surface-panel rounded-[2rem] p-5">
      <div class="flex items-center gap-3">
        <div class="flex h-12 w-12 items-center justify-center rounded-[1.4rem] border border-border bg-bg-secondary/85 text-accent shadow-[var(--shadow-glow)]">
          <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.7" d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z" />
          </svg>
        </div>
        <div>
          <p class="section-label">Local-first archive</p>
          <h2 class="font-display text-[2rem] italic text-text-primary">Eterea</h2>
        </div>
      </div>

      <p class="mt-4 text-sm leading-7 text-text-secondary">
        A calm reading room for saved tweets — fast to open, quiet to browse, easy to keep useful.
      </p>

      <div class="mt-4 flex flex-wrap gap-2 text-xs">
        <span class="pill">{feedMeta.value.total.toLocaleString()} saved</span>
        {#if stats.value}
          <span class="pill">{stats.value.unique_authors.toLocaleString()} authors</span>
          <span class="pill">{stats.value.favorite_bookmarks.toLocaleString()} favorites</span>
          {#if stats.value.earliest_date && stats.value.latest_date}
            <span class="pill" title="{new Date(stats.value.earliest_date).toLocaleDateString('en-US', { month: 'short', year: 'numeric' })} – {new Date(stats.value.latest_date).toLocaleDateString('en-US', { month: 'short', year: 'numeric' })}">
              {new Date(stats.value.earliest_date).getFullYear()}–{new Date(stats.value.latest_date).getFullYear()}
            </span>
          {/if}
        {/if}
      </div>

      <div class="mt-5 grid gap-2">
        <button class="accent-button w-full" onclick={onopenimport}>Import bookmarks</button>
        <a class="ghost-button w-full" href="/settings">Open settings</a>
      </div>
    </div>

    <div class="soft-panel rounded-[1.8rem] p-4">
      <p class="section-label">Navigate</p>
      <div class="mt-3 space-y-2">
        {#each navItems as item}
          <button class={navClass(item.id)} onclick={item.action}>
            <div class="flex items-center justify-between gap-3">
              <div>
                <p class="font-medium">{item.label}</p>
                <p class="mt-1 text-xs text-text-muted">{item.description}</p>
              </div>
              <span class="text-text-muted">→</span>
            </div>
          </button>
        {/each}
      </div>
    </div>

    <div class="soft-panel rounded-[1.8rem] p-4">
      <div class="flex items-center justify-between gap-3">
        <p class="section-label">Tags</p>
        {#if allTags.value.length > 0}
          <span class="pill">{allTags.value.length}</span>
        {/if}
      </div>

      {#if allTags.value.length > 0}
        {#if allTags.value.length > 8}
          <div class="relative mt-3">
            <input
              type="text"
              placeholder="Filter tags…"
              bind:value={tagSearch}
              class="w-full rounded-[1rem] border border-border bg-bg-tertiary/80 px-3 py-2 text-xs text-text-primary placeholder:text-text-muted focus:border-accent focus:outline-none"
            />
          </div>
        {/if}

        <div class="mt-3 max-h-64 space-y-1 overflow-y-auto pr-0.5">
          {#each filteredTags as [tag, count]}
            {@const isActive = selectedTag.value === tag}
            <button
              class={`flex w-full items-center justify-between rounded-[0.9rem] border px-3 py-1.5 text-xs transition-colors ${isActive ? 'border-border-accent bg-[color:var(--accent-soft)] text-accent' : 'border-transparent text-text-secondary hover:border-border-subtle hover:text-text-primary'}`}
              onclick={() => activateTag(tag)}
            >
              <span>#{tag}</span>
              <span class="font-mono text-[11px] text-text-muted">{count}</span>
            </button>
          {/each}
          {#if filteredTags.length === 0}
            <p class="py-2 text-xs text-text-muted">No tags match "{tagSearch}"</p>
          {/if}
        </div>
      {:else}
        <p class="mt-4 text-sm text-text-muted">Tags appear once archive metadata loads.</p>
      {/if}
    </div>
  </div>
</aside>
