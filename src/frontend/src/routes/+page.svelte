<script lang="ts">
  import { onMount } from 'svelte';
  import Header from '$lib/components/Header.svelte';
  import SearchBar from '$lib/components/SearchBar.svelte';
  import BookmarkList from '$lib/components/BookmarkList.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import ImportModal from '$lib/components/ImportModal.svelte';
  import DateFilter from '$lib/components/DateFilter.svelte';
  import LayoutToggle from '$lib/components/LayoutToggle.svelte';
  import { bookmarks, searchQuery, selectedTag, isLoading, stats, viewMode, feedMeta } from '$lib/stores/bookmarks.svelte';
  import { loadBookmarks, loadStats, searchBookmarks, loadMoreBookmarks } from '$lib/api';
  
  let showImportModal = $state(false);
  let sidebarCollapsed = $state(false);
  let lastQuery: string | null = null;
  let lastTag: string | null = null;
  
  onMount(async () => {
    await loadStats();
    await loadBookmarks({ reset: true });
  });
  
  // Reactive search
  $effect(() => {
    const query = searchQuery.value;
    const tag = selectedTag.value;
    if (query === lastQuery && tag === lastTag) return;
    lastQuery = query;
    lastTag = tag;
    
    if (query || tag) {
      searchBookmarks(query, tag);
    } else {
      loadBookmarks({ reset: true });
    }
  });
</script>

<svelte:head>
  <title>Eterea - Your Bookmarks</title>
</svelte:head>

<div class="flex h-screen overflow-hidden bg-bg-primary">
  <!-- Sidebar -->
  <Sidebar 
    collapsed={sidebarCollapsed} 
    ontoggle={() => sidebarCollapsed = !sidebarCollapsed}
  />
  
  <!-- Main Content -->
  <main class="flex-1 flex flex-col overflow-hidden">
    <Header onimport={() => showImportModal = true} />
    
    <div class="flex-1 overflow-hidden">
      <div class="h-full max-w-6xl mx-auto flex flex-col px-4 sm:px-6 lg:px-10">
        <!-- Search Area -->
        <div class="py-6">
          <SearchBar />
        </div>
      
        <!-- Stats Bar & Filters -->
        <div class="flex flex-col gap-4 pb-4 md:flex-row md:items-center md:justify-between">
          {#if stats.value}
            <div class="grid grid-cols-2 sm:grid-cols-4 gap-3 text-sm text-text-secondary">
              <div class="flex flex-col bg-bg-secondary/70 border border-border rounded-xl p-3">
                <span class="text-xs text-text-muted uppercase tracking-wide">Bookmarks</span>
                <span class="text-lg font-mono text-text-primary">{stats.value.total_bookmarks.toLocaleString()}</span>
              </div>
              <div class="flex flex-col bg-bg-secondary/70 border border-border rounded-xl p-3">
                <span class="text-xs text-text-muted uppercase tracking-wide">Favorites</span>
                <span class="text-lg font-mono text-text-primary">{stats.value.favorite_bookmarks.toLocaleString()}</span>
              </div>
              <div class="flex flex-col bg-bg-secondary/70 border border-border rounded-xl p-3">
                <span class="text-xs text-text-muted uppercase tracking-wide">Authors</span>
                <span class="text-lg font-mono text-text-primary">{stats.value.unique_authors.toLocaleString()}</span>
              </div>
              <div class="flex flex-col bg-bg-secondary/70 border border-border rounded-xl p-3">
                <span class="text-xs text-text-muted uppercase tracking-wide">Tags</span>
                <span class="text-lg font-mono text-text-primary">{stats.value.unique_tags}</span>
              </div>
            </div>
          {:else}
            <div class="h-16"></div>
          {/if}
          
          <!-- Filters -->
          <div class="flex items-center gap-3 flex-wrap justify-end">
            <LayoutToggle />
            <DateFilter />
            
            {#if viewMode.value === 'favorites'}
              <span class="px-3 py-1.5 text-sm bg-yellow-500/20 text-yellow-500 rounded-lg">
                ★ Favorites
              </span>
            {/if}
          </div>
        </div>
      
        <!-- Bookmarks List -->
        <div class="flex-1 overflow-y-auto pb-6">
          {#if isLoading.value}
            <div class="flex items-center justify-center h-64">
              <div class="animate-spin w-8 h-8 border-2 border-accent border-t-transparent rounded-full"></div>
            </div>
          {:else if bookmarks.value.length === 0}
            <div class="flex flex-col items-center justify-center h-64 text-text-secondary">
              <svg class="w-16 h-16 mb-4 opacity-50" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z" />
              </svg>
              <p class="text-lg">No bookmarks yet</p>
              <p class="text-sm mt-1">Import your Twitter bookmarks to get started</p>
              <button 
                onclick={() => showImportModal = true}
                class="mt-4 px-4 py-2 bg-accent text-white rounded-lg hover:bg-accent/90 transition-colors"
              >
                Import Bookmarks
              </button>
            </div>
          {:else}
            <BookmarkList items={bookmarks.value} />
            
            {#if feedMeta.value.hasMore}
              <div class="py-6 text-center">
                <button
                  onclick={loadMoreBookmarks}
                  class="px-5 py-2 border border-border rounded-full text-sm text-text-primary hover:border-accent hover:text-accent transition-colors"
                >
                  Load more bookmarks
                </button>
              </div>
            {:else}
              <p class="text-center text-sm text-text-muted py-4">You’re all caught up.</p>
            {/if}
          {/if}
        </div>
      </div>
    </div>
  </main>
</div>

<!-- Import Modal -->
{#if showImportModal}
  <ImportModal onclose={() => showImportModal = false} />
{/if}

