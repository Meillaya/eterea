<script lang="ts">
  import { deleteBookmark, openInBrowser, toggleFavorite } from '$lib/api';
  import LinkPreview from '$lib/components/LinkPreview.svelte';
  import { selectedTag } from '$lib/stores/bookmarks.svelte';
  import { layoutMode } from '$lib/stores/bookmarks.svelte';
  import type { Bookmark } from '$lib/types';

  interface Props {
    items: Bookmark[];
  }

  let { items }: Props = $props();
  const urlRegex = /(https?:\/\/[^\s]+)/g;

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

  function formatRelativeDate(value: string): string {
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) {
      return value;
    }

    const diff = Date.now() - date.getTime();
    const hours = Math.floor(diff / (1000 * 60 * 60));
    const days = Math.floor(hours / 24);

    if (hours < 1) return 'just now';
    if (hours < 24) return `${hours}h`;
    if (days < 7) return `${days}d`;
    if (days < 30) return `${Math.floor(days / 7)}w`;
    if (days < 365) return `${Math.floor(days / 30)}mo`;
    return formatDate(value);
  }

  function initials(bookmark: Bookmark): string {
    const source = bookmark.author_name || bookmark.author_handle || 'E';
    return source
      .split(/\s+/)
      .map((part) => part[0] ?? '')
      .join('')
      .slice(0, 2)
      .toUpperCase();
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
      return 'surface-panel rounded-[1.35rem] border-l-2 border-l-border-accent p-4';
    }

    if (layoutMode.value === 'grid') {
      return 'surface-panel rounded-[1.7rem] p-5';
    }

    return 'surface-panel rounded-[2rem] p-6';
  }

  function copyClass() {
    if (layoutMode.value === 'list') {
      return 'text-sm leading-6';
    }

    if (layoutMode.value === 'grid') {
      return 'text-[0.98rem] leading-7';
    }

    return 'text-[1.08rem] leading-8';
  }

  function metaPillClass(kind: 'accent' | 'muted' = 'muted') {
    if (kind === 'accent') {
      return 'inline-flex items-center rounded-full border border-border-accent bg-[color:var(--accent-soft)] px-2.5 py-1 text-[11px] font-medium text-accent-secondary';
    }

    return 'inline-flex items-center rounded-full border border-border-subtle bg-bg-primary/30 px-2.5 py-1 text-[11px] text-text-muted';
  }

  function extractedLinks(bookmark: Bookmark): string[] {
    return Array.from(
      new Set(
        (bookmark.content.match(urlRegex) ?? [])
          .map((url) => url.replace(/[,.!?]+$/, ''))
          .filter((url) => url !== bookmark.tweet_url),
      ),
    ).slice(0, layoutMode.value === 'list' ? 2 : 1);
  }

  function chromeClass() {
    return 'flex flex-wrap gap-2';
  }

  function avatarClass() {
    if (layoutMode.value === 'list') {
      return 'flex h-10 w-10 shrink-0 items-center justify-center rounded-[1rem] border border-border-subtle bg-linear-to-br from-bg-secondary/95 to-bg-tertiary/95 font-mono text-xs text-accent-secondary';
    }

    return 'flex h-11 w-11 shrink-0 items-center justify-center rounded-[1.1rem] border border-border-subtle bg-linear-to-br from-bg-secondary/95 to-bg-tertiary/95 font-mono text-sm text-accent-secondary';
  }

  function headerLayoutClass() {
    if (layoutMode.value === 'list') {
      return 'flex flex-wrap items-start justify-between gap-2';
    }

    return 'flex flex-wrap items-start justify-between gap-3';
  }

  function identityClass() {
    if (layoutMode.value === 'list') {
      return 'min-w-0 flex flex-1 items-start gap-2.5';
    }

    return 'min-w-0 flex flex-1 items-start gap-3';
  }

  function contentStackClass() {
    if (layoutMode.value === 'grid') {
      return 'mt-4 space-y-4';
    }

    return 'mt-4 space-y-3';
  }

  function contentPanelClass(kind: 'note' | 'comment' | 'media') {
    const base = 'rounded-[1rem] border px-4 py-3';
    if (kind === 'note') {
      return `${base} border-border-subtle bg-bg-primary/25`;
    }
    if (kind === 'comment') {
      return `${base} border-border-subtle bg-bg-secondary/35`;
    }
    return `${base} border-border-subtle bg-bg-primary/20`;
  }

  function footerClass() {
    if (layoutMode.value === 'list') {
      return 'flex flex-wrap items-center justify-between gap-2 border-t border-border-subtle pt-3 text-[11px] text-text-muted';
    }

    return 'flex flex-wrap items-center justify-between gap-2 border-t border-border-subtle pt-3 text-xs text-text-muted';
  }
