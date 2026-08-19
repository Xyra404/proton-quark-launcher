<script lang="ts">
  import { open as openDialog } from '@tauri-apps/plugin-dialog';
  import {
    listCustomProtonPaths,
    addCustomProtonPath,
    removeCustomProtonPath,
  } from '$lib/api';

  interface Props {
    onpathschange?: () => void;
  }

  let { onpathschange }: Props = $props();

  let paths = $state<string[]>([]);
  let loading = $state(true);
  let adding = $state(false);
  let removingPath = $state<string | null>(null);
  let errorMsg = $state('');

  // ── Load custom paths on mount ───────────────────────────────────────────────
  $effect(() => {
    fetchPaths();
  });

  async function fetchPaths() {
    loading = true;
    errorMsg = '';
    try {
      paths = await listCustomProtonPaths();
    } catch (e: unknown) {
      errorMsg = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  // ── Add Folder (Directory Picker) ───────────────────────────────────────────
  async function handleAddFolder() {
    errorMsg = '';
    try {
      const selected = await openDialog({
        directory: true,
        multiple: false,
        title: 'Select Proton Installation Folder',
      });

      if (selected && typeof selected === 'string') {
        adding = true;
        await addCustomProtonPath(selected);
        await fetchPaths();
        onpathschange?.();
      }
    } catch (err: unknown) {
      errorMsg = err instanceof Error ? err.message : String(err);
    } finally {
      adding = false;
    }
  }

  // ── Remove Path ─────────────────────────────────────────────────────────────
  async function handleRemove(path: string) {
    errorMsg = '';
    removingPath = path;
    try {
      await removeCustomProtonPath(path);
      await fetchPaths();
      onpathschange?.();
    } catch (err: unknown) {
      errorMsg = err instanceof Error ? err.message : String(err);
    } finally {
      removingPath = null;
    }
  }
</script>

<div class="custom-paths-card">
  <div class="card-header">
    <div class="header-text">
      <h3 class="section-title">
        <span class="icon">📁</span> Custom Proton Locations
      </h3>
      <p class="section-desc">
        Add custom Proton builds stored outside standard Steam directories (e.g. tarball extractions or custom compilations).
      </p>
    </div>
    <button
      type="button"
      class="btn-add-folder"
      onclick={handleAddFolder}
      disabled={adding}
      aria-label="Add custom Proton directory"
    >
      {#if adding}
        <span class="spinner"></span> Adding…
      {:else}
        <span>＋</span> Add Folder…
      {/if}
    </button>
  </div>

  <!-- Inline Error Notification -->
  {#if errorMsg}
    <div class="error-banner" role="alert">
      <span class="error-icon">⚠</span>
      <span class="error-text">{errorMsg}</span>
      <button
        type="button"
        class="error-dismiss"
        onclick={() => (errorMsg = '')}
        aria-label="Dismiss error"
      >
        ✕
      </button>
    </div>
  {/if}

  <!-- Paths List -->
  {#if loading}
    <div class="status-msg">
      <div class="spinner"></div>
      <span>Loading custom paths…</span>
    </div>
  {:else if paths.length === 0}
    <div class="empty-custom-paths">
      <p>No custom Proton folders registered.</p>
      <p class="empty-sub">
        Standard Steam and system compatibility tools are discovered automatically.
      </p>
    </div>
  {:else}
    <ul class="paths-list" aria-label="Registered custom Proton directories">
      {#each paths as path (path)}
        <li class="path-row">
          <div class="path-info">
            <span class="folder-badge">Proton</span>
            <span class="path-string" title={path}>{path}</span>
          </div>
          <button
            type="button"
            class="btn-remove"
            disabled={removingPath === path}
            onclick={() => handleRemove(path)}
            aria-label="Remove custom path {path}"
          >
            {#if removingPath === path}
              Removing…
            {:else}
              Remove
            {/if}
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .custom-paths-card {
    background: #111128;
    border: 1px solid #1e1e40;
    border-radius: 10px;
    padding: 1.25rem 1.4rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .card-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 1rem;
    flex-wrap: wrap;
  }

  .header-text {
    flex: 1;
    min-width: 240px;
  }

  .section-title {
    margin: 0 0 0.35rem;
    font-size: 1.05rem;
    color: #c0c0ff;
    font-weight: 600;
    display: flex;
    align-items: center;
    gap: 0.45rem;
  }

  .section-title .icon {
    font-size: 1.05rem;
  }

  .section-desc {
    margin: 0;
    font-size: 0.82rem;
    color: #7070a8;
    line-height: 1.4;
  }

  .btn-add-folder {
    background: #252550;
    border: 1px solid #3a3a80;
    border-radius: 7px;
    color: #c0c0ff;
    padding: 0.5rem 0.95rem;
    font-size: 0.85rem;
    font-weight: 500;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 0.4rem;
    white-space: nowrap;
    transition: background 0.15s, border-color 0.15s, color 0.15s;
    flex-shrink: 0;
  }

  .btn-add-folder:hover:not(:disabled) {
    background: #353575;
    border-color: #5555c0;
    color: #ffffff;
  }

  .btn-add-folder:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }

  /* ── Error Banner ───────────────────────────────────────────────────────── */
  .error-banner {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    background: #2a1515;
    border: 1px solid #5a2020;
    border-radius: 7px;
    padding: 0.65rem 0.9rem;
    color: #e07070;
    font-size: 0.85rem;
  }

  .error-icon {
    font-size: 1rem;
    flex-shrink: 0;
  }

  .error-text {
    flex: 1;
    line-height: 1.35;
  }

  .error-dismiss {
    background: none;
    border: none;
    color: #a05050;
    cursor: pointer;
    font-size: 0.85rem;
    padding: 0.2rem 0.4rem;
    border-radius: 4px;
  }

  .error-dismiss:hover {
    color: #e07070;
  }

  /* ── List / Empty / Loading ─────────────────────────────────────────────── */
  .status-msg {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    color: #6060a0;
    font-size: 0.85rem;
    padding: 1rem 0;
  }

  .spinner {
    width: 14px;
    height: 14px;
    border: 2px solid transparent;
    border-top-color: #8080c0;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
    flex-shrink: 0;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .empty-custom-paths {
    background: #0d0d20;
    border: 1px dashed #202048;
    border-radius: 8px;
    padding: 1.25rem 1rem;
    text-align: center;
  }

  .empty-custom-paths p {
    margin: 0 0 0.25rem;
    font-size: 0.85rem;
    color: #606095;
  }

  .empty-custom-paths .empty-sub {
    font-size: 0.78rem;
    color: #484875;
  }

  .paths-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
  }

  .path-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    background: #0d0d20;
    border: 1px solid #1a1a38;
    border-radius: 7px;
    padding: 0.55rem 0.85rem;
    transition: border-color 0.15s;
  }

  .path-row:hover {
    border-color: #2a2a5a;
  }

  .path-info {
    display: flex;
    align-items: center;
    gap: 0.65rem;
    min-width: 0;
    flex: 1;
  }

  .folder-badge {
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    background: #1e1e48;
    color: #8080d0;
    border: 1px solid #2a2a68;
    border-radius: 4px;
    padding: 0.15rem 0.45rem;
    font-weight: 600;
    flex-shrink: 0;
  }

  .path-string {
    font-family: 'Courier New', monospace;
    font-size: 0.8rem;
    color: #b0b0e0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .btn-remove {
    background: none;
    border: 1px solid #4a2020;
    border-radius: 5px;
    color: #d06060;
    font-size: 0.75rem;
    padding: 0.3rem 0.65rem;
    cursor: pointer;
    flex-shrink: 0;
    transition: background 0.15s, border-color 0.15s, color 0.15s;
  }

  .btn-remove:hover:not(:disabled) {
    background: #301010;
    border-color: #702828;
    color: #ff8080;
  }

  .btn-remove:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
