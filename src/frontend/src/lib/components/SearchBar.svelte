<script lang="ts">
  import { searchQuery, selectedTag } from '$lib/stores/bookmarks.svelte';

  let inputValue = $state('');
  let inputRef: HTMLInputElement;
  let debounceTimer: ReturnType<typeof setTimeout>;

  function handleInput(e: Event) {
    const value = (e.target as HTMLInputElement).value;
    inputValue = value;
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

  function handleGlobalKeydown(e: KeyboardEvent) {
    if (e.key === '/' && !['INPUT', 'TEXTAREA'].includes((e.target as HTMLElement).tagName)) {
      e.preventDefault();
      inputRef?.focus();
    }
  }
</script>

<svelte:window onkeydown={handleGlobalKeydown} />

<div class="space-y-3">
  <div class="relative overflow-hidden rounded-[1.5rem] border border-border bg-bg-secondary/75 px-4 py-3 shadow-[var(--shadow-soft)]">
    <div class="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-5">
      <svg class="h-5 w-5 text-text-muted" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
      </svg>
    </div>

    <input
      bind:this={inputRef}
      type="text"
      placeholder="Search your archive by text, author, or tag"
      value={inputValue}
      oninput={handleInput}
      onkeydown={handleKeydown}
      class="w-full border-none bg-transparent pl-9 pr-14 text-base text-text-primary placeholder:text-text-muted focus:outline-none"
    />

    {#if inputValue || selectedTag.value}
      <button
        onclick={handleClear}
        class="absolute inset-y-0 right-0 flex items-center pr-5 text-text-muted transition-colors hover:text-text-primary"
        title="Clear search"
        aria-label="Clear search"
      >
        <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    {:else}
      <div class="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-5">
        <kbd class="rounded-full border border-border-subtle bg-bg-tertiary px-2.5 py-1 text-[11px] font-mono text-text-muted">/</kbd>
      </div>
    {/if}
  </div>

  {#if selectedTag.value}
    <div class="flex items-center gap-2 text-sm text-text-secondary">
      <span>Filtering by</span>
      <button
        onclick={() => selectedTag.clear()}
        class="inline-flex items-center gap-1 rounded-full bg-accent/15 px-3 py-1 text-accent transition-colors hover:bg-accent/25"
      >
        #{selectedTag.value}
        <svg class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </div>
  {/if}
</div>
