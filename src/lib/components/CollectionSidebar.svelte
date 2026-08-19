<script lang="ts">
  import { createCollection, deleteCollection, renameCollection } from '$lib/api';
  import type { Collection } from '$lib/types';

  interface Props {
    collections: Collection[];
    selectedId: string; // 'all', 'uncategorized', or a collection ID
    onselect: (id: string) => void;
    oncollectionschanged: () => void;
  }

  let { collections, selectedId, onselect, oncollectionschanged }: Props = $props();

  // ── Inline Add State ────────────────────────────────────────────────────────
  let isAdding = $state(false);
  let newName = $state('');
  let addError = $state('');
  let addInputRef = $state<HTMLInputElement | null>(null);

  function startAdding() {
    isAdding = true;
    newName = '';
    addError = '';
    setTimeout(() => addInputRef?.focus(), 10);
  }

  function cancelAdding() {
    isAdding = false;
    newName = '';
    addError = '';
  }

  async function submitAdd() {
    const trimmed = newName.trim();
    if (!trimmed) {
      cancelAdding();
      return;
    }
    try {
      await createCollection(trimmed);
      isAdding = false;
      oncollectionschanged();
    } catch (e: unknown) {
      addError = e instanceof Error ? e.message : String(e);
    }
  }

  function handleAddKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') submitAdd();
    if (e.key === 'Escape') cancelAdding();
  }

  // ── Context Menu State (Rename/Delete) ─────────────────────────────────────
  let contextMenuOpen = $state(false);
  let contextX = $state(0);
  let contextY = $state(0);
  let contextTarget = $state<Collection | null>(null);

  // Rename inline state
  let renamingId = $state<string | null>(null);
  let renameValue = $state('');
  let renameInputRef = $state<HTMLInputElement | null>(null);

  function openContextMenu(e: MouseEvent, coll: Collection) {
    e.preventDefault();
    contextMenuOpen = true;
    contextTarget = coll;
    // Basic positioning (can be improved to stay within viewport)
    contextX = e.clientX;
    contextY = e.clientY;
  }

  function closeContextMenu() {
    contextMenuOpen = false;
    contextTarget = null;
  }

  // Handle outside clicks for context menu
  function handleWindowClick() {
    if (contextMenuOpen) closeContextMenu();
  }

  // ── Context Menu Actions ────────────────────────────────────────────────────
  function handleRenameStart() {
    if (!contextTarget) return;
    renamingId = contextTarget.id;
    renameValue = contextTarget.name;
    closeContextMenu();
    setTimeout(() => renameInputRef?.focus(), 10);
  }

  async function handleDelete() {
    if (!contextTarget) return;

    // Capture BOTH values before closeContextMenu() nulls out contextTarget.
    // The previous version read contextTarget.name inside the confirm()
    // call after closeContextMenu() had already run, which threw a
    // "Cannot read properties of null" error and silently aborted the
    // function before deleteCollection() was ever called.
    const targetId = contextTarget.id;
    const targetName = contextTarget.name;
    closeContextMenu();

    if (confirm(`Delete collection "${targetName}"?\nGames inside will not be deleted.`)) {
      try {
        await deleteCollection(targetId);
        if (selectedId === targetId) {
          onselect('all');
        }
        oncollectionschanged();
      } catch (e: unknown) {
        alert(`Failed to delete: ${e instanceof Error ? e.message : String(e)}`);
      }
    }
  }

  async function submitRename() {
    if (!renamingId) return;
    const trimmed = renameValue.trim();
    if (trimmed) {
      try {
        await renameCollection(renamingId, trimmed);
        oncollectionschanged();
      } catch (e: unknown) {
        alert(`Failed to rename: ${e instanceof Error ? e.message : String(e)}`);
      }
    }
    renamingId = null;
  }

  function handleRenameKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') submitRename();
    if (e.key === 'Escape') renamingId = null;
  }

</script>

<svelte:window onclick={handleWindowClick} />

