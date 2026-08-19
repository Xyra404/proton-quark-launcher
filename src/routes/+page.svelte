<script lang="ts">
  import { isUmuInstalled, listCollections } from '$lib/api';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import type { Collection } from '$lib/types';
  import GameList from '$lib/components/GameList.svelte';
  import SettingsDrawer from '$lib/components/SettingsDrawer.svelte';
  import CollectionSidebar from '$lib/components/CollectionSidebar.svelte';

  let umuPresent = $state<boolean | null>(null);
  let settingsOpen = $state(false);

  // Collections state
  let collections = $state<Collection[]>([]);
  let selectedCollectionId = $state<string>('all');

  function fetchCollections() {
    listCollections().then((cols) => {
      collections = cols;
    }).catch(console.error);
  }

  $effect(() => {
    isUmuInstalled().then((result) => {
      umuPresent = result;
    }).catch(() => {
      umuPresent = true;
    });
    fetchCollections();
  });

  async function openUmuLink(e: MouseEvent) {
    e.preventDefault();
    await openUrl('https://github.com/Open-Wine-Components/umu-launcher');
  }
</script>

<svelte:head>
  <title>Proton Quark Launcher</title>
  <meta name="description" content="Launch Windows games on Linux via Proton and umu-launcher" />
</svelte:head>

<div class="app-shell">
  <!-- Titlebar / Header -->
  <header class="app-header">
    <div class="header-brand">
      <span class="brand-icon">⚛</span>
      <span class="brand-name">Proton Quark Launcher</span>
    </div>
    <div class="header-actions">
      <!-- Settings Icon Button -->
      <button
        class="settings-btn"
        onclick={() => (settingsOpen = true)}
        aria-label="Open settings"
        title="Settings"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <circle cx="12" cy="12" r="3"></circle>
          <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
        </svg>
      </button>
    </div>
  </header>

  <!-- Two-column main layout -->
  <div class="app-body">
    <CollectionSidebar
      {collections}
      selectedId={selectedCollectionId}
      onselect={(id) => (selectedCollectionId = id)}
      oncollectionschanged={fetchCollections}
    />

    <main class="app-content">
      <GameList
        {collections}
        selectedCollectionId={selectedCollectionId}
        oncollectionschanged={fetchCollections}
      />
    </main>
  </div>

  <!-- App Footer -->
  <footer class="app-footer">
    <div class="footer-status">
      {#if umuPresent === true}
        <span class="status-dot ok" title="umu-launcher detected"></span>
        <span class="status-label">umu-launcher ready</span>
      {:else if umuPresent === false}
        <span class="status-dot warn" title="umu-launcher not found"></span>
        <span class="status-label warn">
          <strong>Fallback mode:</strong> umu-launcher not detected (using raw Proton).
          <!-- svelte-ignore a11y_missing_attribute -->
          <a href="https://github.com/Open-Wine-Components/umu-launcher" onclick={openUmuLink}>
            Install umu-launcher
          </a>
        </span>
      {/if}
    </div>
  </footer>
</div>

<!-- Settings Drawer -->
<SettingsDrawer bind:open={settingsOpen} onclose={() => (settingsOpen = false)} />

<style>
  :global(*), :global(*::before), :global(*::after) {
    box-sizing: border-box;
  }

  :global(html), :global(body) {
    color-scheme: dark;
    margin: 0;
    padding: 0;
    height: 100%;
    background: #0d0d1a;
    color: #c0c0e0;
    font-family: 'Inter', 'Segoe UI', system-ui, sans-serif;
    font-size: 15px;
    line-height: 1.5;
    -webkit-font-smoothing: antialiased;
  }

  :global(::selection) {
    background: #4040c0;
    color: #fff;
  }

  :global(input), :global(select), :global(button), :global(textarea) {
    font-family: inherit;
    font-size: inherit;
  }

  /* ── App shell ─────────────────────────────────────────────────────────── */
  .app-shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }

  /* ── App Body (Two Columns) ──────────────────────────────────────────────── */
  .app-body {
    display: flex;
    flex: 1;
    overflow: hidden; /* Sidebar internal scrolling */
  }

  /* ── Header ──────────────────────────────────────────────────────────────── */
  .app-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 1.5rem;
    height: 52px;
    background: #0a0a18;
    border-bottom: 1px solid #1a1a3a;
    flex-shrink: 0;
    position: sticky;
    top: 0;
    z-index: 10;
  }

  .header-brand {
    display: flex;
    align-items: center;
    gap: 0.6rem;
  }

  .brand-icon {
    font-size: 1.35rem;
    line-height: 1;
    background: linear-gradient(135deg, #6060e0, #a050e0);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  .brand-name {
    font-size: 0.95rem;
    font-weight: 700;
    letter-spacing: 0.02em;
    color: #c0c0ff;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .settings-btn {
    background: none;
    border: 1px solid transparent;
    border-radius: 50%;
    color: #5050a0;
    cursor: pointer;
    padding: 0.35rem;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: color 0.15s, border-color 0.15s, background 0.15s, transform 0.25s;
  }

  .settings-btn:hover {
    color: #a0a0ff;
    border-color: #2a2a60;
    background: #1a1a38;
    transform: rotate(45deg);
  }

  /* ── Main content ────────────────────────────────────────────────────────── */
  .app-content {
    flex: 1;
    padding: 1.75rem 2rem;
    overflow-y: auto;
  }

  /* ── Footer ───────────────────────────────────────────────────────────────── */
  .app-footer {
    height: 36px;
    background: #080814;
    border-top: 1px solid #1a1a38;
    display: flex;
    align-items: center;
    padding: 0 1.25rem;
    font-size: 0.76rem;
    flex-shrink: 0;
  }

  .footer-status {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    color: #6060a0;
  }

  .status-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .status-dot.ok   { background: #40c040; box-shadow: 0 0 5px #40c04088; }
  .status-dot.warn { background: #e09020; box-shadow: 0 0 5px #e0902088; }

  .status-label {
    color: #5050a0;
  }

  .status-label.warn {
    color: #a07030;
  }

  .status-label a {
    color: #7090d0;
    text-decoration: underline;
    margin-left: 0.25rem;
    cursor: pointer;
  }

  .status-label a:hover {
    color: #90b0ff;
  }
</style>
