<script lang="ts">
  import { searchQuery, selectedAuthor, selectedTag } from '$lib/stores/bookmarks.svelte';

  let inputValue = $state('');
  let inputRef: HTMLInputElement;
  let debounceTimer: ReturnType<typeof setTimeout> | undefined;

  $effect(() => {
    // Sync external changes back to input (e.g. filter chip clear)
    const externalQuery = searchQuery.value;
    const externalAuthor = selectedAuthor.value;

    if (externalAuthor && inputValue !== '@' + externalAuthor) {
      // Author was set externally (or changed) — sync @handle to input box
      inputValue = '@' + externalAuthor;
    } else if (!externalAuthor && inputValue.startsWith('@')) {
      // Author was cleared externally — clear the @handle input
      inputValue = '';
    } else if (!inputValue.startsWith('@') && externalQuery !== inputValue) {
      inputValue = externalQuery;
    }
  });

  function handleInput(event: Event) {
    const value = (event.target as HTMLInputElement).value;
    inputValue = value;

    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      if (value.startsWith('@') && value.length > 1) {
        selectedAuthor.set(value.slice(1).trim());
        if (searchQuery.value) searchQuery.set('');
      } else {
        if (selectedAuthor.value) selectedAuthor.clear();
        searchQuery.set(value);
      }
    }, 100);
  }

  function handleClear() {
    inputValue = '';
    searchQuery.set('');
    selectedTag.clear();
    selectedAuthor.clear();
    inputRef?.focus();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      handleClear();
    }
  }

  function handleGlobalKeydown(event: KeyboardEvent) {
    if (event.key === '/' && !['INPUT', 'TEXTAREA'].includes((event.target as HTMLElement).tagName)) {
      event.preventDefault();
      inputRef?.focus();
    }
  }
</script>

<svelte:window onkeydown={handleGlobalKeydown} />

<div class="relative overflow-hidden rounded-[1.6rem] border border-border bg-bg-secondary/85 px-4 py-3 shadow-[var(--shadow-soft)]">
  <div class="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-5 text-text-muted">
    <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M21 21l-5.5-5.5M16 10.5a5.5 5.5 0 11-11 0 5.5 5.5 0 0111 0z" />
    </svg>
  </div>

  <input
    bind:this={inputRef}
    type="text"
    class="w-full border-none bg-transparent pl-9 pr-18 text-base text-text-primary placeholder:text-text-muted focus:outline-none"
    placeholder="Search text, or type @handle to filter by author"
    value={inputValue}
    oninput={handleInput}
    onkeydown={handleKeydown}
  />

  {#if inputValue || selectedTag.value || selectedAuthor.value}
    <button
      class="absolute inset-y-0 right-0 flex items-center pr-5 text-text-muted transition-colors hover:text-text-primary"
      onclick={handleClear}
      title="Clear search"
      aria-label="Clear search"
    >
      <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M6 18L18 6M6 6l12 12" />
      </svg>
    </button>
  {:else}
    <div class="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-5">
      <kbd class="rounded-full border border-border-subtle bg-bg-tertiary px-2.5 py-1 text-[11px] text-text-muted">/</kbd>
    </div>
  {/if}
</div>
