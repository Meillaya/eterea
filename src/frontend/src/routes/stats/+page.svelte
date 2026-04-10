<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { stats } from '$lib/stores/bookmarks.svelte';
  import { loadStats } from '$lib/api';

  let error = $state<string | null>(null);
  let loading = $state(true);

  async function fetchStats() {
    error = null;
    loading = true;

    try {
      await loadStats({ throwOnError: true });
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    void fetchStats();
  });

  function formatDate(value: string | null) {
    if (!value) return '—';
    return new Date(value).toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' });
  }
</script>

<svelte:head>
  <title>Eterea · Stats</title>
</svelte:head>

<div class="min-h-screen bg-bg-primary text-text-primary">
  <div class="mx-auto max-w-6xl px-6 py-10 space-y-8">
    <button class="text-sm text-text-muted hover:text-accent" onclick={() => goto('/')}>← Back to bookmarks</button>

    <section class="panel rounded-[2rem] p-7">
      <p class="eyebrow">Archive analytics</p>
      <h1 class="mt-2 font-display text-5xl italic">A quick pulse on the library.</h1>
      <p class="mt-3 max-w-2xl text-sm leading-7 text-text-secondary">See what your reading archive looks like across favorites, authors, tags, and time.</p>
    </section>

    {#if error}
      <div class="panel rounded-[1.8rem] border-red-500/30 p-6 text-center">
        <p class="font-medium text-red-400">Failed to load stats</p>
        <p class="mt-2 text-sm text-text-muted">{error}</p>
        <button onclick={fetchStats} class="mt-4 rounded-full bg-accent px-4 py-2 text-white">Retry</button>
      </div>
    {:else if loading}
      <div class="panel flex items-center justify-center rounded-[2rem] py-24">
        <div class="h-10 w-10 animate-spin rounded-full border-2 border-accent border-t-transparent"></div>
      </div>
    {:else if stats.value}
      <section class="grid gap-4 lg:grid-cols-4">
        <div class="panel rounded-[1.6rem] p-5"><p class="eyebrow">Bookmarks</p><p class="mt-3 text-3xl font-mono">{stats.value.total_bookmarks.toLocaleString()}</p></div>
        <div class="panel rounded-[1.6rem] p-5"><p class="eyebrow">Favorites</p><p class="mt-3 text-3xl font-mono">{(stats.value.favorite_bookmarks ?? 0).toLocaleString()}</p></div>
        <div class="panel rounded-[1.6rem] p-5"><p class="eyebrow">Authors</p><p class="mt-3 text-3xl font-mono">{stats.value.unique_authors.toLocaleString()}</p></div>
        <div class="panel rounded-[1.6rem] p-5"><p class="eyebrow">Tags</p><p class="mt-3 text-3xl font-mono">{stats.value.unique_tags.toLocaleString()}</p></div>
      </section>

      <section class="grid gap-4 xl:grid-cols-[0.95fr,1.05fr]">
        <div class="panel rounded-[1.8rem] p-6 space-y-4">
          <div>
            <p class="eyebrow">Timeline</p>
            <h2 class="mt-2 text-xl font-medium">When your archive stretches</h2>
          </div>
          <div class="grid gap-3 sm:grid-cols-2">
            <div class="rounded-[1.4rem] border border-border-subtle bg-bg-secondary/60 p-4"><p class="text-xs uppercase tracking-wide text-text-muted">Earliest tweet</p><p class="mt-2 text-lg text-text-primary">{formatDate(stats.value.earliest_date)}</p></div>
            <div class="rounded-[1.4rem] border border-border-subtle bg-bg-secondary/60 p-4"><p class="text-xs uppercase tracking-wide text-text-muted">Most recent</p><p class="mt-2 text-lg text-text-primary">{formatDate(stats.value.latest_date)}</p></div>
          </div>
        </div>

        <div class="panel rounded-[1.8rem] p-6 space-y-4">
          <div>
            <p class="eyebrow">Top tags</p>
            <h2 class="mt-2 text-xl font-medium">Recurring themes</h2>
          </div>
          {#if stats.value.top_tags?.length}
            <div class="grid gap-3 md:grid-cols-2">
              {#each stats.value.top_tags.slice(0, 10) as [tag, count]}
                <div class="rounded-[1.3rem] border border-border-subtle bg-bg-secondary/60 px-4 py-3">
                  <div class="flex items-center justify-between gap-3">
                    <span class="font-medium text-text-primary">#{tag}</span>
                    <span class="font-mono text-sm text-text-muted">{count}</span>
                  </div>
                </div>
              {/each}
            </div>
          {:else}
            <p class="text-sm text-text-muted">No tags yet.</p>
          {/if}
        </div>
      </section>
    {:else}
      <div class="panel rounded-[1.8rem] py-24 text-center text-text-muted">No stats available yet. Import some bookmarks first.</div>
    {/if}
  </div>
</div>