</script>

<div class={containerClass()}>
  {#each items as bookmark (bookmark.id)}
    {@const links = extractedLinks(bookmark)}
    <article class={articleClass()}>
      <div class={headerLayoutClass()}>
        <div class={identityClass()}>
          <div class={avatarClass()}>
            {initials(bookmark)}
          </div>

          <div class="min-w-0 flex-1">
            <div class="flex flex-wrap items-center gap-x-2 gap-y-1 text-sm">
              <span class="font-medium text-text-primary">{bookmark.author_name || bookmark.author_handle || 'Unknown author'}</span>
              <span class="text-text-muted">@{bookmark.author_handle || 'unknown'}</span>
              {#if bookmark.is_favorite}
                <span class={metaPillClass('accent')}>favorite</span>
              {/if}
            </div>
            <div class="mt-1 flex flex-wrap items-center gap-2 text-xs text-text-muted">
              <span>{formatRelativeDate(bookmark.tweeted_at)}</span>
              <span>·</span>
              <span>{formatDate(bookmark.tweeted_at)}</span>
              <span>·</span>
              <span>saved {formatDate(bookmark.imported_at)}</span>
              {#if bookmark.media?.length}
                <span>·</span>
                <span>{bookmark.media.length} media</span>
              {/if}
            </div>
          </div>
        </div>

        <div class={chromeClass()}>
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

      <div class={contentStackClass()}>
        {#if layoutMode.value !== 'list'}
          <div class="flex items-start gap-3">
            <span class="mt-0.5 text-3xl leading-none text-accent/35">“</span>
            <p class={`whitespace-pre-wrap break-words text-text-primary ${copyClass()}`}>
              {bookmark.content || '(no text content)'}
            </p>
          </div>
        {:else}
          <p class={`whitespace-pre-wrap break-words text-text-primary ${copyClass()}`}>
            {bookmark.content || '(no text content)'}
          </p>
        {/if}

        {#if bookmark.note_text}
          <div class={contentPanelClass('note')}>
            <p class="section-label">Note</p>
            <p class="mt-2 text-sm leading-6 text-text-secondary">{bookmark.note_text}</p>
          </div>
        {/if}

        {#if bookmark.comments}
          <div class={contentPanelClass('comment')}>
            <p class="section-label">Comment</p>
            <p class="mt-2 text-sm leading-6 text-text-secondary">{bookmark.comments}</p>
          </div>
        {/if}

        {#if bookmark.media?.length}
          <div class={contentPanelClass('media')}>
            <div class="flex flex-wrap items-center justify-between gap-2">
              <div>
                <p class="section-label">Media attached</p>
                <p class="mt-2 text-sm text-text-secondary">
                  {bookmark.media.length} item{bookmark.media.length === 1 ? '' : 's'} available for this bookmark.
                </p>
              </div>
              <div class="flex flex-wrap gap-2">
                {#each bookmark.media.slice(0, layoutMode.value === 'list' ? 2 : 3) as media}
                  <span class={metaPillClass()}>{media.media_type}</span>
                {/each}
                <button class="ghost-button px-3 py-2 text-sm" onclick={() => openInBrowser(bookmark.tweet_url)}>
                  Open media on X
                </button>
              </div>
            </div>
          </div>
        {/if}

        {#if links.length > 0}
          {#if layoutMode.value === 'list'}
            <div class="flex flex-wrap gap-2">
              {#each links as url}
                <button class="pill max-w-full transition-colors hover:border-border-accent hover:text-accent" onclick={() => openInBrowser(url)}>
                  <span class="truncate">{url}</span>
                </button>
              {/each}
            </div>
          {:else}
            <div class="space-y-3">
              {#each links as url}
                <LinkPreview {url} />
              {/each}
            </div>
          {/if}
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

        <div class={footerClass()}>
          <div class="flex flex-wrap items-center gap-2">
            <span class={metaPillClass()}>{bookmark.id.slice(0, 8)}</span>
            {#if bookmark.author_profile_url}
              <button class="text-text-muted transition-colors hover:text-accent" onclick={() => openInBrowser(bookmark.author_profile_url!)}>
                Author profile
              </button>
            {/if}
          </div>
          <button class="text-text-muted transition-colors hover:text-accent" onclick={() => openInBrowser(bookmark.tweet_url)}>
            Open original →
          </button>
        </div>
      </div>
    </article>
  {/each}
</div>
