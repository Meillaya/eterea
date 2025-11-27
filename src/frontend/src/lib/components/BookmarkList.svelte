<script lang="ts">
  import type { Bookmark } from '$lib/types';
  import BookmarkCard from './BookmarkCard.svelte';
  import BookmarkRow from './BookmarkRow.svelte';
  import { layoutMode } from '$lib/stores/bookmarks.svelte';
  
  interface Props {
    items: Bookmark[];
  }
  
  let { items }: Props = $props();
</script>

{#if layoutMode.value === 'cards'}
  <div class="grid gap-4 md:grid-cols-2">
    {#each items as bookmark (bookmark.id)}
      <BookmarkCard {bookmark} />
    {/each}
  </div>
{:else if layoutMode.value === 'compact'}
  <div class="divide-y divide-border-subtle rounded-xl border border-border-subtle bg-bg-secondary">
    {#each items as bookmark (bookmark.id)}
      <BookmarkRow {bookmark} />
    {/each}
  </div>
{:else}
  <div class="space-y-4">
    {#each items as bookmark, index (bookmark.id)}
      <div 
        class="animate-slide-up"
        style="animation-delay: {Math.min(index * 30, 300)}ms"
      >
        <BookmarkCard {bookmark} />
      </div>
    {/each}
  </div>
{/if}

