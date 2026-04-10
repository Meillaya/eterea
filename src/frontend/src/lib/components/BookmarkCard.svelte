<script lang="ts">
  import type { Bookmark } from '$lib/types';
  import { selectedTag } from '$lib/stores/bookmarks.svelte';
  import { deleteBookmark, openInBrowser, toggleFavorite } from '$lib/api';
  import LinkPreview from './LinkPreview.svelte';

  interface Props {
    bookmark: Bookmark;
  }

  let { bookmark }: Props = $props();
  let showMenu = $state(false);
  let isTogglingFavorite = $state(false);
  let avatarErrored = $state(false);
  const urlRegex = /(https?:\/\/[^\s]+)/g;
  const fallbackAvatar = (handle: string) => `https://unavatar.io/twitter/${handle}`;
  const proxiedImage = (url: string) => {
    try {
      const parsed = new URL(url);
      return `https://images.weserv.nl/?url=${encodeURIComponent(parsed.host + parsed.pathname + parsed.search)}`;
    } catch {
      return url;
    }
  };
  const linkTargets = $derived.by(() =>
    Array.from(
      new Set(
        (bookmark.content.match(urlRegex) ?? [])
          .map((url) => url.replace(/[,.!?]+$/, ''))
          .filter((url) => url !== bookmark.tweet_url)
      )
    )
  );

  function formatDate(dateString: string): string {
    const date = new Date(dateString);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const days = Math.floor(diff / (1000 * 60 * 60 * 24));
    if (days === 0) return 'Today';
    if (days === 1) return 'Yesterday';
    if (days < 7) return `${days}d ago`;
    if (days < 30) return `${Math.floor(days / 7)}w ago`;
    if (days < 365) return `${Math.floor(days / 30)}mo ago`;
    return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
  }

  async function handleOpen() {
    await openInBrowser(bookmark.tweet_url);
  }

  async function handleDelete() {
    if (confirm('Delete this bookmark?')) {
      await deleteBookmark(bookmark.id);
    }
    showMenu = false;
  }

  async function handleToggleFavorite() {
    isTogglingFavorite = true;
    try {
      await toggleFavorite(bookmark.id);
    } finally {
      isTogglingFavorite = false;
    }
  }

  async function copyLink() {
    try {
      await navigator.clipboard.writeText(bookmark.tweet_url);
    } catch (error) {
      console.error('Failed to copy link', error);
    } finally {
      showMenu = false;
    }
  }

  function handleWindowClick(event: MouseEvent) {
    if (!showMenu) return;
    const target = event.target as HTMLElement;
    if (!target.closest(`[data-menu="${bookmark.id}"]`)) {
      showMenu = false;
    }
  }

  function handleAvatarError(event: Event) {
    if (avatarErrored) return;
    avatarErrored = true;
    (event.currentTarget as HTMLImageElement).src = fallbackAvatar(bookmark.author_handle);
  }

  function handleMediaError(event: Event, url: string) {
    (event.currentTarget as HTMLImageElement).src = proxiedImage(url);
  }
</script>

<svelte:window on:click={handleWindowClick} />

