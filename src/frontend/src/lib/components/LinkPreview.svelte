<script lang="ts">
  import { onMount } from 'svelte';
  import { fetchLinkPreview, openInBrowser } from '$lib/api';
  import type { LinkPreview } from '$lib/types';
  
  interface Props {
    url: string;
  }
  
  let { url }: Props = $props();
  let preview: LinkPreview | null = null;
  let loading = $state(true);
  
  onMount(async () => {
    preview = await fetchLinkPreview(url);
    loading = false;
  });
  
  function handleOpen() {
    openInBrowser(preview?.final_url || url);
  }
</script>

{#if loading}
  <div class="border border-border-subtle rounded-lg p-3 text-sm text-text-muted animate-pulse">
    Loading preview…
  </div>
{:else if preview}
  <button
    class="w-full text-left border border-border rounded-xl overflow-hidden hover:border-accent transition-colors bg-bg-tertiary"
    onclick={handleOpen}
  >
    {#if preview.image_url}
      <img src={preview.image_url} alt={preview.title ?? preview.final_url} class="w-full h-40 object-cover" loading="lazy" />
    {/if}
    <div class="p-4 space-y-2">
      {#if preview.site_name}
        <p class="text-xs uppercase tracking-wider text-text-muted">{preview.site_name}</p>
      {/if}
      <p class="font-medium text-text-primary">{preview.title ?? preview.final_url}</p>
      {#if preview.description}
        <p class="text-sm text-text-secondary line-clamp-3">{preview.description}</p>
      {/if}
    </div>
  </button>
{:else}
  <a href={url} class="text-accent underline text-sm" target="_blank" rel="noreferrer">
    {url}
  </a>
{/if}


