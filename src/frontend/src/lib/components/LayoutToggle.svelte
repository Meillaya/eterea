<script lang="ts">
  import { layoutMode, type LayoutMode } from '$lib/stores/bookmarks.svelte';

  const modes: { id: LayoutMode; label: string }[] = [
    { id: 'default', label: 'Focus' },
    { id: 'cards', label: 'Grid' },
    { id: 'compact', label: 'List' }
  ];
</script>

<div class="inline-flex rounded-full border border-border bg-bg-secondary/75 p-1 text-xs shadow-[var(--shadow-soft)]">
  {#each modes as mode}
    <button
      class="rounded-full px-3 py-1.5 transition-colors"
      class:bg-bg-elevated={layoutMode.value === mode.id}
      class:text-text-primary={layoutMode.value === mode.id}
      class:text-text-secondary={layoutMode.value !== mode.id}
      onclick={() => layoutMode.set(mode.id)}
    >
      {mode.label}
    </button>
  {/each}
</div>
