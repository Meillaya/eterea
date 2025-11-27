<script lang="ts">
  import { layoutMode, feedMeta } from '$lib/stores/bookmarks.svelte';
  import { loadBookmarks } from '$lib/api';
  import { goto } from '$app/navigation';
  
  let limit = $state(feedMeta.value.limit ?? 50);
  
  async function saveLimit() {
    feedMeta.reset(limit);
    await loadBookmarks({ limit, reset: true });
  }
</script>

<svelte:head>
  <title>Eterea · Settings</title>
</svelte:head>

<div class="min-h-screen bg-bg-primary text-text-primary">
  <div class="max-w-3xl mx-auto px-6 py-10 space-y-8">
    <button class="text-sm text-text-muted hover:text-accent" onclick={() => goto('/')}>
      ← Back to bookmarks
    </button>
    
    <div>
      <h1 class="text-3xl font-display italic">Settings</h1>
      <p class="text-text-secondary mt-2">Tune how Eterea looks and behaves.</p>
    </div>
    
    <section class="bg-bg-secondary border border-border rounded-2xl p-6 space-y-4">
      <h2 class="text-lg font-medium">Layout</h2>
      <p class="text-sm text-text-secondary">Choose how bookmarks are rendered.</p>
      <div class="flex gap-3 mt-3 flex-wrap">
        <label class="flex items-center gap-2 cursor-pointer">
          <input type="radio" name="layout" value="default" checked={layoutMode.value === 'default'} onchange={() => layoutMode.set('default')} />
          <span>Default</span>
        </label>
        <label class="flex items-center gap-2 cursor-pointer">
          <input type="radio" name="layout" value="cards" checked={layoutMode.value === 'cards'} onchange={() => layoutMode.set('cards')} />
          <span>Cards grid</span>
        </label>
        <label class="flex items-center gap-2 cursor-pointer">
          <input type="radio" name="layout" value="compact" checked={layoutMode.value === 'compact'} onchange={() => layoutMode.set('compact')} />
          <span>Compact rows</span>
        </label>
      </div>
    </section>
    
    <section class="bg-bg-secondary border border-border rounded-2xl p-6 space-y-4">
      <h2 class="text-lg font-medium">Feed density</h2>
      <p class="text-sm text-text-secondary">How many bookmarks to load per page.</p>
      <div class="flex items-center gap-3 mt-3">
        <input
          type="number"
          min="10"
          max="200"
          step="10"
          bind:value={limit}
          class="px-3 py-2 bg-bg-tertiary border border-border rounded-lg w-24"
        />
        <button
          class="px-4 py-2 bg-accent text-white rounded-lg hover:opacity-90 transition-colors"
          onclick={saveLimit}
        >
          Save
        </button>
      </div>
    </section>
  </div>
</div>


