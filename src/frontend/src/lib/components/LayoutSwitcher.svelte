<script lang="ts">
  import { layoutMode, type LayoutMode } from '$lib/stores/bookmarks.svelte';

  const modes: { id: LayoutMode; label: string; blurb: string }[] = [
    { id: 'focus', label: 'Focus', blurb: 'Roomy reading' },
    { id: 'grid', label: 'Grid', blurb: 'Balanced density' },
    { id: 'list', label: 'List', blurb: 'Fast scanning' },
  ];

  function modeClass(id: LayoutMode) {
    return `rounded-full px-3 py-1.5 transition-colors ${layoutMode.value === id ? 'bg-bg-elevated text-text-primary' : 'text-text-secondary'}`;
  }
</script>

<div class="inline-flex rounded-full border border-border bg-bg-secondary/70 p-1 text-xs shadow-[var(--shadow-soft)]">
  {#each modes as mode}
    <button class={modeClass(mode.id)} onclick={() => layoutMode.set(mode.id)} title={mode.blurb}>
      {mode.label}
    </button>
  {/each}
</div>
