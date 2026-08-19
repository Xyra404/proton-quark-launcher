<script lang="ts">
  import { listGames, removeGame, launchGame } from '$lib/api';
  import type { Game } from '$lib/types';
  import Toast from './Toast.svelte';
  import AddGameModal from './AddGameModal.svelte';

  // ── State ────────────────────────────────────────────────────────────────────
  let games = $state<Game[]>([]);
  let loading = $state(true);
  let loadError = $state('');
  let toastMsg = $state('');

  // Track which game IDs are currently mid-launch (for per-button spinner).
  let launchingIds = $state<Set<string>>(new Set());

  let addModalOpen = $state(false);
  let gameToEdit = $state<Game | undefined>(undefined);

  // Track delete confirmation states
  let pendingDeleteIds = $state<Set<string>>(new Set());
  let deleteTimers = new Map<string, ReturnType<typeof setTimeout>>();

  // ── Load games on mount ───────────────────────────────────────────────────────
  $effect(() => {
    fetchGames();
  });

  async function fetchGames() {
    loading = true;
    loadError = '';
    try {
      games = await listGames();
    } catch (e: unknown) {
      loadError = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  // ── Launch ────────────────────────────────────────────────────────────────────
  async function handleLaunch(game: Game) {
    launchingIds = new Set([...launchingIds, game.id]);
    try {
      await launchGame(game);
      // Refresh so last_played updates in the UI.
      await fetchGames();
    } catch (e: unknown) {
      toastMsg = e instanceof Error ? e.message : String(e);
    } finally {
      launchingIds = new Set([...launchingIds].filter((id) => id !== game.id));
    }
  }

  // ── Delete ────────────────────────────────────────────────────────────────────
  async function handleDeleteClick(game: Game) {
    if (pendingDeleteIds.has(game.id)) {
      // Confirmed
      clearTimeout(deleteTimers.get(game.id));
      deleteTimers.delete(game.id);
      pendingDeleteIds = new Set([...pendingDeleteIds].filter((id) => id !== game.id));
      
      try {
        await removeGame(game.id);
        games = games.filter((g) => g.id !== game.id);
      } catch (e: unknown) {
        toastMsg = e instanceof Error ? e.message : String(e);
      }
    } else {
      // Ask for confirmation
      pendingDeleteIds = new Set([...pendingDeleteIds, game.id]);
      
      const timer = setTimeout(() => {
        pendingDeleteIds = new Set([...pendingDeleteIds].filter((id) => id !== game.id));
        deleteTimers.delete(game.id);
      }, 3000);
      
      deleteTimers.set(game.id, timer);
    }
  }

  // ── Edit ──────────────────────────────────────────────────────────────────────
  function handleEdit(game: Game) {
    gameToEdit = game;
    addModalOpen = true;
  }

  function handleModalClose() {
    addModalOpen = false;
    gameToEdit = undefined;
  }

  // ── Helpers ───────────────────────────────────────────────────────────────────
  function formatDate(iso?: string): string {
    if (!iso) return 'Never played';
    try {
      return new Intl.DateTimeFormat(undefined, {
        dateStyle: 'medium',
        timeStyle: 'short',
      }).format(new Date(iso));
    } catch {
      return iso;
    }
  }
</script>

<div class="game-list-root">
  <div class="list-header">
    <h2 class="list-title">
      <span class="title-icon">🎮</span>
      My Games
      {#if !loading}
        <span class="count-badge">{games.length}</span>
      {/if}
    </h2>
    <button class="add-btn" onclick={() => (addModalOpen = true)}>
      <span>＋</span> Add Game
    </button>
  </div>

  {#if loading}
    <div class="state-msg">
      <div class="spinner"></div>
      <span>Loading games…</span>
    </div>
  {:else if loadError}
    <div class="state-msg error">
      <span>Failed to load games: {loadError}</span>
      <button class="retry-btn" onclick={fetchGames}>Retry</button>
    </div>
  {:else if games.length === 0}
    <div class="empty-state">
      <div class="empty-icon">📂</div>
      <p>No games added yet.</p>
      <p class="empty-sub">Click <strong>Add Game</strong> to get started.</p>
    </div>
  {:else}
    <ul class="game-cards" aria-label="Game library">
      {#each games as game (game.id)}
        {@const isLaunching = launchingIds.has(game.id)}
        <li class="game-card">
          <div class="card-body">
            <div class="game-info">
              <span class="game-name">{game.name}</span>
              <span class="game-meta">
                <span class="meta-proton" title={game.proton_path}>
                  ⚙ {game.proton_version}
                </span>
                <span class="meta-sep">·</span>
                <span class="meta-played" title={game.last_played ?? ''}>
                  🕐 {formatDate(game.last_played)}
                </span>
              </span>
              <span class="game-exe" title={game.exe_path}>{game.exe_path}</span>
            </div>

            <div class="card-actions">
              <button
                class="btn-launch"
                onclick={() => handleLaunch(game)}
                disabled={isLaunching}
                aria-label="Launch {game.name}"
              >
                {#if isLaunching}
                  <span class="btn-spinner"></span> Launching…
                {:else}
                  ▶ Launch
                {/if}
              </button>
              <button
                class="btn-icon btn-edit"
                onclick={() => handleEdit(game)}
                aria-label="Edit {game.name}"
                title="Edit game"
              >
                ✎
              </button>
              <button
                class="btn-icon btn-delete"
                class:confirming={pendingDeleteIds.has(game.id)}
                onclick={() => handleDeleteClick(game)}
                aria-label="Delete {game.name}"
                title={pendingDeleteIds.has(game.id) ? "Confirm delete" : "Remove game"}
              >
                {pendingDeleteIds.has(game.id) ? 'Sure?' : '🗑'}
              </button>
            </div>
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<!-- Add/Edit Game Modal -->
<AddGameModal
  bind:open={addModalOpen}
  existingGame={gameToEdit}
  onclose={handleModalClose}
  ongameadded={fetchGames}
/>

<!-- Error toast -->
{#if toastMsg}
  <Toast message={toastMsg} ondismiss={() => (toastMsg = '')} />
{/if}

<style>
  .game-list-root {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }

  /* ── Header ─────────────────────────────────────────────────────────────── */
  .list-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: 0.75rem;
  }

  .list-title {
    margin: 0;
    font-size: 1.25rem;
    color: #c0c0ff;
    font-weight: 600;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .title-icon { font-size: 1rem; }

  .count-badge {
    font-size: 0.75rem;
    background: #252550;
    color: #8080c0;
    border: 1px solid #3030a0;
    border-radius: 12px;
    padding: 0.1em 0.55em;
    font-weight: 500;
  }

  .add-btn {
    background: #4040c0;
    border: 1px solid #6060e0;
    border-radius: 8px;
    color: #fff;
    padding: 0.55rem 1.1rem;
    font-size: 0.88rem;
    font-weight: 500;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 0.4rem;
    transition: background 0.15s;
  }

  .add-btn:hover { background: #5555d5; }

  /* ── States ─────────────────────────────────────────────────────────────── */
  .state-msg {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.75rem;
    color: #6060a0;
    font-size: 0.9rem;
    padding: 3rem 0;
  }

  .state-msg.error { color: #e06060; }

  .retry-btn {
    background: none;
    border: 1px solid #5a2020;
    border-radius: 5px;
    color: #e06060;
    padding: 0.3rem 0.7rem;
    font-size: 0.82rem;
    cursor: pointer;
  }

  .spinner,
  .btn-spinner {
    width: 16px;
    height: 16px;
    border: 2px solid transparent;
    border-top-color: #8080c0;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
    flex-shrink: 0;
  }

  .btn-spinner {
    width: 12px;
    height: 12px;
    border-top-color: #fff;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .empty-state {
    text-align: center;
    padding: 4rem 0;
    color: #5050a0;
  }

  .empty-icon { font-size: 3rem; margin-bottom: 0.75rem; }

  .empty-state p { margin: 0 0 0.4rem; font-size: 0.95rem; }

  .empty-sub { color: #404080; font-size: 0.85rem !important; }

  /* ── Cards ──────────────────────────────────────────────────────────────── */
  .game-cards {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
  }

  .game-card {
    background: #111128;
    border: 1px solid #1e1e40;
    border-radius: 10px;
    transition: border-color 0.15s, box-shadow 0.15s;
  }

  .game-card:hover {
    border-color: #3030a0;
    box-shadow: 0 2px 16px rgba(60, 60, 180, 0.12);
  }

  .card-body {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.9rem 1.1rem;
    flex-wrap: wrap;
  }

  .game-info {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    min-width: 0;
    flex: 1;
  }

  .game-name {
    font-size: 1rem;
    font-weight: 600;
    color: #d0d0ff;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .game-meta {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    flex-wrap: wrap;
  }

  .meta-proton,
  .meta-played {
    font-size: 0.78rem;
    color: #6060a0;
  }

  .meta-sep { color: #3030a0; }

  .game-exe {
    font-size: 0.72rem;
    color: #404070;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-family: 'Courier New', monospace;
  }

  /* ── Card Actions ───────────────────────────────────────────────────────── */
  .card-actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-shrink: 0;
  }

  .btn-launch {
    background: #1a3a1a;
    border: 1px solid #2a6a2a;
    border-radius: 7px;
    color: #70e070;
    padding: 0.45rem 1rem;
    font-size: 0.85rem;
    font-weight: 500;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 0.4rem;
    transition: background 0.15s, border-color 0.15s;
    min-width: 100px;
    justify-content: center;
  }

  .btn-launch:hover:not(:disabled) {
    background: #204a20;
    border-color: #40a040;
  }

  .btn-launch:disabled {
    opacity: 0.7;
    cursor: not-allowed;
  }

  .btn-icon {
    background: none;
    border: 1px solid transparent;
    border-radius: 6px;
    font-size: 1rem;
    cursor: pointer;
    padding: 0.4rem;
    transition: background 0.15s, border-color 0.15s;
    line-height: 1;
  }

  .btn-edit {
    color: #60a0e0;
  }

  .btn-edit:hover {
    background: #102a40;
    border-color: #3060a0;
    color: #a0d0ff;
  }

  .btn-delete {
    color: #804040;
    min-width: 32px;
  }

  .btn-delete:hover {
    background: #2a1010;
    border-color: #803030;
    color: #e06060;
  }

  .btn-delete.confirming {
    background: #803030;
    color: #ffffff;
    font-size: 0.8rem;
    font-weight: 500;
    padding: 0.4rem 0.6rem;
  }
</style>
