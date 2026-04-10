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
  <div class="mx-auto max-w-5xl px-6 py-10 space-y-8">
    <button class="text-sm text-text-muted hover:text-accent" onclick={() => goto('/')}>← Back to bookmarks</button>

    <section class="panel rounded-[2rem] p-7">
      <p class="eyebrow">Workspace settings</p>
      <h1 class="mt-2 font-display text-5xl italic">Tune the reading experience.</h1>
      <p class="mt-3 max-w-2xl text-sm leading-7 text-text-secondary">Adjust presentation density and browsing style without changing how the archive works.</p>
    </section>

    <section class="grid gap-4 xl:grid-cols-[1fr,0.85fr]">
      <section class="panel rounded-[1.8rem] p-6 space-y-5">
        <div>
          <p class="eyebrow">Layout</p>
          <h2 class="mt-2 text-xl font-medium">Choose how bookmarks read</h2>
          <p class="mt-2 text-sm text-text-secondary">Focus mode gives content more room, grid mode balances density, and list mode compresses the archive for scanning.</p>
        </div>
        <div class="grid gap-3 md:grid-cols-3">
          <label class="rounded-[1.4rem] border border-border bg-bg-secondary/60 p-4 transition-colors has-[:checked]:border-accent has-[:checked]:bg-accent/10">
            <input type="radio" name="layout" value="default" checked={layoutMode.value === 'default'} onchange={() => layoutMode.set('default')} class="sr-only" />
            <p class="font-medium text-text-primary">Focus</p>
            <p class="mt-2 text-sm text-text-secondary">Large reading cards for calm browsing.</p>
          </label>
          <label class="rounded-[1.4rem] border border-border bg-bg-secondary/60 p-4 transition-colors has-[:checked]:border-accent has-[:checked]:bg-accent/10">
            <input type="radio" name="layout" value="cards" checked={layoutMode.value === 'cards'} onchange={() => layoutMode.set('cards')} class="sr-only" />
            <p class="font-medium text-text-primary">Grid</p>
            <p class="mt-2 text-sm text-text-secondary">Balanced density for mixed browsing.</p>
          </label>
          <label class="rounded-[1.4rem] border border-border bg-bg-secondary/60 p-4 transition-colors has-[:checked]:border-accent has-[:checked]:bg-accent/10">
            <input type="radio" name="layout" value="compact" checked={layoutMode.value === 'compact'} onchange={() => layoutMode.set('compact')} class="sr-only" />
            <p class="font-medium text-text-primary">List</p>
            <p class="mt-2 text-sm text-text-secondary">Compact rows for fast scanning.</p>
          </label>
        </div>
      </section>

      <section class="panel rounded-[1.8rem] p-6 space-y-5">
        <div>
          <p class="eyebrow">Feed density</p>
          <h2 class="mt-2 text-xl font-medium">Control how much loads at once</h2>
          <p class="mt-2 text-sm text-text-secondary">Increase the page size for deeper scans, or keep it light for a calmer feed.</p>
        </div>
        <div class="space-y-4">
          <input type="number" min="10" max="200" step="10" bind:value={limit} class="w-full rounded-[1rem] border border-border bg-bg-secondary/60 px-4 py-3 text-text-primary focus:outline-none focus:border-accent" />
          <button class="rounded-full bg-accent px-5 py-2.5 text-white transition-opacity hover:opacity-90" onclick={saveLimit}>Save feed density</button>
        </div>
      </section>
    </section>
  </div>
</div>
