<script lang="ts">
  import { onMount } from 'svelte';
  import { importBookmarksFromX, importFileContent, getXSyncStatus } from '$lib/api';
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
  let isLoadingXStatus = $state(true);
  let fileInput: HTMLInputElement | null = null;
  const isTauri = typeof window !== 'undefined' && '__TAURI__' in window;

  onMount(async () => {
    await refreshXStatus();
  });

  async function refreshXStatus() {
    isLoadingXStatus = true;
    try {
      xSyncStatus = await getXSyncStatus();
    } catch (error) {
      console.error('Failed to load X sync status', error);
      xSyncStatus = null;
    } finally {
      isLoadingXStatus = false;
    }
  }

  async function handleFileSelect() {
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
      importResult = { kind: 'file', success: false, error: error instanceof Error ? error.message : 'Import failed' };
    } finally {
      isImporting = false;
    }
  }

  async function handleXImport() {
    isImporting = true;
    importResult = null;
    try {
      const summary = await importBookmarksFromX();
      importResult = {
        kind: 'x',
        success: true,
        count: summary.imported_count,
        skipped: summary.skipped_count,
        total: summary.total_fetched,
        lastSyncedAt: summary.last_synced_at,
        message: summary.status_message,
      };
      await refreshXStatus();
    } catch (error) {
      importResult = { kind: 'x', success: false, error: error instanceof Error ? error.message : 'X import failed' };
      await refreshXStatus();
    } finally {
      isImporting = false;
    }
  }

  async function handleDrop(e: DragEvent) {
    e.preventDefault();
    dragOver = false;
    if (!isTauri) {
      alert('Drag-and-drop import only works in the desktop app.');
      return;
    }
    const file = e.dataTransfer?.files?.[0];
    if (file) {
      await doFileImport(file);
    } else {
      alert('Unable to read dropped file. Please use the Browse button.');
    }
  }

  async function handleFileInputChange(e: Event) {
    const file = (e.currentTarget as HTMLInputElement).files?.[0];
    if (!file) return;
    await doFileImport(file);
    (e.currentTarget as HTMLInputElement).value = '';
  }

  const canUseXImport = $derived(Boolean(isTauri && xSyncStatus?.configured));
  const xButtonLabel = $derived(xSyncStatus?.last_synced_at ? 'Re-sync from X' : 'Import from X');

  function formatDate(value: string | null) {
    if (!value) return 'Never';
    return new Date(value).toLocaleString();
  }
</script>

<button class="fixed inset-0 z-50 border-none bg-black/65 backdrop-blur-sm" onclick={onclose} aria-label="Close modal"></button>

