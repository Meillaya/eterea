<script lang="ts">
  import { goto } from '$app/navigation';
  import { loadBookmarks } from '$lib/api';
  import { DEFAULT_FEED_LIMIT, feedMeta, layoutMode, type LayoutMode } from '$lib/stores/bookmarks.svelte';

  let limit = $state(feedMeta.value.limit ?? DEFAULT_FEED_LIMIT);

  const layouts: { id: LayoutMode; title: string; blurb: string }[] = [
    { id: 'focus', title: 'Focus', blurb: 'The calmest, most spacious reading mode.' },
    { id: 'grid', title: 'Grid', blurb: 'Balanced density when you want more on screen.' },
    { id: 'list', title: 'List', blurb: 'A faster skim for larger archives.' },
  ];

  function optionClass(id: LayoutMode) {
    return `rounded-[1.5rem] border p-4 transition-colors ${layoutMode.value === id ? 'border-border-accent bg-[color:var(--accent-soft)]' : 'border-border bg-bg-secondary/60'}`;
  }

  async function saveLimit() {
    feedMeta.reset(limit);
    await loadBookmarks({ limit, reset: true });
  }
</script>

<svelte:head>
  <title>Eterea — Settings</title>
</svelte:head>

<div class="min-h-screen bg-bg-primary text-text-primary">
  <div class="mx-auto max-w-6xl px-6 py-10 space-y-6">
    <button class="ghost-button text-sm" onclick={() => goto('/')}>← Back to library</button>

    <section class="surface-panel rounded-[2.25rem] p-7">
      <p class="section-label">Settings</p>
      <h1 class="mt-2 font-display text-5xl italic text-text-primary">Tune the room, not the noise.</h1>
      <p class="mt-3 max-w-3xl text-sm leading-7 text-text-secondary">
        The new frontend stays intentionally small. These controls keep reading quick without dragging the app back toward dashboard clutter.
      </p>
    </section>

    <section class="grid gap-5 xl:grid-cols-[1.2fr,0.8fr]">
      <section class="soft-panel rounded-[2rem] p-6">
        <p class="section-label">Layout mode</p>
        <div class="mt-4 grid gap-3 md:grid-cols-3">
          {#each layouts as option}
            <label class={optionClass(option.id)}>
              <input class="sr-only" type="radio" name="layout" value={option.id} checked={layoutMode.value === option.id} onchange={() => layoutMode.set(option.id)} />
              <p class="font-medium text-text-primary">{option.title}</p>
              <p class="mt-2 text-sm leading-6 text-text-secondary">{option.blurb}</p>
            </label>
          {/each}
        </div>
      </section>

      <section class="soft-panel rounded-[2rem] p-6">
        <p class="section-label">Feed density</p>
        <h2 class="mt-2 text-xl font-medium text-text-primary">How much loads at once</h2>
        <p class="mt-2 text-sm leading-7 text-text-secondary">
          Keep the default light, or raise the page size when you want fewer pagination stops.
        </p>

        <div class="mt-5 space-y-4">
          <input type="number" min="10" max="200" step="10" bind:value={limit} class="w-full rounded-[1rem] border border-border bg-bg-secondary/70 px-4 py-3 text-text-primary focus:border-accent focus:outline-none" />
          <button class="accent-button w-full" onclick={saveLimit}>Save feed density</button>
        </div>
      </section>
    </section>

    <section class="grid gap-5 lg:grid-cols-[0.9fr,1.1fr]">
      <section class="soft-panel rounded-[2rem] p-6">
        <p class="section-label">Keyboard flow</p>
        <div class="mt-4 space-y-3 text-sm text-text-secondary">
          <div class="pill w-full justify-between"><span>Focus search</span><kbd>/</kbd></div>
          <div class="pill w-full justify-between"><span>Clear search</span><kbd>Esc</kbd></div>
          <p class="pt-2 leading-7">The remake keeps search keyboard-first so you can open the app and move immediately.</p>
        </div>
      </section>

      <section class="soft-panel rounded-[2rem] p-6">
        <p class="section-label">Palette guardrail</p>
        <h2 class="mt-2 text-xl font-medium text-text-primary">Same colors, cleaner structure</h2>
        <p class="mt-2 text-sm leading-7 text-text-secondary">
          The redesign intentionally kept the slate background and ember/gold accents while rebuilding the shell from scratch.
        </p>
        <div class="mt-5 flex flex-wrap gap-3">
          <span class="h-12 w-12 rounded-2xl border border-border" style="background:#07090f"></span>
          <span class="h-12 w-12 rounded-2xl border border-border" style="background:#0f131b"></span>
          <span class="h-12 w-12 rounded-2xl border border-border" style="background:#171c26"></span>
          <span class="h-12 w-12 rounded-2xl border border-border" style="background:#ff8752"></span>
          <span class="h-12 w-12 rounded-2xl border border-border" style="background:#f2bb76"></span>
        </div>
      </section>
    </section>
  </div>
</div>
