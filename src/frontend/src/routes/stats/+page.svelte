<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { stats } from '$lib/stores/bookmarks.svelte';
  import { loadStats } from '$lib/api';
  
  let error = $state<string | null>(null);
  let loading = $state(true);
  
  onMount(async () => {
    try {
      await loadStats();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      console.error('Stats load error:', e);
    } finally {
      loading = false;
    }
  });
  
  const formatDate = (value: string | null) => {
    if (!value) return '—';
    return new Date(value).toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' });
  };
</script>

<svelte:head>
  <title>Eterea · Stats</title>
</svelte:head>

<div class="min-h-screen bg-bg-primary text-text-primary">
  <div class="max-w-5xl mx-auto px-6 py-10 space-y-8">
    <button class="text-sm text-text-muted hover:text-accent" onclick={() => goto('/')}>
      ← Back to bookmarks
    </button>
    
    <div>
      <h1 class="text-4xl font-display italic">Stats</h1>
      <p class="text-text-secondary mt-2">A quick pulse on your saved knowledge.</p>
    </div>
    
    {#if error}
      <div class="bg-red-500/10 border border-red-500/30 rounded-xl p-6 text-center">
        <p class="text-red-400 font-medium">Failed to load stats</p>
        <p class="text-sm text-text-muted mt-2">{error}</p>
        <button onclick={() => { error = null; loading = true; loadStats().finally(() => loading = false); }} class="mt-4 px-4 py-2 bg-accent text-white rounded-lg">
          Retry
        </button>
      </div>
    {:else if loading}
      <div class="flex items-center justify-center py-20">
        <div class="animate-spin w-8 h-8 border-2 border-accent border-t-transparent rounded-full"></div>
      </div>
    {:else if stats.value}
      <section class="grid gap-4 md:grid-cols-2">
        <div class="bg-bg-secondary/80 border border-border rounded-2xl p-6">
          <p class="text-text-muted text-sm uppercase tracking-wide">Total bookmarks</p>
          <p class="text-3xl font-mono mt-2">{stats.value.total_bookmarks.toLocaleString()}</p>
        </div>
        <div class="bg-bg-secondary/80 border border-border rounded-2xl p-6">
          <p class="text-text-muted text-sm uppercase tracking-wide">Favorites</p>
          <p class="text-3xl font-mono mt-2">{(stats.value.favorite_bookmarks ?? 0).toLocaleString()}</p>
        </div>
        <div class="bg-bg-secondary/80 border border-border rounded-2xl p-6">
          <p class="text-text-muted text-sm uppercase tracking-wide">Unique authors</p>
          <p class="text-3xl font-mono mt-2">{stats.value.unique_authors.toLocaleString()}</p>
        </div>
        <div class="bg-bg-secondary/80 border border-border rounded-2xl p-6">
          <p class="text-text-muted text-sm uppercase tracking-wide">Unique tags</p>
          <p class="text-3xl font-mono mt-2">{stats.value.unique_tags.toLocaleString()}</p>
        </div>
      </section>
      
      <section class="bg-bg-secondary/60 border border-border rounded-2xl p-6 space-y-4">
        <h2 class="text-lg font-medium">Timeline</h2>
        <div class="grid sm:grid-cols-2 gap-4">
          <div class="bg-bg-tertiary/70 border border-border-subtle rounded-xl p-4">
            <p class="text-xs text-text-muted uppercase tracking-wide">Earliest tweet</p>
            <p class="text-lg mt-2">{formatDate(stats.value.earliest_date)}</p>
          </div>
          <div class="bg-bg-tertiary/70 border border-border-subtle rounded-xl p-4">
            <p class="text-xs text-text-muted uppercase tracking-wide">Most recent</p>
            <p class="text-lg mt-2">{formatDate(stats.value.latest_date)}</p>
          </div>
        </div>
      </section>
      
      {#if stats.value.top_tags && stats.value.top_tags.length > 0}
        <section class="bg-bg-secondary/60 border border-border rounded-2xl p-6 space-y-4">
          <h2 class="text-lg font-medium">Top tags</h2>
          <div class="grid gap-3 md:grid-cols-2">
            {#each stats.value.top_tags.slice(0, 10) as [tag, count]}
              <div class="flex items-center justify-between bg-bg-tertiary/60 border border-border-subtle rounded-xl px-4 py-3">
                <span class="text-text-primary font-medium">#{tag}</span>
                <span class="text-sm font-mono text-text-muted">{count}</span>
              </div>
            {/each}
          </div>
        </section>
      {/if}
    {:else}
      <div class="text-center py-20 text-text-muted">
        <p>No stats available yet. Import some bookmarks first!</p>
      </div>
    {/if}
  </div>
</div>


