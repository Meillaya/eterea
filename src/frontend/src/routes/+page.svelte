<script lang="ts">
  import { onMount } from 'svelte';
  import { hydrateCachedLibrarySnapshot, loadStats, refreshBookmarks } from '$lib/api';
  import LibraryWorkspace from '$lib/components/LibraryWorkspace.svelte';
  import ImportSheet from '$lib/components/ImportSheet.svelte';
  import { dateRange, hasMediaFilter, runtimeDiagnostics, searchQuery, selectedAuthor, selectedTag, viewMode } from '$lib/stores/bookmarks.svelte';

  let showImportSheet = $state(false);
  let ready = $state(false);
  let lastSignature = $state('');
  let lastFocusRefreshAt = $state(0);
  const FOCUS_REFRESH_COOLDOWN_MS = 30_000;

  function filtersSignature() {
    return JSON.stringify({
      query: searchQuery.value,
      tag: selectedTag.value,
      author: selectedAuthor.value,
      from: dateRange.value.from,
      to: dateRange.value.to,
      view: viewMode.value,
      hasMedia: hasMediaFilter.value,
    });
  }

  onMount(async () => {
    runtimeDiagnostics.clear();

    hydrateCachedLibrarySnapshot();
    lastSignature = filtersSignature();
    ready = true;
    lastFocusRefreshAt = Date.now();
    await refreshBookmarks();
    void loadStats({ suppressErrors: true });
  });

  onMount(() => {
    const handleError = (event: ErrorEvent) => {
      const message = event.error instanceof Error ? event.error.stack ?? event.error.message : event.message;
      console.error('[eterea][runtime:error]', message);
      runtimeDiagnostics.set(message, 'window.error');
    };

    const handleRejection = (event: PromiseRejectionEvent) => {
      const reason =
        event.reason instanceof Error
          ? event.reason.stack ?? event.reason.message
          : String(event.reason);
      console.error('[eterea][runtime:rejection]', reason);
      runtimeDiagnostics.set(reason, 'unhandledrejection');
    };

    window.addEventListener('error', handleError);
    window.addEventListener('unhandledrejection', handleRejection);

    return () => {
      window.removeEventListener('error', handleError);
      window.removeEventListener('unhandledrejection', handleRejection);
    };
  });

  async function refreshVisibleLibrary() {
    const now = Date.now();
    if (now - lastFocusRefreshAt < FOCUS_REFRESH_COOLDOWN_MS) return;
    lastFocusRefreshAt = now;
    await refreshBookmarks();
    void loadStats({ suppressErrors: true });
  }

  $effect(() => {
    const signature = filtersSignature();
    if (!ready || signature === lastSignature) return;
    lastSignature = signature;
    void refreshBookmarks();
  });
</script>

<svelte:window onfocus={refreshVisibleLibrary} />

<svelte:head>
  <title>Eterea — Library</title>
</svelte:head>

<LibraryWorkspace onopenimport={() => (showImportSheet = true)} />

{#if showImportSheet}
  <ImportSheet onclose={() => (showImportSheet = false)} />
{/if}