<article class="panel animate-slide-up rounded-[1.75rem] p-6 transition-colors hover:border-border-strong">
  <div class="mb-5 flex items-start justify-between gap-4">
    <div class="flex min-w-0 items-center gap-3">
      <img
        src={avatarErrored || !bookmark.author_profile_image ? fallbackAvatar(bookmark.author_handle) : bookmark.author_profile_image}
        alt={bookmark.author_name}
        class="h-12 w-12 rounded-2xl border border-border-subtle bg-bg-tertiary object-cover"
        loading="lazy"
        onerror={handleAvatarError}
      />
      <div class="min-w-0">
        <div class="flex flex-wrap items-center gap-x-2 gap-y-1 text-sm">
          <span class="font-medium text-text-primary">{bookmark.author_name}</span>
          <span class="text-text-muted">@{bookmark.author_handle}</span>
        </div>
        <time class="text-xs text-text-muted" datetime={bookmark.tweeted_at}>{formatDate(bookmark.tweeted_at)}</time>
      </div>
    </div>

    <div class="flex items-center gap-2">
      <button
        onclick={handleToggleFavorite}
        disabled={isTogglingFavorite}
        class="flex h-10 w-10 items-center justify-center rounded-full border border-border bg-bg-secondary/75 text-text-secondary transition-colors hover:text-yellow-400"
        title={bookmark.is_favorite ? 'Remove from favorites' : 'Add to favorites'}
      >
        <svg class="h-4 w-4" fill={bookmark.is_favorite ? 'currentColor' : 'none'} viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z" />
        </svg>
      </button>
      <button
        onclick={handleOpen}
        class="flex h-10 w-10 items-center justify-center rounded-full border border-border bg-bg-secondary/75 text-text-secondary transition-colors hover:text-accent"
        title="Open in browser"
      >
        <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
        </svg>
      </button>
      <div class="relative" data-menu={bookmark.id}>
        <button
          onclick={() => showMenu = !showMenu}
          class="flex h-10 w-10 items-center justify-center rounded-full border border-border bg-bg-secondary/75 text-text-secondary transition-colors hover:text-text-primary"
          title="More options"
          aria-label="More options"
        >
          <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 5v.01M12 12v.01M12 19v.01" />
          </svg>
        </button>
        {#if showMenu}
          <div class="absolute right-0 top-full z-20 mt-2 min-w-44 rounded-2xl border border-border bg-bg-elevated p-2 shadow-2xl">
            <button onclick={handleOpen} class="w-full rounded-xl px-3 py-2 text-left text-sm text-text-primary transition-colors hover:bg-bg-hover">Open tweet</button>
            <button onclick={copyLink} class="w-full rounded-xl px-3 py-2 text-left text-sm text-text-primary transition-colors hover:bg-bg-hover">Copy link</button>
            <button onclick={handleDelete} class="w-full rounded-xl px-3 py-2 text-left text-sm text-red-400 transition-colors hover:bg-bg-hover">Delete bookmark</button>
          </div>
        {/if}
      </div>
    </div>
  </div>

  <div class="space-y-4">
    <p class="text-[1.02rem] leading-8 text-text-primary whitespace-pre-wrap break-words">{bookmark.content}</p>

    {#if bookmark.note_text}
      <div class="rounded-[1.2rem] border border-border bg-bg-secondary/70 p-4">
        <p class="text-sm italic text-text-secondary">{bookmark.note_text}</p>
      </div>
    {/if}

    {#if bookmark.media.length > 0}
      <div class="grid gap-3" class:grid-cols-2={bookmark.media.length > 1}>
        {#each bookmark.media.slice(0, 4) as media}
          <div class="overflow-hidden rounded-[1.3rem] border border-border-subtle bg-bg-tertiary">
            {#if media.media_type === 'Video' || media.media_type === 'Gif'}
              <video src={media.url} class="h-52 w-full object-cover" controls muted playsinline preload="metadata" poster={proxiedImage(media.url)}>
                <track kind="captions" />
              </video>
            {:else}
              <img src={media.url} alt="Media" class="h-52 w-full object-cover" loading="lazy" onerror={(event) => handleMediaError(event, media.url)} />
            {/if}
          </div>
        {/each}
      </div>
    {/if}

    {#if linkTargets.length > 0}
      <div class="space-y-3">
        {#each linkTargets as target}
          <LinkPreview url={target} />
        {/each}
      </div>
    {/if}

    <div class="flex flex-wrap items-center justify-between gap-3 pt-1">
      {#if bookmark.tags.length > 0}
        <div class="flex flex-wrap gap-2">
          {#each bookmark.tags as tag}
            <button onclick={() => selectedTag.set(tag)} class="rounded-full border border-border-subtle bg-bg-secondary/70 px-3 py-1.5 text-xs text-text-secondary transition-colors hover:text-accent">
              #{tag}
            </button>
          {/each}
        </div>
      {/if}
      <button onclick={handleOpen} class="text-sm text-text-muted transition-colors hover:text-accent">View on X →</button>
    </div>
  </div>
</article>
