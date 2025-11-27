<script lang="ts">
  import { searchQuery, selectedTag } from '$lib/stores/bookmarks.svelte';
  
  let inputValue = $state('');
  let inputRef: HTMLInputElement;
  let debounceTimer: ReturnType<typeof setTimeout>;
  
  function handleInput(e: Event) {
    const value = (e.target as HTMLInputElement).value;
    inputValue = value;
    
    // Debounce search
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      searchQuery.set(value);
    }, 150);
  }
  
  function handleClear() {
    inputValue = '';
    searchQuery.set('');
    selectedTag.clear();
    inputRef?.focus();
  }
  
  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      handleClear();
    }
  }
  
  // Global keyboard shortcut
  function handleGlobalKeydown(e: KeyboardEvent) {
    if (e.key === '/' && !['INPUT', 'TEXTAREA'].includes((e.target as HTMLElement).tagName)) {
      e.preventDefault();
      inputRef?.focus();
    }
  }
</script>

<svelte:window onkeydown={handleGlobalKeydown} />

<div class="relative group">
  <div class="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none">
    <svg class="w-5 h-5 text-text-muted group-focus-within:text-accent transition-colors" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
    </svg>
  </div>
  
  <input
    bind:this={inputRef}
    type="text"
    placeholder="Search bookmarks... (press / to focus)"
    value={inputValue}
    oninput={handleInput}
    onkeydown={handleKeydown}
    class="w-full pl-12 pr-12 py-3 bg-bg-secondary border border-border rounded-xl
           text-text-primary placeholder:text-text-muted
           focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/50
           transition-all duration-200"
  />
  
  {#if inputValue || selectedTag.value}
    <button
      onclick={handleClear}
      class="absolute inset-y-0 right-0 pr-4 flex items-center text-text-muted hover:text-text-primary transition-colors"
      title="Clear search"
      aria-label="Clear search"
    >
      <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
      </svg>
    </button>
  {:else}
    <div class="absolute inset-y-0 right-0 pr-4 flex items-center pointer-events-none">
      <kbd class="px-2 py-1 text-xs text-text-muted bg-bg-tertiary rounded border border-border-subtle font-mono">/</kbd>
    </div>
  {/if}
</div>

<!-- Active filters -->
{#if selectedTag.value}
  <div class="flex items-center gap-2 mt-3">
    <span class="text-sm text-text-secondary">Filtering by:</span>
    <button
      onclick={() => selectedTag.clear()}
      class="inline-flex items-center gap-1 px-3 py-1 bg-accent/20 text-accent text-sm rounded-full
             hover:bg-accent/30 transition-colors"
    >
      #{selectedTag.value}
      <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
      </svg>
    </button>
  </div>
{/if}

