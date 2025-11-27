<script lang="ts">
  import type { Bookmark } from '$lib/types';
  import { openInBrowser, toggleFavorite } from '$lib/api';
  import { selectedTag } from '$lib/stores/bookmarks.svelte';
  
  interface Props {
    bookmark: Bookmark;
  }
  
  let { bookmark }: Props = $props();
  
  async function handleOpen() {
    await openInBrowser(bookmark.tweet_url);
  }
  
  function handleTag(tag: string) {
    selectedTag.set(tag);
  }
</script>

<div class="flex items-center justify-between gap-4 p-3">
  <div class="flex-1 min-w-0">
    <button class="text-left" onclick={handleOpen}>
      <p class="text-sm font-medium text-text-primary truncate">{bookmark.content}</p>
      <p class="text-xs text-text-muted mt-1">
        @{bookmark.author_handle} · {new Date(bookmark.tweeted_at).toLocaleDateString()}
      </p>
    </button>
    
    {#if bookmark.tags.length > 0}
      <div class="mt-2 flex flex-wrap gap-1">
        {#each bookmark.tags as tag}
          <button
            class="text-xs px-2 py-0.5 rounded-full bg-bg-tertiary text-text-muted hover:text-accent"
            onclick={() => handleTag(tag)}
          >
            #{tag}
          </button>
        {/each}
      </div>
    {/if}
  </div>
  
  <button
    class="text-sm text-accent border border-accent px-3 py-1 rounded-full hover:bg-accent hover:text-white transition-colors"
    onclick={() => toggleFavorite(bookmark.id)}
  >
    {bookmark.is_favorite ? '★' : '☆'}
  </button>
</div>


