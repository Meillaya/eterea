<script lang="ts">
  import { deleteBookmark, openInBrowser, toggleFavorite } from '$lib/api';
  import { selectedTag } from '$lib/stores/bookmarks.svelte';
  import { layoutMode } from '$lib/stores/bookmarks.svelte';
  import type { Bookmark } from '$lib/types';

  interface Props {
    items: Bookmark[];
  }

  let { items }: Props = $props();

  function formatDate(value: string): string {
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) {
      return value;
    }

    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
    });
  }

  async function handleDelete(id: string) {
    if (!confirm('Delete this bookmark?')) return;
    await deleteBookmark(id);
  }

  function containerClass() {
    if (layoutMode.value === 'grid') {
      return 'grid gap-4 lg:grid-cols-2 2xl:grid-cols-3';
    }

    return 'space-y-4';
  }

  function articleClass() {
    if (layoutMode.value === 'list') {
      return 'surface-panel rounded-[1.4rem] p-4';
    }

    return 'surface-panel rounded-[1.8rem] p-5';
  }
</script>

<div class={containerClass()}>
  {#each items as bookmark (bookmark.id)}
    <article class={articleClass()}>
      <div class="flex flex-wrap items-start justify-between gap-3">
        <div class="min-w-0 flex-1">
          <div class="flex flex-wrap items-center gap-x-2 gap-y-1 text-sm">
            <span class="font-medium text-text-primary">{bookmark.author_name || bookmark.author_handle || 'Unknown author'}</span>
            <span class="text-text-muted">@{bookmark.author_handle || 'unknown'}</span>
          </div>
          <div class="mt-1 flex flex-wrap items-center gap-2 text-xs text-text-muted">
            <span>{formatDate(bookmark.tweeted_at)}</span>
            <span>·</span>
            <span>saved {formatDate(bookmark.imported_at)}</span>
            {#if bookmark.media?.length}
              <span>·</span>
              <span>{bookmark.media.length} media</span>
            {/if}
          </div>
        </div>

        <div class="flex flex-wrap gap-2">
          <button class="ghost-button px-3 py-2 text-sm" onclick={() => toggleFavorite(bookmark.id)}>
            {bookmark.is_favorite ? '★ Favorite' : '☆ Favorite'}
          </button>
          <button class="ghost-button px-3 py-2 text-sm" onclick={() => openInBrowser(bookmark.tweet_url)}>
            Open
          </button>
          <button class="ghost-button px-3 py-2 text-sm text-red-300 hover:text-red-200" onclick={() => handleDelete(bookmark.id)}>
            Delete
          </button>
        </div>
      </div>

      <div class="mt-4 space-y-3">
        <p class="whitespace-pre-wrap break-words text-text-primary" class:text-sm={layoutMode.value === 'list'} class:leading-6={layoutMode.value === 'list'} class:text-[1.02rem]={layoutMode.value !== 'list'} class:leading-8={layoutMode.value !== 'list'}>
          {bookmark.content || '(no text content)'}
        </p>

        {#if bookmark.note_text}
          <div class="rounded-[1rem] border border-border-subtle bg-bg-primary/25 px-4 py-3">
            <p class="section-label">Note</p>
            <p class="mt-2 text-sm leading-6 text-text-secondary">{bookmark.note_text}</p>
          </div>
        {/if}

        {#if bookmark.tags?.length}
          <div class="flex flex-wrap gap-2">
            {#each bookmark.tags as tag}
              <button class="pill transition-colors hover:border-border-accent hover:text-accent" onclick={() => selectedTag.set(tag)}>
                #{tag}
              </button>
            {/each}
          </div>
        {/if}
      </div>
    </article>
  {/each}
</div>
