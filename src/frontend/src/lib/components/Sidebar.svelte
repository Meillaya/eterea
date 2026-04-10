<script lang="ts">
  import { allTags, dateRange, selectedTag, viewMode } from '$lib/stores/bookmarks.svelte';

  interface Props {
    collapsed: boolean;
    ontoggle: () => void;
  }

  let { collapsed, ontoggle }: Props = $props();

  function handleAllBookmarks() {
    selectedTag.clear();
    dateRange.clear();
    viewMode.set('all');
  }

  function handleRecent() {
    selectedTag.clear();
    const to = new Date();
    const from = new Date();
    from.setDate(from.getDate() - 30);
    dateRange.set(from.toISOString(), to.toISOString());
    viewMode.set('recent');
  }

  function handleFavorites() {
    selectedTag.clear();
    viewMode.set('favorites');
  }

  const navItems = [
    { id: 'all', icon: 'bookmark', label: 'Library', description: 'All saved posts', action: handleAllBookmarks },
    { id: 'recent', icon: 'clock', label: 'Recent', description: 'Last 30 days', action: handleRecent },
    { id: 'favorites', icon: 'star', label: 'Favorites', description: 'Pinned highlights', action: handleFavorites }
  ];

  function getNavItemClasses(itemId: string): string {
    const base = 'group w-full rounded-2xl border px-3 py-3 text-left transition-all';
    if (viewMode.value === itemId) {
      return `${base} border-border-accent bg-accent/10 text-text-primary shadow-[var(--shadow-glow)]`;
    }
    return `${base} border-transparent bg-transparent text-text-secondary hover:border-border hover:bg-bg-secondary/70 hover:text-text-primary`;
  }

  function getTagClasses(tag: string): string {
    const base = 'flex w-full items-center justify-between rounded-full px-3 py-2 text-sm transition-colors';
    if (selectedTag.value === tag) {
      return `${base} bg-accent/15 text-accent`;
    }
    return `${base} text-text-secondary hover:bg-bg-secondary/70 hover:text-text-primary`;
  }
</script>

<aside
  class="hidden border-r border-border-subtle/80 bg-bg-primary/65 backdrop-blur-xl lg:flex lg:flex-col"
  class:w-24={collapsed}
  class:w-[286px]={!collapsed}
>
  <div class="flex items-center justify-between px-4 py-5">
    {#if !collapsed}
      <div>
        <p class="eyebrow">Navigation</p>
        <p class="mt-1 text-sm text-text-secondary">Browse and filter your archive.</p>
      </div>
    {/if}
    <button
      onclick={ontoggle}
      class="flex h-10 w-10 items-center justify-center rounded-2xl border border-border bg-bg-secondary/75 text-text-secondary transition-colors hover:text-text-primary"
      title={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
    >
      <svg class="h-5 w-5 transition-transform duration-300" class:rotate-180={collapsed} fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M11 19l-7-7 7-7m8 14l-7-7 7-7" />
      </svg>
    </button>
  </div>

  <nav class="flex-1 px-4 pb-4">
    <ul class="space-y-2">
      {#each navItems as item}
        <li>
          <button
            onclick={item.action}
            class={getNavItemClasses(item.id)}
            class:items-center={collapsed}
          >
            <div class="flex items-center gap-3">
              <span class="flex h-10 w-10 items-center justify-center rounded-2xl border border-border-subtle bg-bg-secondary/75 text-current">
                {#if item.icon === 'bookmark'}
                  <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.6" d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z" /></svg>
                {:else if item.icon === 'clock'}
                  <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.6" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
                {:else}
                  <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.6" d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z" /></svg>
                {/if}
              </span>
              {#if !collapsed}
                <div>
                  <p class="font-medium text-text-primary">{item.label}</p>
                  <p class="text-xs text-text-muted">{item.description}</p>
                </div>
              {/if}
            </div>
          </button>
        </li>
      {/each}
    </ul>

    {#if !collapsed && allTags.value.length > 0}
      <div class="mt-8 rounded-[1.5rem] border border-border bg-bg-secondary/45 p-4">
        <div class="mb-3 flex items-center justify-between">
          <div>
            <p class="eyebrow">Top tags</p>
            <p class="mt-1 text-sm text-text-secondary">Quick pivots through recurring themes.</p>
          </div>
        </div>
        <ul class="space-y-1.5">
          {#each allTags.value.slice(0, 8) as [tag, count]}
            <li>
              <button
                onclick={() => {
                  viewMode.set('all');
                  selectedTag.set(tag);
                }}
                class={getTagClasses(tag)}
              >
                <span class="truncate">#{tag}</span>
                <span class="font-mono text-xs opacity-60">{count}</span>
              </button>
            </li>
          {/each}
        </ul>
      </div>
    {/if}
  </nav>

  {#if !collapsed}
    <div class="px-4 pb-5">
      <div class="rounded-[1.4rem] border border-border-subtle bg-bg-secondary/55 p-4 text-sm text-text-secondary">
        <p class="font-medium text-text-primary">Reading workspace</p>
        <p class="mt-1">Search fast, import deliberately, keep the archive local.</p>
        <a href="/settings" class="mt-3 inline-flex text-sm text-accent transition-colors hover:text-text-primary">
          Tune the workspace →
        </a>
      </div>
    </div>
  {/if}
</aside>
