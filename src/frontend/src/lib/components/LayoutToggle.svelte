<script lang="ts">
  import { layoutMode, type LayoutMode } from '$lib/stores/bookmarks.svelte';
  
  const modes: { id: LayoutMode; label: string }[] = [
    { id: 'default', label: 'Default' },
    { id: 'cards', label: 'Cards' },
    { id: 'compact', label: 'Compact' }
  ];
  
  function setLayout(mode: LayoutMode) {
    layoutMode.set(mode);
  }
</script>

<div class="inline-flex rounded-full border border-border-subtle bg-bg-tertiary text-xs">
  {#each modes as mode}
    <button
      class="px-3 py-1 rounded-full transition-colors"
      class:bg-accent={layoutMode.value === mode.id}
      class:text-white={layoutMode.value === mode.id}
      class:text-text-secondary={layoutMode.value !== mode.id}
      onclick={() => setLayout(mode.id)}
    >
      {mode.label}
    </button>
  {/each}
</div>


