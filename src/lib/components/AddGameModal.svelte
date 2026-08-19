<script lang="ts">
  import { open as openDialog } from '@tauri-apps/plugin-dialog';
  import { listProtonVersions, addGame, updateGame } from '$lib/api';
  import type { ProtonInstall, Game, GamePlatform } from '$lib/types';

  interface Props {
    open: boolean;
    existingGame?: Game;
    onclose: () => void;
    ongameadded: () => void;
  }

  let { open = $bindable(), existingGame, onclose, ongameadded }: Props = $props();

  // ── Form state ──────────────────────────────────────────────────────────────
  let platform = $state<GamePlatform>('Windows');
  let name = $state('');
  let exePath = $state('');
  let selectedProtonIndex = $state(-1);
  let prefixPath = $state('');
  let launchArgs = $state('');

  // ── Async state ─────────────────────────────────────────────────────────────
  let protonVersions = $state<ProtonInstall[]>([]);
  let loadingProton = $state(false);
  let submitting = $state(false);
  let errorMsg = $state('');

  // ── Derived validation ───────────────────────────────────────────────────────
  let nameError = $derived(name.trim() === '' ? 'Name is required.' : '');
  let exeError = $derived(
    exePath === ''
      ? 'Please pick an executable.'
      : (platform === 'Windows' && !exePath.toLowerCase().endsWith('.exe'))
      ? 'Windows executable must end in .exe.'
      : ''
  );
  let protonError = $derived(selectedProtonIndex < 0 ? 'Select a Proton version.' : '');

  let canSubmit = $derived(
    nameError === '' && exeError === '' && (platform === 'Linux' || protonError === '') && !submitting
  );

  let selectedProton = $derived<ProtonInstall | undefined>(
    protonVersions[selectedProtonIndex]
  );

  // ── Load Proton versions whenever modal opens ────────────────────────────────
  $effect(() => {
    if (!open) return;
    loadingProton = true;
    errorMsg = '';
    listProtonVersions()
      .then((versions) => {
        protonVersions = versions;
        if (existingGame) {
          selectedProtonIndex = versions.findIndex(v => v.path === existingGame.proton_path);
          if (selectedProtonIndex < 0 && versions.length > 0) selectedProtonIndex = 0;
        } else {
          selectedProtonIndex = versions.length > 0 ? 0 : -1;
        }
      })
      .catch((e: Error) => {
        errorMsg = e.message;
      })
      .finally(() => {
        loadingProton = false;
      });
  });

  // ── Reset form whenever modal is closed or opened ────────────────────────────
  $effect(() => {
    if (open) {
      if (existingGame) {
        platform = existingGame.platform || 'Windows';
        name = existingGame.name;
        exePath = existingGame.exe_path;
        prefixPath = existingGame.prefix_path || '';
        launchArgs = existingGame.launch_args || '';
      }
    } else {
      platform = 'Windows';
      name = '';
      exePath = '';
      selectedProtonIndex = protonVersions.length > 0 ? 0 : -1;
      prefixPath = '';
      launchArgs = '';
      errorMsg = '';
      submitting = false;
    }
  });

  // ── File picker ──────────────────────────────────────────────────────────────
  async function pickExe() {
    const title = platform === 'Windows' ? 'Select Windows Executable' : 'Select Linux Executable';
    const filters = platform === 'Windows' ? [{ name: 'Windows Executable', extensions: ['exe'] }] : [];
    
    const result = await openDialog({
      title,
      multiple: false,
      filters,
    });
    if (result && typeof result === 'string') {
      exePath = result;
    }
  }

  // ── Submit ───────────────────────────────────────────────────────────────────
  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (!canSubmit || !selectedProton) return;

    submitting = true;
    errorMsg = '';

    try {
      const gameData: Game = {
        id: existingGame ? existingGame.id : crypto.randomUUID(),
        name: name.trim(),
        platform,
        exe_path: exePath,
        proton_version: platform === 'Windows' ? selectedProton.name : undefined,
        proton_path: platform === 'Windows' ? selectedProton.path : undefined,
        prefix_path: platform === 'Windows' ? (prefixPath.trim() || undefined) : undefined,
        launch_args: launchArgs.trim() || undefined,
        last_played: existingGame ? existingGame.last_played : undefined,
      };

      if (existingGame) {
        await updateGame(gameData);
      } else {
        await addGame(gameData);
      }
      ongameadded();
      onclose();
    } catch (err: unknown) {
      errorMsg = err instanceof Error ? err.message : String(err);
    } finally {
      submitting = false;
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if ((e.target as HTMLElement).classList.contains('modal-backdrop')) {
      onclose();
    }
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="modal-backdrop" onclick={handleBackdropClick}>
    <div class="modal-panel" role="dialog" aria-modal="true" aria-label="{existingGame ? 'Edit Game' : 'Add Game'}">
      <header class="modal-header">
        <h2>{existingGame ? 'Edit Game' : 'Add Game'}</h2>
        <button class="close-btn" onclick={onclose} aria-label="Close">✕</button>
      </header>

      <form onsubmit={handleSubmit} novalidate>

        <!-- Platform Toggle -->
        <div class="field">
          <span class="group-label">Platform</span>
          <div class="segmented-control">
            <button
              type="button"
              class="segment {platform === 'Windows' ? 'active' : ''}"
              onclick={() => platform = 'Windows'}
            >
              Windows (Proton)
            </button>
            <button
              type="button"
              class="segment {platform === 'Linux' ? 'active' : ''}"
              onclick={() => platform = 'Linux'}
            >
              Linux Native
            </button>
          </div>
        </div>

        <!-- Name -->
        <div class="field">
          <label for="game-name">Game Name</label>
          <input
            id="game-name"
            type="text"
            bind:value={name}
            placeholder="e.g. Half-Life 2"
            autocomplete="off"
          />
          {#if nameError}<span class="field-error">{nameError}</span>{/if}
        </div>

        <!-- Executable -->
        <div class="field">
          <label for="exe-path">Executable</label>
          <div class="input-row">
            <input
              id="exe-path"
              type="text"
              bind:value={exePath}
              placeholder={platform === 'Windows' ? "/mnt/games/game.exe" : "/mnt/games/game_bin"}
              readonly
            />
            <button type="button" class="browse-btn" onclick={pickExe}>Browse…</button>
          </div>
          {#if exeError && exePath !== ''}<span class="field-error">{exeError}</span>{/if}
        </div>

        {#if platform === 'Windows'}
          <!-- Proton Version -->
          <div class="field">
            <label for="proton-select">Proton Version</label>
            {#if loadingProton}
              <p class="loading-text">Scanning for Proton installations…</p>
            {:else if protonVersions.length === 0}
              <p class="no-proton">No Proton installations found. Install GE-Proton or Steam Proton first.</p>
            {:else}
              <select id="proton-select" bind:value={selectedProtonIndex}>
                {#each protonVersions as version, i}
                  <option value={i}>{version.name}</option>
                {/each}
              </select>
              {#if selectedProton}
                <span class="path-hint" title={selectedProton.path}>{selectedProton.path}</span>
              {/if}
            {/if}
            {#if protonError && !loadingProton}<span class="field-error">{protonError}</span>{/if}
          </div>
        {/if}

        <!-- Advanced -->
        <details class="advanced">
          <summary>Advanced options</summary>
          {#if platform === 'Windows'}
            <div class="field">
              <label for="prefix-path">Wine Prefix (optional)</label>
              <input
                id="prefix-path"
                type="text"
                bind:value={prefixPath}
                placeholder="Leave blank to use auto-managed prefix"
              />
            </div>
          {/if}
          <div class="field">
            <label for="launch-args">Extra Launch Arguments (optional)</label>
            <input
              id="launch-args"
              type="text"
              bind:value={launchArgs}
              placeholder="-windowed -nosplash"
            />
          </div>
        </details>

        <!-- Error -->
        {#if errorMsg}
          <p class="submit-error">{errorMsg}</p>
        {/if}

        <!-- Actions -->
        <div class="modal-actions">
          <button type="button" class="btn-secondary" onclick={onclose}>Cancel</button>
          <button type="submit" class="btn-primary" disabled={!canSubmit}>
            {#if submitting}
              {existingGame ? 'Saving…' : 'Adding…'}
            {:else}
              {existingGame ? 'Save Changes' : 'Add Game'}
            {/if}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.72);
    display: flex;
    padding: 1rem;
    z-index: 100;
    backdrop-filter: blur(3px);
    animation: fade-in 0.15s ease-out;
  }

  @keyframes fade-in {
    from { opacity: 0; }
    to   { opacity: 1; }
  }

  .modal-panel {
    margin: auto;
    background: #1a1a2e;
    border: 1px solid #2a2a4a;
    border-radius: 12px;
    width: min(540px, 92vw);
    max-height: 90vh;
    overflow-y: auto;
    box-shadow: 0 24px 64px rgba(0, 0, 0, 0.8);
    animation: slide-in 0.18s ease-out;
  }

  @keyframes slide-in {
    from { opacity: 0; transform: translateY(-16px) scale(0.98); }
    to   { opacity: 1; transform: translateY(0) scale(1); }
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1.25rem 1.5rem 0;
    border-bottom: 1px solid #2a2a4a;
    padding-bottom: 1rem;
  }

  .modal-header h2 {
    margin: 0;
    font-size: 1.2rem;
    color: #e0e0ff;
    font-weight: 600;
  }

  .close-btn {
    background: none;
    border: none;
    color: #6060a0;
    font-size: 1.1rem;
    cursor: pointer;
    padding: 0.25rem 0.4rem;
    border-radius: 4px;
    transition: color 0.15s;
  }

  .close-btn:hover { color: #c0c0ff; }

  form {
    padding: 1.25rem 1.5rem 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  /* Segmented control for Platform */
  .segmented-control {
    display: flex;
    background: #111128;
    border: 1px solid #2a2a4a;
    border-radius: 6px;
    overflow: hidden;
  }

  .segment {
    flex: 1;
    background: transparent;
    border: none;
    color: #9090cc;
    padding: 0.5rem;
    font-size: 0.85rem;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.15s, color 0.15s;
  }

  .segment:hover:not(.active) {
    background: #181838;
    color: #c0c0ff;
  }

  .segment.active {
    background: #303080;
    color: #ffffff;
  }

  label, .group-label {
    font-size: 0.82rem;
    font-weight: 500;
    color: #8888cc;
    letter-spacing: 0.03em;
    text-transform: uppercase;
  }

  input[type="text"],
  select {
    color-scheme: dark;
    background: #111128;
    border: 1px solid #2a2a4a;
    border-radius: 6px;
    color: #e0e0ff;
    padding: 0.55rem 0.75rem;
    font-size: 0.9rem;
    outline: none;
    transition: border-color 0.15s;
    width: 100%;
    box-sizing: border-box;
  }

  select {
    cursor: pointer;
    appearance: none;
    -webkit-appearance: none;
    -moz-appearance: none;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%238888cc' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpolyline points='6 9 12 15 18 9'%3E%3C/polyline%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 0.85rem center;
    padding-right: 2.2rem;
  }

  select option {
    color-scheme: dark;
    background-color: #181830;
    color: #e0e0ff;
    padding: 0.5rem;
  }

  input[type="text"]:focus,
  select:focus {
    border-color: #6060e0;
  }

  input[readonly] {
    color: #9090cc;
    cursor: default;
  }

  .input-row {
    display: flex;
    gap: 0.5rem;
  }

  .input-row input {
    flex: 1;
    min-width: 0;
  }

  .browse-btn {
    background: #252550;
    border: 1px solid #3030a0;
    border-radius: 6px;
    color: #c0c0ff;
    padding: 0.55rem 0.9rem;
    font-size: 0.88rem;
    cursor: pointer;
    white-space: nowrap;
    transition: background 0.15s, border-color 0.15s;
  }

  .browse-btn:hover {
    background: #303080;
    border-color: #5050c0;
  }

  .path-hint {
    font-size: 0.75rem;
    color: #5050a0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .loading-text,
  .no-proton {
    font-size: 0.85rem;
    color: #6060a0;
    margin: 0;
    font-style: italic;
  }

  .field-error {
    font-size: 0.78rem;
    color: #e06060;
  }

  .submit-error {
    font-size: 0.85rem;
    color: #e07070;
    background: #2a1515;
    border: 1px solid #5a2020;
    border-radius: 6px;
    padding: 0.6rem 0.8rem;
    margin: 0;
  }

  .advanced {
    border: 1px solid #2a2a4a;
    border-radius: 8px;
    padding: 0.75rem 1rem;
  }

  .advanced summary {
    font-size: 0.85rem;
    color: #7070b0;
    cursor: pointer;
    user-select: none;
    list-style: none;
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .advanced summary::before {
    content: '▶';
    font-size: 0.7rem;
    transition: transform 0.15s;
  }

  .advanced[open] summary::before {
    transform: rotate(90deg);
  }

  .advanced .field {
    margin-top: 0.75rem;
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.6rem;
    padding-top: 0.5rem;
    border-top: 1px solid #2a2a4a;
    margin-top: 0.5rem;
  }

  .btn-primary,
  .btn-secondary {
    border-radius: 7px;
    padding: 0.55rem 1.3rem;
    font-size: 0.9rem;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.15s, border-color 0.15s, opacity 0.15s;
  }

  .btn-primary {
    background: #4040c0;
    border: 1px solid #6060e0;
    color: #ffffff;
  }

  .btn-primary:hover:not(:disabled) {
    background: #5555d5;
  }

  .btn-primary:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  .btn-secondary {
    background: #1a1a30;
    border: 1px solid #2a2a4a;
    color: #9090c0;
  }

  .btn-secondary:hover {
    background: #252545;
    color: #c0c0ff;
  }
</style>
