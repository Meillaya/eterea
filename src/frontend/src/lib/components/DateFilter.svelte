<script lang="ts">
  import { dateRange, viewMode } from '$lib/stores/bookmarks.svelte';

  let showPicker = $state(false);
  let fromDate = $state('');
  let toDate = $state('');

  const presets = [
    { label: 'Today', days: 0 },
    { label: '7 days', days: 7 },
    { label: '30 days', days: 30 },
    { label: '90 days', days: 90 },
    { label: 'Year', days: 365 }
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
    viewMode.set('all');
    showPicker = false;
  }

  function applyCustomRange() {
    if (fromDate && toDate) {
      dateRange.set(new Date(fromDate).toISOString(), new Date(toDate).toISOString());
      viewMode.set('all');
      showPicker = false;
    }
  }

  function clearFilter(e: MouseEvent) {
    e.stopPropagation();
    dateRange.clear();
    if (viewMode.value === 'recent') viewMode.set('all');
    fromDate = '';
    toDate = '';
    showPicker = false;
  }

  function formatDateRange() {
    if (!dateRange.value.from || !dateRange.value.to) return 'All time';
    const from = new Date(dateRange.value.from);
    const to = new Date(dateRange.value.to);
    const options: Intl.DateTimeFormatOptions = { month: 'short', day: 'numeric' };
    return `${from.toLocaleDateString('en-US', options)} – ${to.toLocaleDateString('en-US', options)}`;
  }
</script>

<div class="relative flex items-center gap-1">
  <button
    onclick={() => showPicker = !showPicker}
    class="flex items-center gap-2 rounded-full border border-border bg-bg-secondary/75 px-3 py-2 text-sm text-text-secondary shadow-[var(--shadow-soft)] transition-colors hover:text-text-primary"
    title="Filter by date"
  >
    <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
    </svg>
    <span>{formatDateRange()}</span>
  </button>

  {#if dateRange.value.from}
    <button
      onclick={clearFilter}
      class="rounded-full p-1.5 text-text-muted transition-colors hover:bg-bg-secondary/75 hover:text-text-primary"
      title="Clear date filter"
      aria-label="Clear date filter"
    >
      <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
      </svg>
    </button>
  {/if}

  {#if showPicker}
    <div class="panel absolute left-0 top-full z-20 mt-2 min-w-72 rounded-[1.4rem] p-4">
      <div class="mb-4">
        <p class="eyebrow mb-3">Quick range</p>
        <div class="flex flex-wrap gap-2">
          {#each presets as preset}
            <button onclick={() => applyPreset(preset.days)} class="rounded-full border border-border bg-bg-tertiary px-3 py-1.5 text-sm text-text-secondary transition-colors hover:text-text-primary">
              {preset.label}
            </button>
          {/each}
        </div>
      </div>

      <div class="border-t border-border-subtle pt-4">
        <p class="eyebrow mb-3">Custom range</p>
        <div class="grid gap-2">
          <input type="date" bind:value={fromDate} class="rounded-xl border border-border bg-bg-tertiary px-3 py-2 text-sm text-text-primary focus:outline-none focus:border-accent" aria-label="From date" />
          <input type="date" bind:value={toDate} class="rounded-xl border border-border bg-bg-tertiary px-3 py-2 text-sm text-text-primary focus:outline-none focus:border-accent" aria-label="To date" />
        </div>
        <button
          onclick={applyCustomRange}
          disabled={!fromDate || !toDate}
          class="mt-3 w-full rounded-xl bg-accent px-4 py-2 text-sm font-medium text-white transition-opacity hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-50"
        >
          Apply range
        </button>
      </div>
    </div>
  {/if}
</div>

{#if showPicker}
  <button class="fixed inset-0 z-10 cursor-default border-none bg-transparent" onclick={() => showPicker = false} aria-label="Close date picker"></button>
{/if}
