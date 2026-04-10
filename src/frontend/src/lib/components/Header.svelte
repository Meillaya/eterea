<script lang="ts">
  interface Props {
    onimport: () => void;
  }

  let { onimport }: Props = $props();
</script>

<header class="border-b border-border-subtle/80 bg-bg-primary/70 backdrop-blur-xl">
  <div class="mx-auto flex max-w-7xl items-center justify-between gap-6 px-6 py-4 lg:px-10">
    <div class="flex min-w-0 items-center gap-4">
      <div class="flex h-11 w-11 items-center justify-center rounded-2xl border border-border bg-bg-secondary/80 text-accent shadow-[var(--shadow-glow)]">
        <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.7" d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z" />
        </svg>
      </div>
      <div class="min-w-0">
        <p class="eyebrow">Local-first archive</p>
        <div class="flex items-center gap-3">
          <h1 class="truncate font-display text-2xl italic text-text-primary">Eterea</h1>
          <span class="rounded-full border border-border-subtle bg-bg-secondary/70 px-2.5 py-1 text-[11px] font-mono text-text-muted">v0.1</span>
        </div>
      </div>
    </div>

    <div class="flex items-center gap-2 sm:gap-3">
      <a
        href="/stats"
        class="rounded-full border border-transparent px-4 py-2 text-sm text-text-secondary transition-colors hover:border-border hover:bg-bg-secondary/70 hover:text-text-primary"
      >
        Stats
      </a>
      <a
        href="/settings"
        class="rounded-full border border-transparent px-4 py-2 text-sm text-text-secondary transition-colors hover:border-border hover:bg-bg-secondary/70 hover:text-text-primary"
      >
        Settings
      </a>
      <button
        onclick={onimport}
        class="rounded-full bg-accent px-4 py-2.5 text-sm font-medium text-white shadow-[var(--shadow-glow)] transition-all hover:-translate-y-0.5 hover:opacity-95"
      >
        Import bookmarks
      </button>
    </div>
  </div>
</header>
