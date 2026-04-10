<script lang="ts">
  import type { Bookmark } from '$lib/types';
  import { openInBrowser, toggleFavorite } from '$lib/api';
  import { selectedTag } from '$lib/stores/bookmarks.svelte';

  interface Props {
    bookmark: Bookmark;
  }

  let { bookmark }: Props = $props();
</script>

<div class="flex items-center justify-between gap-4 px-4 py-4 transition-colors hover:bg-bg-secondary/55">
  <div class="min-w-0 flex-1">
    <button class="w-full text-left" onclick={() => openInBrowser(bookmark.tweet_url)}>
      <p class="truncate text-sm font-medium text-text-primary">{bookmark.content}</p>
      <p class="mt-1 text-xs text-text-muted">@{bookmark.author_handle} · {new Date(bookmark.tweeted_at).toLocaleDateString()}</p>
    </button>

    {#if bookmark.tags.length > 0}
      <div class="mt-2 flex flex-wrap gap-1.5">
        {#each bookmark.tags as tag}
          <button class="rounded-full bg-bg-tertiary px-2.5 py-1 text-[11px] text-text-muted transition-colors hover:text-accent" onclick={() => selectedTag.set(tag)}>
            #{tag}
          </button>
        {/each}
      </div>
    {/if}
  </div>

  <button class="rounded-full border border-border px-3 py-1 text-sm text-text-secondary transition-colors hover:border-accent hover:text-accent" onclick={() => toggleFavorite(bookmark.id)}>
    {bookmark.is_favorite ? '★' : '☆'}
  </button>
</div>