<aside class="sidebar">
  <div class="sidebar-header">
    <h2 class="sidebar-title">Library</h2>
  </div>

  <nav class="sidebar-nav">
    <!-- Static items -->
    <button
      class="nav-item {selectedId === 'all' ? 'active' : ''}"
      onclick={() => onselect('all')}
    >
      <span class="nav-icon">
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 19.5v-15A2.5 2.5 0 0 1 6.5 2H20v20H6.5a2.5 2.5 0 0 1 0-5H20"></path></svg>
      </span>
      All Games
    </button>
    <button
      class="nav-item {selectedId === 'uncategorized' ? 'active' : ''}"
      onclick={() => onselect('uncategorized')}
    >
      <span class="nav-icon">
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="7" height="7"></rect><rect x="14" y="3" width="7" height="7"></rect><rect x="14" y="14" width="7" height="7"></rect><rect x="3" y="14" width="7" height="7"></rect></svg>
      </span>
      Uncategorized
    </button>

    <hr class="nav-divider" />

    <!-- Collections -->
    <div class="collections-header">
      <span class="col-label">Collections</span>
      <button class="add-btn" onclick={startAdding} aria-label="New Collection" title="New Collection">
        <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="5" x2="12" y2="19"></line><line x1="5" y1="12" x2="19" y2="12"></line></svg>
      </button>
    </div>

    <ul class="col-list">
      {#each collections as coll (coll.id)}
        <li>
          {#if renamingId === coll.id}
            <div class="inline-input-wrap">
              <!-- svelte-ignore a11y_autofocus -->
              <input
                bind:this={renameInputRef}
                type="text"
                class="inline-input"
                bind:value={renameValue}
                onkeydown={handleRenameKeydown}
                onblur={submitRename}
              />
            </div>
          {:else}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              class="nav-item col-item {selectedId === coll.id ? 'active' : ''}"
              onclick={() => onselect(coll.id)}
              oncontextmenu={(e) => openContextMenu(e, coll)}
            >
              <span class="nav-icon">
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path></svg>
              </span>
              <span class="col-name">{coll.name}</span>
            </div>
          {/if}
        </li>
      {/each}

      <!-- Inline Add -->
      {#if isAdding}
        <li>
          <div class="inline-input-wrap">
            <input
              bind:this={addInputRef}
              type="text"
              class="inline-input {addError ? 'error' : ''}"
              placeholder="Collection name..."
              bind:value={newName}
              onkeydown={handleAddKeydown}
              onblur={cancelAdding}
              title={addError}
            />
          </div>
        </li>
      {/if}
    </ul>
  </nav>
</aside>

<!-- Context Menu -->
{#if contextMenuOpen}
  <div
    class="context-menu"
    style="left: {contextX}px; top: {contextY}px;"
  >
    <button class="ctx-item" onclick={handleRenameStart}>
      <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 20h9"></path><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"></path></svg>
      Rename
    </button>
    <button class="ctx-item danger" onclick={handleDelete}>
      <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg>
      Delete Collection
    </button>
  </div>
{/if}

<style>
  .sidebar {
    width: 240px;
    background: #0b0b1a;
    border-right: 1px solid #1a1a3a;
    display: flex;
    flex-direction: column;
    height: 100%;
    flex-shrink: 0;
  }

  .sidebar-header {
    padding: 1.25rem 1.25rem 0.75rem;
  }

  .sidebar-title {
    margin: 0;
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: #5050a0;
    font-weight: 700;
  }

  .sidebar-nav {
    flex: 1;
    overflow-y: auto;
    padding: 0 0.75rem 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    width: 100%;
    background: transparent;
    border: none;
    padding: 0.5rem 0.75rem;
    border-radius: 6px;
    color: #a0a0c0;
    font-size: 0.88rem;
    cursor: pointer;
    text-align: left;
    transition: background 0.15s, color 0.15s;
  }

  .nav-item:hover {
    background: #1a1a3a;
    color: #d0d0ff;
  }

  .nav-item.active {
    background: #252550;
    color: #fff;
    font-weight: 500;
  }

  .nav-icon {
    color: #6060a0;
    display: flex;
  }

  .nav-item:hover .nav-icon,
  .nav-item.active .nav-icon {
    color: #8080e0;
  }

  .nav-divider {
    border: none;
    border-top: 1px solid #1a1a3a;
    margin: 0.75rem 0.5rem;
  }

  .collections-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 0.75rem;
    margin-bottom: 0.4rem;
  }

  .col-label {
    font-size: 0.72rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: #5050a0;
    font-weight: 600;
  }

  .add-btn {
    background: none;
    border: none;
    color: #5050a0;
    cursor: pointer;
    padding: 0.2rem;
    border-radius: 4px;
    display: flex;
  }

  .add-btn:hover {
    background: #252550;
    color: #a0a0ff;
  }

  .col-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
  }

  .col-item {
    user-select: none;
  }

  .col-name {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* ── Inline Input ──────────────────────────────────────────────────────── */
  .inline-input-wrap {
    padding: 0.25rem 0.75rem;
  }

  .inline-input {
    width: 100%;
    background: #12122a;
    border: 1px solid #4040a0;
    border-radius: 4px;
    color: #fff;
    padding: 0.35rem 0.5rem;
    font-size: 0.85rem;
    outline: none;
  }

  .inline-input.error {
    border-color: #a04040;
  }

  .inline-input:focus {
    border-color: #6060e0;
    box-shadow: 0 0 0 2px rgba(96, 96, 224, 0.25);
  }

  /* ── Context Menu ──────────────────────────────────────────────────────── */
  .context-menu {
    position: fixed;
    background: #1a1a35;
    border: 1px solid #30305a;
    border-radius: 6px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
    padding: 0.25rem;
    display: flex;
    flex-direction: column;
    min-width: 150px;
    z-index: 1000;
  }

  .ctx-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background: none;
    border: none;
    color: #c0c0e0;
    padding: 0.45rem 0.75rem;
    font-size: 0.82rem;
    cursor: pointer;
    border-radius: 4px;
    text-align: left;
  }

  .ctx-item:hover {
    background: #252550;
    color: #fff;
  }

  .ctx-item.danger {
    color: #e06060;
  }

  .ctx-item.danger:hover {
    background: #351a1a;
    color: #ff8080;
  }
</style>
