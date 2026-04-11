<script lang="ts">
  import { dateRange, viewMode } from '$lib/stores/bookmarks.svelte';

  let open = $state(false);
  let fromDate = $state('');
  let toDate = $state('');

  const presets = [
    { label: 'Today', days: 0 },
    { label: '7 days', days: 7 },
    { label: '30 days', days: 30 },
    { label: '90 days', days: 90 },
    { label: 'Year', days: 365 },
  ] as const;

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
    open = false;
  }

  function applyCustomRange() {
    if (!fromDate || !toDate) return;

    dateRange.set(new Date(fromDate).toISOString(), new Date(toDate).toISOString());
    viewMode.set('all');
    open = false;
  }

  function clearRange(event: MouseEvent) {
    event.stopPropagation();
    dateRange.clear();
    if (viewMode.value === 'recent') {
      viewMode.set('all');
    }
    fromDate = '';
    toDate = '';
    open = false;
  }

  function label() {
    if (!dateRange.value.from || !dateRange.value.to) return 'Any time';

    const options: Intl.DateTimeFormatOptions = { month: 'short', day: 'numeric' };
    const from = new Date(dateRange.value.from).toLocaleDateString('en-US', options);
    const to = new Date(dateRange.value.to).toLocaleDateString('en-US', options);
    return `${from} – ${to}`;
  }
</script>

<div class="relative">
  <div class="flex items-center gap-2">
    <button class="ghost-button" onclick={() => (open = !open)} title="Filter by date">
      <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
      </svg>
      <span>{label()}</span>
    </button>

    {#if dateRange.value.from}
      <button class="icon-button h-10 w-10" onclick={clearRange} title="Clear date filter" aria-label="Clear date filter">
        <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    {/if}
  </div>

  {#if open}
    <div class="surface-panel absolute left-0 top-full z-30 mt-3 min-w-[18rem] rounded-[1.5rem] p-4">
      <p class="section-label">Quick range</p>
      <div class="mt-3 flex flex-wrap gap-2">
        {#each presets as preset}
          <button class="pill transition-colors hover:border-border-accent hover:text-accent" onclick={() => applyPreset(preset.days)}>
            {preset.label}
          </button>
        {/each}
      </div>

      <div class="mt-5 border-t border-border-subtle pt-4">
        <p class="section-label">Custom</p>
        <div class="mt-3 grid gap-2">
          <input type="date" bind:value={fromDate} class="rounded-[1rem] border border-border bg-bg-tertiary px-3 py-2 text-text-primary focus:border-accent focus:outline-none" aria-label="From date" />
          <input type="date" bind:value={toDate} class="rounded-[1rem] border border-border bg-bg-tertiary px-3 py-2 text-text-primary focus:border-accent focus:outline-none" aria-label="To date" />
        </div>
        <button class="accent-button mt-3 w-full" onclick={applyCustomRange} disabled={!fromDate || !toDate}>Apply range</button>
      </div>
    </div>
    <button class="fixed inset-0 z-20 cursor-default border-none bg-transparent" onclick={() => (open = false)} aria-label="Close date picker"></button>
  {/if}
</div>
