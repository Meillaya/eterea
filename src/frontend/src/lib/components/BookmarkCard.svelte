<script lang="ts">
  import type { Bookmark } from '$lib/types';
  import { selectedTag } from '$lib/stores/bookmarks.svelte';
  import { openInBrowser, deleteBookmark, toggleFavorite } from '$lib/api';
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
  const linkTargets = $derived(() =>
    Array.from(
      new Set(
        (bookmark.content.match(urlRegex) ?? [])
          .map(url => url.replace(/[,.!?]+$/, ''))
          .filter(url => url !== bookmark.tweet_url)
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
    if (days < 7) return `${days} days ago`;
    if (days < 30) return `${Math.floor(days / 7)} weeks ago`;
    if (days < 365) return `${Math.floor(days / 30)} months ago`;
    
    return date.toLocaleDateString('en-US', { 
      month: 'short', 
      day: 'numeric', 
      year: 'numeric' 
    });
  }
  
  function handleTagClick(tag: string) {
    selectedTag.set(tag);
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

<article 
  class="group relative p-5 bg-bg-secondary/80 rounded-2xl border border-border-subtle shadow-sm
         hover:border-border hover:bg-bg-tertiary/60 transition-all duration-200"
>
  <!-- Author Row -->
  <div class="flex items-start justify-between gap-4 mb-4">
    <div class="flex items-center gap-3 min-w-0">
      <img 
        src={avatarErrored || !bookmark.author_profile_image ? fallbackAvatar(bookmark.author_handle) : bookmark.author_profile_image}
        alt={bookmark.author_name}
        class="w-11 h-11 rounded-full object-cover bg-bg-tertiary"
        loading="lazy"
        onerror={handleAvatarError}
      />
      
      <div class="min-w-0">
        <div class="flex items-center gap-2">
          <span class="font-medium text-text-primary truncate">{bookmark.author_name}</span>
          <span class="text-text-muted truncate">@{bookmark.author_handle}</span>
        </div>
        <time class="text-xs text-text-muted" datetime={bookmark.tweeted_at}>
          {formatDate(bookmark.tweeted_at)}
        </time>
      </div>
    </div>
    
    <!-- Actions -->
    <div class="flex items-center gap-1 sm:gap-2 opacity-100 sm:opacity-0 group-hover:opacity-100 focus-within:opacity-100 transition-opacity">
      <button
        onclick={handleToggleFavorite}
        disabled={isTogglingFavorite}
        class="p-2 rounded-lg transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/60"
        class:text-yellow-500={bookmark.is_favorite}
        class:hover:text-yellow-400={bookmark.is_favorite}
        class:text-text-secondary={!bookmark.is_favorite}
        class:hover:text-yellow-500={!bookmark.is_favorite}
        class:hover:bg-bg-elevated={true}
        title={bookmark.is_favorite ? 'Remove from favorites' : 'Add to favorites'}
      >
        <svg class="w-4 h-4" fill={bookmark.is_favorite ? 'currentColor' : 'none'} viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z" />
        </svg>
      </button>
      
      <button
        onclick={handleOpen}
        class="p-2 rounded-lg text-text-secondary hover:text-accent hover:bg-bg-elevated transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/60"
        title="Open in browser"
      >
        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
        </svg>
      </button>
      
      <div class="relative" data-menu={bookmark.id}>
        <button
          onclick={() => showMenu = !showMenu}
          class="p-2 rounded-lg text-text-secondary hover:text-text-primary hover:bg-bg-elevated transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/60"
          title="More options"
          aria-label="More options"
          aria-haspopup="menu"
          aria-expanded={showMenu}
        >
          <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 5v.01M12 12v.01M12 19v.01M12 6a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2z" />
          </svg>
        </button>
        
        {#if showMenu}
          <div 
            class="absolute right-0 top-full mt-2 py-2 bg-bg-elevated border border-border rounded-xl shadow-2xl z-20 min-w-40"
            role="menu"
          >
            <button
              onclick={handleOpen}
              class="w-full px-4 py-2 text-left text-sm text-text-primary hover:bg-bg-hover transition-colors"
              role="menuitem"
            >
              Open tweet
            </button>
            <button
              onclick={copyLink}
              class="w-full px-4 py-2 text-left text-sm text-text-primary hover:bg-bg-hover transition-colors"
              role="menuitem"
            >
              Copy link
            </button>
            <button
              onclick={handleDelete}
              class="w-full px-4 py-2 text-left text-sm text-red-400 hover:bg-bg-hover transition-colors"
              role="menuitem"
            >
              Delete bookmark
            </button>
          </div>
        {/if}
      </div>
    </div>
  </div>
  
  <!-- Content -->
  <p class="text-text-primary leading-relaxed whitespace-pre-wrap break-words">
    {bookmark.content}
  </p>
  
  <!-- Note text -->
  {#if bookmark.note_text}
    <div class="mt-3 p-3 bg-bg-tertiary rounded-lg border-l-2 border-accent">
      <p class="text-sm text-text-secondary italic">{bookmark.note_text}</p>
    </div>
  {/if}
  
  <!-- Media -->
  {#if bookmark.media.length > 0}
    <div class="mt-4 grid gap-3" class:grid-cols-2={bookmark.media.length > 1}>
      {#each bookmark.media.slice(0, 4) as media}
        <div class="relative overflow-hidden rounded-xl bg-bg-tertiary border border-border-subtle">
          {#if media.media_type === 'Video' || media.media_type === 'Gif'}
            <video
              src={media.url}
              class="w-full h-48 object-cover"
              controls
              preload="metadata"
              poster={proxiedImage(media.url)}
            />
          {:else}
            <img 
              src={media.url} 
              alt="Media"
              class="w-full h-48 object-cover"
              loading="lazy"
              onerror={(event) => handleMediaError(event, media.url)}
            />
          {/if}
        </div>
      {/each}
    </div>
  {/if}

  {#if linkTargets.length > 0}
    <div class="mt-4 space-y-3">
      {#each linkTargets as target}
        <LinkPreview url={target} />
      {/each}
    </div>
  {/if}
  
  <!-- Tags -->
  {#if bookmark.tags.length > 0}
    <div class="mt-4 flex flex-wrap gap-2">
      {#each bookmark.tags as tag}
        <button
          onclick={() => handleTagClick(tag)}
          class="px-3 py-1 text-sm bg-bg-tertiary text-text-secondary rounded-full
                 hover:bg-bg-elevated hover:text-accent transition-colors"
        >
          #{tag}
        </button>
      {/each}
    </div>
  {/if}
</article>

