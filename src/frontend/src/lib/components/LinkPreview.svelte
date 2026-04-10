<script lang="ts">
  import { onMount } from 'svelte';
  import { fetchLinkPreview, openInBrowser } from '$lib/api';
  import type { LinkPreview } from '$lib/types';

  interface Props {
    url: string;
  }

  let { url }: Props = $props();
  let container: HTMLDivElement | undefined;
  let preview = $state<LinkPreview | null>(null);
  let failed = $state(false);
  let hasStarted = false;

  async function loadPreview() {
    if (hasStarted) return;
    hasStarted = true;

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
      { rootMargin: '320px 0px' }
    );

    observer.observe(container);

    return () => observer.disconnect();
  });
</script>

<div bind:this={container}>
  {#if preview}
    <button
      class="overflow-hidden rounded-[1.3rem] border border-border bg-bg-secondary/60 text-left transition-colors hover:border-border-strong"
      onclick={() => openInBrowser(preview?.final_url || url)}
    >
      <div class="grid gap-0 md:grid-cols-[180px,1fr]">
        {#if preview.image_url}
          <img
            src={preview.image_url}
            alt={preview.title ?? preview.final_url}
            class="h-full min-h-32 w-full object-cover"
            loading="lazy"
          />
        {/if}
        <div class="space-y-2 p-4">
          {#if preview.site_name}
            <p class="eyebrow">{preview.site_name}</p>
          {/if}
          <p class="text-sm font-medium text-text-primary">
            {preview.title ?? preview.final_url}
          </p>
          {#if preview.description}
            <p class="line-clamp-3 text-sm text-text-secondary">
              {preview.description}
            </p>
          {/if}
        </div>
      </div>
    </button>
  {:else}
    <a
      href={url}
      class="flex items-center gap-2 rounded-[1.1rem] border border-border-subtle bg-bg-secondary/45 px-3 py-2 text-sm text-text-secondary transition-colors hover:border-border hover:text-accent"
      target="_blank"
      rel="noreferrer"
    >
      <span class="truncate">{failed ? url : 'Open linked article'}</span>
      <span aria-hidden="true">↗</span>
    </a>
  {/if}
</div>
