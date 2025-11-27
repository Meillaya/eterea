<script lang="ts">
  interface Props {
    onimport: () => void;
  }
  
  let { onimport }: Props = $props();
</script>

<header class="flex items-center justify-between px-6 lg:px-10 py-4 border-b border-border-subtle">
  <div class="flex items-center gap-4">
    <h1 class="font-display text-2xl italic text-text-primary">
      Eterea
    </h1>
    <span class="text-xs text-text-muted font-mono bg-bg-tertiary px-2 py-1 rounded">v0.1</span>
  </div>
  
  <div class="flex items-center gap-3">
    <button
      onclick={onimport}
      class="flex items-center gap-2 px-4 py-2 bg-accent text-white text-sm font-medium rounded-lg
             hover:bg-accent/90 transition-all duration-200 hover:shadow-lg hover:shadow-accent/20"
    >
      <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
      </svg>
      Import
    </button>
    
    <a
      href="/stats"
      class="px-4 py-2 rounded-lg text-sm text-text-secondary hover:text-text-primary hover:bg-bg-tertiary transition-colors"
    >
      Stats
    </a>
    
    <a
      href="/settings"
      class="p-2 rounded-lg text-text-secondary hover:text-text-primary hover:bg-bg-tertiary transition-colors"
      title="Settings"
    >
      <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
      </svg>
    </a>
  </div>
</header>

