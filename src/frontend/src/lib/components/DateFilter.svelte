<script lang="ts">
  import { dateRange } from '$lib/stores/bookmarks.svelte';
  import { searchWithFilters } from '$lib/api';
  
  let showPicker = $state(false);
  let fromDate = $state('');
  let toDate = $state('');
  
  // Preset options
  const presets = [
    { label: 'Today', days: 0 },
    { label: 'Last 7 days', days: 7 },
    { label: 'Last 30 days', days: 30 },
    { label: 'Last 90 days', days: 90 },
    { label: 'This year', days: 365 },
  ];
  
  function applyPreset(days: number) {
    const to = new Date();
    const from = new Date();
    
    if (days === 0) {
      from.setHours(0, 0, 0, 0);
    } else {
      from.setDate(from.getDate() - days);
    }
    
    dateRange.set(from.toISOString(), to.toISOString());
    showPicker = false;
    
    searchWithFilters({
      fromDate: from.toISOString(),
      toDate: to.toISOString(),
    });
  }
  
  function applyCustomRange() {
    if (fromDate && toDate) {
      const from = new Date(fromDate).toISOString();
      const to = new Date(toDate).toISOString();
      dateRange.set(from, to);
      showPicker = false;
      
      searchWithFilters({ fromDate: from, toDate: to });
    }
  }
  
  function clearFilter(e: MouseEvent) {
    e.stopPropagation();
    dateRange.clear();
    fromDate = '';
    toDate = '';
    showPicker = false;
  }
  
  function formatDateRange(): string {
    if (!dateRange.value.from || !dateRange.value.to) return 'All time';
    
    const from = new Date(dateRange.value.from);
    const to = new Date(dateRange.value.to);
    
    const opts: Intl.DateTimeFormatOptions = { month: 'short', day: 'numeric' };
    return `${from.toLocaleDateString('en-US', opts)} - ${to.toLocaleDateString('en-US', opts)}`;
  }
  
  function getButtonClasses(): string {
    const base = 'flex items-center gap-2 px-3 py-2 text-sm bg-bg-tertiary rounded-lg hover:bg-bg-elevated transition-colors';
    return dateRange.value.from !== null 
      ? `${base} text-accent` 
      : `${base} text-text-secondary`;
  }
</script>

<div class="relative flex items-center gap-1">
  <button
    onclick={() => showPicker = !showPicker}
    class={getButtonClasses()}
    title="Filter by date"
  >
    <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
    </svg>
    <span>{formatDateRange()}</span>
  </button>
  
  {#if dateRange.value.from}
    <button
      onclick={clearFilter}
      class="p-1 rounded hover:bg-bg-hover text-text-muted hover:text-text-primary transition-colors"
      title="Clear date filter"
      aria-label="Clear date filter"
    >
      <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
      </svg>
    </button>
  {/if}
  
  {#if showPicker}
    <div class="absolute top-full left-0 mt-2 p-4 bg-bg-elevated border border-border rounded-xl shadow-xl z-20 min-w-64">
      <!-- Presets -->
      <div class="mb-4">
        <p class="text-xs text-text-muted mb-2 uppercase tracking-wider">Quick select</p>
        <div class="flex flex-wrap gap-2">
          {#each presets as preset}
            <button
              onclick={() => applyPreset(preset.days)}
              class="px-3 py-1.5 text-sm bg-bg-tertiary rounded-lg hover:bg-bg-hover transition-colors"
            >
              {preset.label}
            </button>
          {/each}
        </div>
      </div>
      
      <!-- Custom range -->
      <div class="pt-4 border-t border-border-subtle">
        <p class="text-xs text-text-muted mb-2 uppercase tracking-wider">Custom range</p>
        <div class="flex items-center gap-2">
          <input
            type="date"
            bind:value={fromDate}
            class="flex-1 px-3 py-2 bg-bg-tertiary border border-border rounded-lg text-sm
                   focus:outline-none focus:border-accent"
            aria-label="From date"
          />
          <span class="text-text-muted">to</span>
          <input
            type="date"
            bind:value={toDate}
            class="flex-1 px-3 py-2 bg-bg-tertiary border border-border rounded-lg text-sm
                   focus:outline-none focus:border-accent"
            aria-label="To date"
          />
        </div>
        <button
          onclick={applyCustomRange}
          disabled={!fromDate || !toDate}
          class="mt-3 w-full py-2 bg-accent text-white text-sm rounded-lg
                 hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          Apply
        </button>
      </div>
    </div>
  {/if}
</div>

<!-- Click outside to close -->
{#if showPicker}
  <button 
    class="fixed inset-0 z-10 cursor-default bg-transparent border-none" 
    onclick={() => showPicker = false}
    aria-label="Close date picker"
  ></button>
{/if}
