<script lang="ts">
  import type { Bookmark } from '$lib/types';
  import { deleteBookmark, openInBrowser, toggleFavorite } from '$lib/api';
  import { dateRange, searchQuery, selectedTag, viewMode } from '$lib/stores/bookmarks.svelte';
  import LinkPreview from './LinkPreview.svelte';

  interface Props {
    bookmark: Bookmark;
    variant: 'focus' | 'grid' | 'list';
  }

  let { bookmark, variant }: Props = $props();
  let isTogglingFavorite = $state(false);
  let avatarErrored = $state(false);
  let mediaUnlocked = $state(false);
  const urlRegex = /(https?:\/\/[^\s]+)/g;

  const isList = $derived(variant === 'list');
  const previewTargets = $derived.by(() =>
    Array.from(
      new Set(
        (bookmark.content.match(urlRegex) ?? [])
          .map((url) => url.replace(/[,.!?]+$/, ''))
          .filter((url) => url !== bookmark.tweet_url),
      ),
    ).slice(0, variant === 'list' ? 1 : 2),
  );
  const mediaItems = $derived.by(() => bookmark.media.slice(0, variant === 'list' ? 2 : 4));
  const articleClass = $derived.by(() =>
    `surface-panel overflow-hidden ${variant === 'list' ? 'rounded-[1.6rem] p-4' : 'rounded-[2rem] p-5'}`,
  );
  const avatarClass = $derived.by(() =>
    `rounded-[1.2rem] border border-border-subtle bg-bg-tertiary object-cover ${variant === 'list' ? 'h-10 w-10' : 'h-12 w-12'}`,
  );
  const contentClass = $derived.by(() =>
    `whitespace-pre-wrap break-words text-text-primary ${variant === 'list' ? 'text-sm leading-6 compact-copy' : 'text-[1.08rem] leading-8'}`,
  );
  const noteClass = $derived.by(() =>
    `mt-2 leading-7 text-text-secondary ${variant === 'list' ? 'compact-note' : ''}`,
  );
  const mediaGridClass = $derived.by(() => `mt-4 grid gap-3 ${mediaItems.length > 1 ? 'grid-cols-2' : ''}`);

  function fallbackAvatar(name: string, handle: string) {
    const seed = `${name}:${handle}`;
    const initials = (name || handle)
      .split(/\s+/)
      .map((part) => part[0] ?? '')
      .join('')
      .slice(0, 2)
      .toUpperCase();
    const hash = Array.from(seed).reduce((value, char) => value + char.charCodeAt(0), 0);
    const palette = [
      ['#1d2430', '#f2bb76'],
      ['#232d3a', '#ff8752'],
      ['#171c26', '#f3f5f8'],
    ] as const;
    const [background, foreground] = palette[hash % palette.length];
    const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="96" height="96" viewBox="0 0 96 96"><rect width="96" height="96" rx="24" fill="${background}"/><text x="50%" y="54%" dominant-baseline="middle" text-anchor="middle" font-family="system-ui, sans-serif" font-size="32" font-weight="700" fill="${foreground}">${initials || 'E'}</text></svg>`;
    return `data:image/svg+xml;utf8,${encodeURIComponent(svg)}`;
  }

  function formatRelativeDate(value: string) {
    const date = new Date(value);
    const hours = Math.floor((Date.now() - date.getTime()) / (1000 * 60 * 60));
    const days = Math.floor(hours / 24);

    if (hours < 1) return 'just now';
    if (hours < 24) return `${hours}h`;
    if (days < 7) return `${days}d`;
    if (days < 30) return `${Math.floor(days / 7)}w`;
    if (days < 365) return `${Math.floor(days / 30)}mo`;

    return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
  }

  function formatSavedDate(value: string) {
    return new Date(value).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
    });
  }

  async function handleToggleFavorite() {
    isTogglingFavorite = true;
    try {
      await toggleFavorite(bookmark.id);
    } finally {
      isTogglingFavorite = false;
    }
  }

  async function handleCopy() {
    try {
      await navigator.clipboard.writeText(bookmark.tweet_url);
    } catch (error) {
      console.error('Failed to copy bookmark link', error);
    }
  }

  async function handleDelete() {
    if (!confirm('Delete this bookmark?')) return;
    await deleteBookmark(bookmark.id);
  }

  function activateTag(tag: string) {
    searchQuery.set('');
    dateRange.clear();
    selectedTag.set(tag);
    viewMode.set('all');
  }

  function handleAvatarError(event: Event) {
    if (avatarErrored) return;
    avatarErrored = true;
    (event.currentTarget as HTMLImageElement).src = fallbackAvatar(bookmark.author_name, bookmark.author_handle);
  }
</script>

<article class={articleClass}>
  <div class="flex items-start justify-between gap-4">
    <div class="flex min-w-0 items-start gap-3">
      <img
        src={fallbackAvatar(bookmark.author_name, bookmark.author_handle)}
        alt={bookmark.author_name}
        class={avatarClass}
        loading="lazy"
        onerror={handleAvatarError}
      />

      <div class="min-w-0">
        <div class="flex flex-wrap items-center gap-x-2 gap-y-1 text-sm">
          <span class="font-medium text-text-primary">{bookmark.author_name}</span>
          <span class="text-text-muted">@{bookmark.author_handle}</span>
        </div>
        <div class="mt-1 flex flex-wrap items-center gap-2 text-xs text-text-muted">
          <time datetime={bookmark.tweeted_at}>{formatRelativeDate(bookmark.tweeted_at)}</time>
          <span>·</span>
          <span class="font-mono uppercase tracking-[0.14em]">saved {formatSavedDate(bookmark.imported_at)}</span>
        </div>
      </div>
    </div>

    <div class="flex shrink-0 flex-wrap items-center justify-end gap-2">
      <button class="icon-button h-10 w-10" onclick={handleToggleFavorite} disabled={isTogglingFavorite} title={bookmark.is_favorite ? 'Remove from favorites' : 'Add to favorites'}>
        <svg class="h-4 w-4" fill={bookmark.is_favorite ? 'currentColor' : 'none'} viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z" />
        </svg>
      </button>
      <button class="ghost-button px-4 py-2.5 text-sm" onclick={() => openInBrowser(bookmark.tweet_url)}>Open</button>
      {#if !isList}
        <button class="ghost-button px-4 py-2.5 text-sm" onclick={handleCopy}>Copy</button>
      {/if}
      <button class="ghost-button px-4 py-2.5 text-sm text-red-300 hover:text-red-200" onclick={handleDelete}>Delete</button>
    </div>
  </div>

  <button class="mt-4 block w-full text-left" onclick={() => openInBrowser(bookmark.tweet_url)}>
    <p class={contentClass}>{bookmark.content}</p>
  </button>

  {#if bookmark.note_text}
    <div class="mt-4 rounded-[1.25rem] border border-border-subtle bg-bg-primary/32 p-4">
      <p class="section-label">Note</p>
      <p class={noteClass}>{bookmark.note_text}</p>
    </div>
  {/if}

  {#if mediaItems.length > 0}
    {#if mediaUnlocked}
      <div class={mediaGridClass}>
        {#each mediaItems as media}
          <div class="overflow-hidden rounded-[1.35rem] border border-border-subtle bg-bg-tertiary">
            {#if media.media_type === 'Video' || media.media_type === 'Gif'}
              <video src={media.url} class="aspect-[16/10] w-full object-cover" controls muted playsinline preload="metadata">
                <track kind="captions" />
              </video>
            {:else}
              <img src={media.url} alt="Bookmark media" class="aspect-[16/10] w-full object-cover" loading="lazy" />
            {/if}
          </div>
        {/each}
      </div>
    {:else}
      <button class="mt-4 flex w-full items-center justify-between rounded-[1.35rem] border border-border-subtle bg-bg-primary/24 px-4 py-4 text-left transition-colors hover:border-border-accent hover:text-accent" onclick={() => (mediaUnlocked = true)}>
        <span>
          <span class="block text-sm font-medium text-text-primary">Load media</span>
          <span class="mt-1 block text-xs text-text-muted">External media stays off until you ask for it.</span>
        </span>
        <span class="pill">{mediaItems.length} item{mediaItems.length === 1 ? '' : 's'}</span>
      </button>
    {/if}
  {/if}

  {#if previewTargets.length > 0}
    <div class="mt-4 space-y-3 border-t border-border-subtle pt-4">
      {#each previewTargets as url}
        <LinkPreview {url} />
      {/each}
    </div>
  {/if}

  <div class="mt-4 flex flex-wrap items-center justify-between gap-3 border-t border-border-subtle pt-4">
    <div class="flex flex-wrap gap-2">
      {#each bookmark.tags as tag}
        <button class="pill transition-colors hover:border-border-accent hover:text-accent" onclick={() => activateTag(tag)}>
          #{tag}
        </button>
      {/each}
    </div>

    <button class="text-sm text-text-muted transition-colors hover:text-accent" onclick={() => openInBrowser(bookmark.tweet_url)}>
      Open on X →
    </button>
  </div>
</article>
