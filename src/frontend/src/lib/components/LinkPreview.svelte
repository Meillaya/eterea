<script lang="ts">
  import { onMount } from 'svelte';
  import { fetchLinkPreview, openInBrowser } from '$lib/api';
  import type { LinkPreview as LinkPreviewData } from '$lib/types';

  interface Props {
    url: string;
  }

  let { url }: Props = $props();
  let container: HTMLDivElement | undefined;
  let preview = $state<LinkPreviewData | null>(null);
  let failed = $state(false);
  let started = false;

  async function loadPreview() {
    if (started) return;
    started = true;

    try {
      preview = await fetchLinkPreview(url);
      failed = preview === null;
    } catch {
      failed = true;
    }
  }

  onMount(() => {
    if (typeof IntersectionObserver === 'undefined' || !container) {
      void loadPreview();
      return;
    }

    const observer = new IntersectionObserver(
      (entries) => {
        if (!entries.some((entry) => entry.isIntersecting)) return;
        observer.disconnect();
        void loadPreview();
      },
      { rootMargin: '280px 0px' },
    );

    observer.observe(container);
    return () => observer.disconnect();
  });
</script>

<div bind:this={container}>
  {#if preview}
    <button class="soft-panel w-full overflow-hidden rounded-[1.35rem] text-left transition-colors hover:border-border-strong" onclick={() => openInBrowser(preview?.final_url || url)}>
      <div class="space-y-2 p-4">
        {#if preview.site_name}
          <p class="section-label">{preview.site_name}</p>
        {/if}
        <p class="text-sm font-medium text-text-primary">{preview.title ?? preview.final_url}</p>
        {#if preview.description}
          <p class="text-sm leading-6 text-text-secondary compact-note">{preview.description}</p>
        {/if}
      </div>
    </button>
  {:else}
    <button class="pill w-full justify-between transition-colors hover:border-border-accent hover:text-accent" onclick={() => openInBrowser(url)}>
      <span class="truncate">{failed ? url : 'Open linked article'}</span>
      <span aria-hidden="true">↗</span>
    </button>
  {/if}
</div>
