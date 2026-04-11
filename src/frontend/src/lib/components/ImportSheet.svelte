<script lang="ts">
  import { onMount } from 'svelte';
  import { getXSyncStatus, importBookmarksFromX, importFileContent } from '$lib/api';
  import type { XSyncStatus } from '$lib/types';

  interface Props {
    onclose: () => void;
  }

  type ImportResult =
    | { kind: 'file'; success: true; count: number }
    | { kind: 'x'; success: true; count: number; skipped: number; total: number; lastSyncedAt: string; message: string }
    | { kind: 'file' | 'x'; success: false; error: string };

  let { onclose }: Props = $props();
  let isImporting = $state(false);
  let importResult = $state<ImportResult | null>(null);
  let dragOver = $state(false);
  let xSyncStatus = $state<XSyncStatus | null>(null);
  let loadingXStatus = $state(true);
  let fileInput: HTMLInputElement | null = null;
  const isTauri = typeof window !== 'undefined' && '__TAURI__' in window;
  const importGuidance = 'CSV · JSON · X archive JS · max 10 MB';

  const canUseXImport = $derived(Boolean(isTauri && xSyncStatus?.configured));
  const xButtonLabel = $derived.by(() => {
    if (!isTauri) return 'Desktop app required';
    if (!xSyncStatus?.configured) return 'X import unavailable';
    return xSyncStatus.connected ? 'Import directly from X' : 'Sign in and import from X';
  });

  onMount(async () => {
    await refreshXStatus();
  });

  function formatDate(value: string | null) {
    if (!value) return 'Never';
    return new Date(value).toLocaleString('en-US', {
      month: 'short',
      day: 'numeric',
      hour: 'numeric',
      minute: '2-digit',
    });
  }

  async function refreshXStatus() {
    loadingXStatus = true;
    try {
      xSyncStatus = await getXSyncStatus();
    } catch (error) {
      console.error('Failed to load X sync status', error);
      xSyncStatus = null;
    } finally {
      loadingXStatus = false;
    }
  }

  function closeOnEscape(event: KeyboardEvent) {
    if (event.key === 'Escape' && !isImporting) {
      onclose();
    }
  }

  function pickFile() {
    if (!isTauri) {
      alert('Import is only available inside the desktop app.');
      return;
    }
    fileInput?.click();
  }

  async function doFileImport(file: File) {
    isImporting = true;
    importResult = null;

    try {
      const count = await importFileContent(file);
      importResult = { kind: 'file', success: true, count };
      await refreshXStatus();
    } catch (error) {
      importResult = {
        kind: 'file',
        success: false,
        error: error instanceof Error ? error.message : 'Import failed.',
      };
    } finally {
      isImporting = false;
      dragOver = false;
    }
  }

  async function handleFileChange(event: Event) {
    const input = event.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    await doFileImport(file);
    input.value = '';
  }

  async function handleDrop(event: DragEvent) {
    event.preventDefault();
    dragOver = false;
    const file = event.dataTransfer?.files?.[0];
    if (file) {
      await doFileImport(file);
    }
  }

  async function handleXImport() {
    if (!canUseXImport) return;

    isImporting = true;
    importResult = null;
    try {
      const result = await importBookmarksFromX();
      importResult = {
        kind: 'x',
        success: true,
        count: result.imported_count,
        skipped: result.skipped_count,
        total: result.total_fetched,
        lastSyncedAt: result.last_synced_at,
        message: result.status_message,
      };
      await refreshXStatus();
    } catch (error) {
      importResult = {
        kind: 'x',
        success: false,
        error: error instanceof Error ? error.message : 'X import failed.',
      };
      await refreshXStatus();
    } finally {
      isImporting = false;
    }
  }
</script>

<svelte:window onkeydown={closeOnEscape} />