<div class="fixed inset-0 z-50 flex items-center justify-center p-4 pointer-events-none">
  <input
    bind:this={fileInput}
    type="file"
    accept=".csv,.json,.js"
    class="hidden"
    onchange={handleFileInputChange}
  />
  <div class="panel pointer-events-auto w-full max-w-4xl rounded-[2rem] overflow-hidden">
    <div class="flex items-center justify-between border-b border-border-subtle px-6 py-5">
      <div>
        <p class="eyebrow">Import flow</p>
        <h2 class="mt-1 font-display text-3xl italic text-text-primary">Bring new bookmarks in</h2>
      </div>
      <button onclick={onclose} class="flex h-11 w-11 items-center justify-center rounded-2xl border border-border bg-bg-secondary/75 text-text-muted transition-colors hover:text-text-primary" aria-label="Close modal">
        <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
      </button>
    </div>

    <div class="p-6">
      {#if importResult}
        <div class="mx-auto max-w-xl py-10 text-center">
          <div class="mx-auto flex h-16 w-16 items-center justify-center rounded-full bg-emerald-500/20 text-emerald-400">
            {#if importResult.success}
              <svg class="h-8 w-8" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" /></svg>
            {:else}
              <svg class="h-8 w-8 text-red-400" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
            {/if}
          </div>
          <h3 class="mt-5 text-2xl font-medium text-text-primary">{importResult.success ? 'Import complete' : 'Import failed'}</h3>
          {#if importResult.success && importResult.kind === 'x'}
            <p class="mt-3 text-text-secondary">{importResult.message}</p>
            <p class="mt-3 text-sm text-text-muted">Imported {importResult.count}, skipped {importResult.skipped}, fetched {importResult.total}</p>
            <p class="mt-1 text-xs text-text-muted">Last synced {formatDate(importResult.lastSyncedAt)}</p>
          {:else if importResult.success}
            <p class="mt-3 text-text-secondary">Successfully imported <span class="font-mono text-accent">{importResult.count.toLocaleString()}</span> bookmarks.</p>
          {:else}
            <p class="mt-3 text-text-secondary">{importResult.error}</p>
          {/if}
          <div class="mt-6 flex justify-center gap-3">
            <button onclick={() => (importResult = null)} class="rounded-full border border-border px-5 py-2 text-text-primary transition-colors hover:border-accent hover:text-accent">Back</button>
            <button onclick={onclose} class="rounded-full bg-accent px-5 py-2 text-white transition-opacity hover:opacity-90">Done</button>
          </div>
        </div>
      {:else if isImporting}
        <div class="py-14 text-center">
          <div class="mx-auto h-12 w-12 animate-spin rounded-full border-2 border-accent border-t-transparent"></div>
          <p class="mt-4 text-text-primary">Connecting and importing bookmarks…</p>
          <p class="mt-1 text-sm text-text-muted">Finish the browser login if prompted, then return to Eterea.</p>
        </div>
      {:else}
        <div class="grid gap-5 lg:grid-cols-[1.15fr,0.85fr]">
          <section class="rounded-[1.8rem] border border-border-accent bg-gradient-to-br from-accent/14 to-transparent p-6 shadow-[var(--shadow-glow)]">
            <p class="eyebrow">Recommended path</p>
            <h3 class="mt-2 text-2xl font-medium text-text-primary">Import directly from X</h3>
            <p class="mt-3 max-w-md text-sm leading-7 text-text-secondary">Connect in your browser and pull bookmarks straight into your local library. Re-sync later whenever you want fresh saves.</p>

            <div class="mt-5 rounded-[1.3rem] border border-border bg-bg-primary/35 p-4 text-sm">
              {#if isLoadingXStatus}
                <p class="text-text-muted">Checking X import availability…</p>
              {:else if !isTauri}
                <p class="text-amber-300">Available only inside the desktop app.</p>
              {:else if !canUseXImport}
                <p class="text-amber-300">This build is not configured yet. Set <span class="font-mono">ETEREA_X_CLIENT_ID</span> to enable X import.</p>
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

            <button onclick={handleXImport} disabled={!canUseXImport} class="mt-6 rounded-full bg-accent px-5 py-3 text-sm font-medium text-white transition-all hover:-translate-y-0.5 hover:opacity-95 disabled:cursor-not-allowed disabled:opacity-40">
              {xButtonLabel}
            </button>
          </section>

          <section class="rounded-[1.8rem] border border-border bg-bg-secondary/45 p-6">
            <p class="eyebrow">Fallback</p>
            <h3 class="mt-2 text-xl font-medium text-text-primary">Import a file</h3>
            <p class="mt-3 text-sm leading-7 text-text-secondary">Use CSV, JSON, or archive JS exports when you want an offline import path or a backfill from another tool.</p>

            <div class="mt-5 rounded-[1.4rem] border-2 border-dashed p-8 text-center transition-colors {dragOver ? 'border-accent bg-accent/6' : 'border-border bg-bg-primary/30'}" role="button" tabindex="0" aria-label="Import file drop zone" ondragover={(e) => { e.preventDefault(); dragOver = true; }} ondragleave={() => dragOver = false} ondrop={handleDrop}>
              <svg class="mx-auto mb-3 h-10 w-10 text-text-muted" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" /></svg>
              <p class="text-text-primary">Drop an export here</p>
              <p class="mt-1 text-sm text-text-muted">or browse manually</p>
              <button onclick={handleFileSelect} class="mt-4 rounded-full border border-border bg-bg-secondary/75 px-5 py-2 text-sm text-text-primary transition-colors hover:border-accent hover:text-accent">Browse files</button>
              <p class="mt-4 text-xs text-text-muted">Supports CSV, JSON, and X archive JS formats</p>
            </div>
          </section>
        </div>
      {/if}
    </div>
  </div>
</div>
