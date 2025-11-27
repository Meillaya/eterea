<script lang="ts">
  import { importFile } from '$lib/api';
  import { open as openDialog } from '@tauri-apps/api/dialog';
  
  interface Props {
    onclose: () => void;
  }
  
  let { onclose }: Props = $props();
  
  let isImporting = $state(false);
  let importResult = $state<{ success: boolean; count: number; error?: string } | null>(null);
  let dragOver = $state(false);
  const isTauri = typeof window !== 'undefined' && '__TAURI__' in window;
  
  async function handleFileSelect() {
    if (!isTauri) {
      alert('Import is only available inside the desktop app.');
      return;
    }
    try {
      const selected = await openDialog({
        multiple: false,
        filters: [{
          name: 'Bookmark Files',
          extensions: ['csv', 'json']
        }]
      });
      
      if (typeof selected === 'string') {
        await doImport(selected);
      }
    } catch (error) {
      console.error('File dialog failed', error);
    }
  }
  
  async function doImport(path: string) {
    isImporting = true;
    importResult = null;
    
    try {
      const count = await importFile(path);
      importResult = { success: true, count };
    } catch (error) {
      importResult = { 
        success: false, 
        count: 0, 
        error: error instanceof Error ? error.message : 'Import failed' 
      };
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
    
    const file = e.dataTransfer?.files?.[0] as File & { path?: string };
    if (file?.path) {
      await doImport(file.path);
    } else {
      alert('Unable to read dropped file path. Please use the Browse button.');
    }
  }
  
  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      onclose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- Backdrop -->
<button 
  class="fixed inset-0 bg-black/60 backdrop-blur-sm z-50 animate-fade-in cursor-default border-none"
  onclick={onclose}
  aria-label="Close modal"
></button>

<!-- Modal -->
<div class="fixed inset-0 flex items-center justify-center z-50 p-4 pointer-events-none">
  <div 
    class="bg-bg-secondary border border-border rounded-2xl shadow-2xl w-full max-w-lg pointer-events-auto animate-slide-up"
    role="dialog"
    aria-modal="true"
    aria-labelledby="modal-title"
  >
    <!-- Header -->
    <div class="flex items-center justify-between p-6 border-b border-border-subtle">
      <h2 id="modal-title" class="text-xl font-display italic text-text-primary">
        Import Bookmarks
      </h2>
      <button
        onclick={onclose}
        class="p-2 rounded-lg text-text-muted hover:text-text-primary hover:bg-bg-tertiary transition-colors"
        title="Close"
        aria-label="Close modal"
      >
        <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </div>
    
    <!-- Content -->
    <div class="p-6">
      {#if importResult}
        <!-- Result -->
        <div class="text-center py-8">
          {#if importResult.success}
            <div class="w-16 h-16 mx-auto mb-4 rounded-full bg-emerald-500/20 flex items-center justify-center">
              <svg class="w-8 h-8 text-emerald-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
            </div>
            <h3 class="text-lg font-medium text-text-primary mb-2">Import Complete!</h3>
            <p class="text-text-secondary">
              Successfully imported <span class="font-mono text-accent">{importResult.count.toLocaleString()}</span> bookmarks
            </p>
          {:else}
            <div class="w-16 h-16 mx-auto mb-4 rounded-full bg-red-500/20 flex items-center justify-center">
              <svg class="w-8 h-8 text-red-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </div>
            <h3 class="text-lg font-medium text-text-primary mb-2">Import Failed</h3>
            <p class="text-text-secondary">{importResult.error}</p>
          {/if}
          
          <button
            onclick={onclose}
            class="mt-6 px-6 py-2 bg-accent text-white rounded-lg hover:opacity-90 transition-colors"
          >
            Done
          </button>
        </div>
      {:else if isImporting}
        <!-- Loading -->
        <div class="text-center py-12">
          <div class="w-12 h-12 mx-auto mb-4 border-2 border-accent border-t-transparent rounded-full animate-spin"></div>
          <p class="text-text-secondary">Importing bookmarks...</p>
          <p class="text-sm text-text-muted mt-1">This may take a moment</p>
        </div>
      {:else}
        <!-- Drop Zone -->
        <div
          class="relative border-2 border-dashed rounded-xl p-12 text-center transition-colors {dragOver ? 'border-accent bg-accent/5' : 'border-border'}"
          ondragover={(e) => { e.preventDefault(); dragOver = true; }}
          ondragleave={() => dragOver = false}
          ondrop={handleDrop}
        >
          <svg class="w-12 h-12 mx-auto mb-4 text-text-muted" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
          </svg>
          
          <p class="text-text-primary mb-2">Drop your file here</p>
          <p class="text-sm text-text-muted mb-4">or</p>
          
          <button
            onclick={handleFileSelect}
            class="px-6 py-2 bg-accent text-white rounded-lg hover:opacity-90 transition-colors"
          >
            Browse Files
          </button>
          
          <p class="mt-4 text-xs text-text-muted">
            Supports CSV and JSON formats
          </p>
        </div>
        
        <!-- Supported Formats -->
        <div class="mt-6 grid grid-cols-2 gap-4">
          <div class="p-4 bg-bg-tertiary rounded-lg">
            <div class="flex items-center gap-2 mb-2">
              <span class="font-mono text-sm text-accent">.csv</span>
              <span class="text-xs text-text-muted">Dewey / Twitter</span>
            </div>
            <p class="text-xs text-text-secondary">
              Export from Dewey or other Twitter bookmark managers
            </p>
          </div>
          <div class="p-4 bg-bg-tertiary rounded-lg">
            <div class="flex items-center gap-2 mb-2">
              <span class="font-mono text-sm text-accent">.json</span>
              <span class="text-xs text-text-muted">Twitter Archive</span>
            </div>
            <p class="text-xs text-text-secondary">
              JSON export from Twitter/X data archive
            </p>
          </div>
        </div>
      {/if}
    </div>
  </div>
</div>

