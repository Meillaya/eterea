<script lang="ts">
  import { allTags, selectedTag, viewMode } from '$lib/stores/bookmarks.svelte';
  import { loadBookmarks, loadFavorites } from '$lib/api';
  
  interface Props {
    collapsed: boolean;
    ontoggle: () => void;
  }
  
  let { collapsed, ontoggle }: Props = $props();
  
  function handleAllBookmarks() {
    selectedTag.clear();
    viewMode.set('all');
    loadBookmarks({ reset: true });
  }
  
  function handleRecent() {
    selectedTag.clear();
    viewMode.set('recent');
    loadBookmarks({ reset: true });
  }
  
  function handleFavorites() {
    selectedTag.clear();
    viewMode.set('favorites');
    loadFavorites();
  }
  
  const navItems = [
    { id: 'all', icon: 'bookmark', label: 'All Bookmarks', action: handleAllBookmarks },
    { id: 'recent', icon: 'clock', label: 'Recent', action: handleRecent },
    { id: 'favorites', icon: 'star', label: 'Favorites', action: handleFavorites },
  ];
  
  function getNavItemClasses(itemId: string): string {
    const base = 'w-full flex items-center gap-3 px-3 py-2 rounded-lg transition-colors';
    if (viewMode.value === itemId) {
      return `${base} bg-accent/20 text-accent`;
    }
    return `${base} text-text-secondary hover:text-text-primary hover:bg-bg-tertiary`;
  }
  
  function getTagClasses(tag: string): string {
    const base = 'w-full flex items-center justify-between px-3 py-1.5 rounded-lg text-sm transition-colors';
    if (selectedTag.value === tag) {
      return `${base} bg-accent/20 text-accent`;
    }
    return `${base} text-text-secondary hover:text-text-primary hover:bg-bg-tertiary`;
  }
</script>

<aside 
  class="flex flex-col border-r border-border-subtle bg-bg-secondary/50 transition-all duration-300"
  class:w-16={collapsed}
  class:w-64={!collapsed}
>
  <!-- Toggle Button -->
  <button
    onclick={ontoggle}
    class="p-4 flex items-center justify-center text-text-muted hover:text-text-primary transition-colors"
    title={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
  >
    <svg 
      class="w-5 h-5 transition-transform duration-300" 
      class:rotate-180={collapsed}
      fill="none" 
      viewBox="0 0 24 24" 
      stroke="currentColor"
    >
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 19l-7-7 7-7m8 14l-7-7 7-7" />
    </svg>
  </button>
  
  <!-- Navigation -->
  <nav class="flex-1 px-3 py-2">
    <ul class="space-y-1">
      {#each navItems as item}
        <li>
          <button
            onclick={item.action}
            class={getNavItemClasses(item.id)}
            class:justify-center={collapsed}
          >
            {#if item.icon === 'bookmark'}
              <svg class="w-5 h-5 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z" />
              </svg>
            {:else if item.icon === 'clock'}
              <svg class="w-5 h-5 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            {:else if item.icon === 'star'}
              <svg class="w-5 h-5 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z" />
              </svg>
            {/if}
            
            {#if !collapsed}
              <span class="truncate">{item.label}</span>
            {/if}
          </button>
        </li>
      {/each}
    </ul>
    
    <!-- Tags Section -->
    {#if !collapsed && allTags.value.length > 0}
      <div class="mt-6">
        <h3 class="px-3 mb-2 text-xs font-medium text-text-muted uppercase tracking-wider">
          Tags
        </h3>
        <ul class="space-y-1">
          {#each allTags.value.slice(0, 10) as [tag, count]}
            <li>
              <button
                onclick={() => selectedTag.set(tag)}
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
  
  <!-- Footer -->
  {#if !collapsed}
    <div class="p-4 border-t border-border-subtle">
      <p class="text-xs text-text-muted text-center">
        Built with 🦀 + ⚡
      </p>
    </div>
  {/if}
</aside>
