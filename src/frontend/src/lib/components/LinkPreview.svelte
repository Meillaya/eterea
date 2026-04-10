<script lang="ts">
  import { onMount } from 'svelte';
  import { fetchLinkPreview, openInBrowser } from '$lib/api';
  import type { LinkPreview } from '$lib/types';

  interface Props {
    url: string;
  }

  let { url }: Props = $props();
  let preview = $state<LinkPreview | null>(null);
  let loading = $state(true);
  let failed = $state(false);

  onMount(async () => {
    try {
      preview = await fetchLinkPreview(url);
      failed = preview === null;
    } finally {
      loading = false;
    }
  });
</script>

{#if loading}
  <div class="rounded-[1.2rem] border border-border-subtle bg-bg-secondary/55 p-4 text-sm text-text-muted animate-pulse">Loading preview…</div>
{:else if preview}
  <button class="overflow-hidden rounded-[1.3rem] border border-border bg-bg-secondary/60 text-left transition-colors hover:border-border-strong" onclick={() => openInBrowser(preview?.final_url || url)}>
    <div class="grid gap-0 md:grid-cols-[180px,1fr]">
      {#if preview.image_url}
        <img src={preview.image_url} alt={preview.title ?? preview.final_url} class="h-full min-h-32 w-full object-cover" loading="lazy" />
      {/if}
      <div class="space-y-2 p-4">
        {#if preview.site_name}
          <p class="eyebrow">{preview.site_name}</p>
        {/if}
        <p class="text-sm font-medium text-text-primary">{preview.title ?? preview.final_url}</p>
        {#if preview.description}
          <p class="line-clamp-3 text-sm text-text-secondary">{preview.description}</p>
        {/if}
      </div>
    </div>
  </button>
{:else if failed}
  <a href={url} class="text-sm text-accent underline" target="_blank" rel="noreferrer">{url}</a>
{/if}