<div class="fixed inset-0 z-50 flex items-end justify-center bg-black/70 px-4 py-4 backdrop-blur-md sm:items-center" role="button" tabindex="0" aria-label="Close import sheet backdrop" onclick={(event) => event.target === event.currentTarget && onclose()} onkeydown={(event) => { if (event.key === 'Enter' || event.key === ' ' || event.key === 'Escape') { event.preventDefault(); onclose(); } }}>
  <div class="surface-panel max-h-[90vh] w-full max-w-5xl overflow-hidden rounded-[2.25rem]">
    <div class="flex items-center justify-between border-b border-border-subtle px-6 py-5 sm:px-7">
      <div>
        <p class="section-label">Bring more into the room</p>
        <h2 class="mt-1 text-2xl font-medium text-text-primary">Import bookmarks</h2>
      </div>
      <button class="icon-button h-11 w-11" onclick={onclose} title="Close import sheet" aria-label="Close import sheet">
        <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </div>

    <div class="max-h-[calc(90vh-5.5rem)] overflow-y-auto p-6 sm:p-7">
      {#if importResult}
        <div class="mx-auto max-w-2xl py-10 text-center">
          <div class="mx-auto flex h-16 w-16 items-center justify-center rounded-full border border-border-subtle bg-bg-secondary/70 text-accent">
            {#if importResult.success}
              <svg class="h-8 w-8" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" /></svg>
            {:else}
              <svg class="h-8 w-8 text-red-300" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
            {/if}
          </div>

          <h3 class="mt-5 text-2xl font-medium text-text-primary">{importResult.success ? 'Import complete' : 'Import failed'}</h3>

          {#if importResult.success && importResult.kind === 'x'}
            <p class="mt-3 text-text-secondary">{importResult.message}</p>
            <p class="mt-2 text-sm text-text-muted">Imported {importResult.count}, skipped {importResult.skipped}, fetched {importResult.total}</p>
            <p class="mt-1 text-xs text-text-muted">Last synced {formatDate(importResult.lastSyncedAt)}</p>
          {:else if importResult.success}
            <p class="mt-3 text-text-secondary">Imported <span class="font-mono text-accent">{importResult.count.toLocaleString()}</span> bookmarks into your library.</p>
          {:else}
            <p class="mt-3 text-text-secondary">{importResult.error}</p>
          {/if}

          <div class="mt-6 flex justify-center gap-3">
            <button class="ghost-button" onclick={() => (importResult = null)}>Import something else</button>
            <button class="accent-button" onclick={onclose}>Done</button>
          </div>
        </div>
      {:else if isImporting}
        <div class="py-16 text-center">
          <div class="mx-auto h-12 w-12 animate-spin rounded-full border-2 border-accent border-t-transparent"></div>
          <p class="mt-5 text-text-primary">Importing bookmarks…</p>
          <p class="mt-2 text-sm text-text-muted">Finish the browser sign-in if prompted, then return to Eterea.</p>
        </div>
      {:else}
        <div class="grid gap-5 lg:grid-cols-[1.05fr,0.95fr]">
          <section class="surface-panel rounded-[2rem] border-border-accent bg-gradient-to-br from-accent/14 to-transparent p-6">
            <div class="flex items-start justify-between gap-4">
              <div>
                <p class="section-label">Recommended</p>
                <h3 class="mt-2 text-2xl font-medium text-text-primary">Import directly from X</h3>
              </div>
              <span class="pill">fastest</span>
            </div>

            <p class="mt-3 max-w-md text-sm leading-7 text-text-secondary">
              Use the direct import when you want the quickest path back into your reading archive. Re-sync later whenever you want fresh saves.
            </p>

            <div class="mt-5 rounded-[1.4rem] border border-border bg-bg-primary/30 p-4 text-sm">
              {#if loadingXStatus}
                <p class="text-text-muted">Checking X import availability…</p>
              {:else if !isTauri}
                <p class="text-amber-300">Available only inside the desktop app.</p>
              {:else if !xSyncStatus?.configured}
                <p class="text-amber-300">Set <span class="font-mono">ETEREA_X_CLIENT_ID</span> to enable X import in this build.</p>
              {:else}
                <div class="space-y-2">
                  <div class="flex items-center justify-between gap-3"><span class="text-text-secondary">Session</span><span class={xSyncStatus?.connected ? 'text-emerald-400' : 'text-text-muted'}>{xSyncStatus?.connected ? 'Connected' : 'Needs sign-in'}</span></div>
                  <div class="flex items-center justify-between gap-3"><span class="text-text-secondary">Last sync</span><span class="text-text-primary">{formatDate(xSyncStatus?.last_synced_at ?? null)}</span></div>
                  {#if xSyncStatus?.last_error}
                    <p class="pt-1 text-xs text-amber-300">Last error: {xSyncStatus.last_error}</p>
                  {/if}
                </div>
              {/if}
            </div>

            <button class="accent-button mt-6 w-full" onclick={handleXImport} disabled={!canUseXImport}>{xButtonLabel}</button>
          </section>

          <section class="soft-panel rounded-[2rem] p-6">
            <p class="section-label">Fallback</p>
            <h3 class="mt-2 text-2xl font-medium text-text-primary">Import a file</h3>
            <p class="mt-3 text-sm leading-7 text-text-secondary">
              Use CSV, JSON, or archive JS when you want an offline recovery path or a one-off migration.
            </p>

            <div
              class={`mt-5 rounded-[1.6rem] border-2 border-dashed p-8 text-center transition-colors ${dragOver ? 'border-accent bg-accent/6' : 'border-border bg-bg-primary/25'}`}
              role="button"
              tabindex="0"
              aria-label="Import file drop zone"
              onclick={pickFile}
              onkeydown={(event) => { if (event.key === 'Enter' || event.key === ' ') { event.preventDefault(); pickFile(); } }}
              ondragover={(event) => {
                event.preventDefault();
                dragOver = true;
              }}
              ondragleave={() => (dragOver = false)}
              ondrop={handleDrop}
            >
              <svg class="mx-auto mb-3 h-10 w-10 text-text-muted" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" /></svg>
              <p class="text-text-primary">Drop an export here</p>
              <p class="mt-1 text-sm text-text-muted">or browse manually</p>
              <p class="mt-4 text-xs text-text-muted">{importGuidance}</p>
            </div>

            <button class="ghost-button mt-4 w-full" onclick={pickFile}>Browse files</button>
          </section>
        </div>
      {/if}
    </div>

    <input bind:this={fileInput} type="file" class="hidden" accept=".csv,.json,.js" onchange={handleFileChange} />
  </div>
</div>
