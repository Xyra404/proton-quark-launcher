<script lang="ts">
  import { listen } from '@tauri-apps/api/event';
  import {
    listProtonVersions,
    listAvailableProtonDownloads,
    downloadProtonVersion,
    deleteProtonVersion,
  } from '$lib/api';
  import type { ProtonInstall, ProtonRelease } from '$lib/types';

  // ── State ─────────────────────────────────────────────────────────────────
  let installed = $state<ProtonInstall[]>([]);
  let available = $state<ProtonRelease[]>([]);

  let loadingInstalled = $state(false);
  let loadingAvailable = $state(false);
  let installedError = $state('');
  let availableError = $state('');

  // tag → pending delete confirmation
  let pendingDeletePaths = $state<Set<string>>(new Set());
  let deleteTimers = new Map<string, ReturnType<typeof setTimeout>>();

  // tag → { bytesDownloaded, totalBytes }
  interface DownloadProgress {
    bytesDownloaded: number;
    totalBytes?: number;
  }
  let downloads = $state<Map<string, DownloadProgress>>(new Map());
  let downloadErrors = $state<Map<string, string>>(new Map());

  // ── Known compat-tool dirs (for distinguishing deletable vs. system installs)
  const COMPAT_MARKERS = [
    'compatibilitytools.d',
  ];

  function isDeletable(install: ProtonInstall): boolean {
    return COMPAT_MARKERS.some((m) => install.path.includes(m));
  }

  // ── Format helpers ─────────────────────────────────────────────────────────
  function fmtSize(bytes?: number): string {
    if (!bytes) return '';
    if (bytes >= 1_073_741_824) return `${(bytes / 1_073_741_824).toFixed(1)} GB`;
    if (bytes >= 1_048_576) return `${(bytes / 1_048_576).toFixed(0)} MB`;
    return `${(bytes / 1024).toFixed(0)} KB`;
  }

  function fmtDate(iso: string): string {
    try {
      return new Intl.DateTimeFormat(undefined, { dateStyle: 'medium' }).format(new Date(iso));
    } catch {
      return iso.slice(0, 10);
    }
  }

  function pct(p: DownloadProgress): number {
    if (!p.totalBytes) return 0;
    return Math.min(100, (p.bytesDownloaded / p.totalBytes) * 100);
  }

  // ── Data loading ──────────────────────────────────────────────────────────
  async function fetchInstalled() {
    loadingInstalled = true;
    installedError = '';
    try {
      installed = await listProtonVersions();
    } catch (e: unknown) {
      installedError = e instanceof Error ? e.message : String(e);
    } finally {
      loadingInstalled = false;
    }
  }

  async function fetchAvailable() {
    loadingAvailable = true;
    availableError = '';
    try {
      available = await listAvailableProtonDownloads();
    } catch (e: unknown) {
      availableError = e instanceof Error ? e.message : String(e);
    } finally {
      loadingAvailable = false;
    }
  }

  // ── Tauri event listeners (set up on mount, torn down on destroy) ──────────
  $effect(() => {
    fetchInstalled();
    fetchAvailable();

    const unlistenProgress = listen<{ tag: string; bytes_downloaded: number; total_bytes?: number }>(
      'proton-download-progress',
      ({ payload }) => {
        const next = new Map(downloads);
        next.set(payload.tag, {
          bytesDownloaded: payload.bytes_downloaded,
          totalBytes: payload.total_bytes,
        });
        downloads = next;
      }
    );

    const unlistenComplete = listen<{ tag: string; success: boolean; error?: string }>(
      'proton-download-complete',
      ({ payload }) => {
        const next = new Map(downloads);
        next.delete(payload.tag);
        downloads = next;

        if (payload.success) {
          const errNext = new Map(downloadErrors);
          errNext.delete(payload.tag);
          downloadErrors = errNext;
          // Refresh installed list
          fetchInstalled();
        } else if (payload.error) {
          const errNext = new Map(downloadErrors);
          errNext.set(payload.tag, payload.error);
          downloadErrors = errNext;
        }
      }
    );

    return () => {
      unlistenProgress.then((u) => u());
      unlistenComplete.then((u) => u());
    };
  });

  // ── Install ────────────────────────────────────────────────────────────────
  async function handleInstall(release: ProtonRelease) {
    // Optimistically add to downloads map so the button disables immediately
    const next = new Map(downloads);
    next.set(release.tag, { bytesDownloaded: 0, totalBytes: release.size_bytes });
    downloads = next;

    // Clear prior error for this tag
    const errNext = new Map(downloadErrors);
    errNext.delete(release.tag);
    downloadErrors = errNext;

    try {
      await downloadProtonVersion(release);
      // Success handled by 'proton-download-complete' event
    } catch (e: unknown) {
      // If the command itself throws synchronously (shouldn't happen since
      // the Rust command emits events and returns Ok), still surface it.
      const errNext2 = new Map(downloadErrors);
      errNext2.set(release.tag, e instanceof Error ? e.message : String(e));
      downloadErrors = errNext2;

      const dNext = new Map(downloads);
      dNext.delete(release.tag);
      downloads = dNext;
    }
  }

  // ── Delete ─────────────────────────────────────────────────────────────────
  function handleDeleteClick(install: ProtonInstall) {
    const key = install.path;

    if (pendingDeletePaths.has(key)) {
      // Confirmed — do the delete
      clearTimeout(deleteTimers.get(key));
      deleteTimers.delete(key);
      pendingDeletePaths = new Set([...pendingDeletePaths].filter((p) => p !== key));

      deleteProtonVersion(key)
        .then(() => fetchInstalled())
        .catch((e: unknown) => {
          installedError = e instanceof Error ? e.message : String(e);
        });
    } else {
      // Ask for confirmation
      pendingDeletePaths = new Set([...pendingDeletePaths, key]);
      const timer = setTimeout(() => {
        pendingDeletePaths = new Set([...pendingDeletePaths].filter((p) => p !== key));
        deleteTimers.delete(key);
      }, 3000);
      deleteTimers.set(key, timer);
    }
  }

  // ── Derived: which tags are already installed ──────────────────────────────
  let installedNames = $derived(new Set(installed.map((i) => i.name)));
</script>

<div class="pm-root">

  <!-- ── Installed Versions ─────────────────────────────────────────────── -->
  <section class="pm-section">
    <div class="section-hd">
      <h4 class="section-title">Installed</h4>
      <button class="refresh-btn" onclick={fetchInstalled} title="Refresh installed list">
        <svg xmlns="http://www.w3.org/2000/svg" width="13" height="13" viewBox="0 0 24 24"
          fill="none" stroke="currentColor" stroke-width="2.5"
          stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <polyline points="23 4 23 10 17 10"></polyline>
          <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"></path>
        </svg>
      </button>
    </div>

    {#if loadingInstalled}
      <p class="state-msg">Scanning…</p>
    {:else if installedError}
      <p class="state-msg error">{installedError}</p>
    {:else if installed.length === 0}
      <p class="state-msg muted">No Proton installations found.</p>
    {:else}
      <ul class="item-list">
        {#each installed as inst (inst.path)}
          <li class="item-row">
            <div class="item-info">
              <span class="item-name">{inst.name}</span>
              <span class="item-path" title={inst.path}>{inst.path}</span>
            </div>
            {#if isDeletable(inst)}
              {@const isPending = pendingDeletePaths.has(inst.path)}
              <button
                class="action-btn del-btn"
                class:confirming={isPending}
                onclick={() => handleDeleteClick(inst)}
                title={isPending ? 'Confirm delete' : 'Remove this Proton version'}
              >
                {isPending ? 'Sure?' : '🗑'}
              </button>
            {:else}
              <span class="system-badge" title="System install — cannot delete">System</span>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}
  </section>

  <!-- ── Available Downloads ────────────────────────────────────────────── -->
  <section class="pm-section">
    <div class="section-hd">
      <h4 class="section-title">Available (GE-Proton)</h4>
      <button class="refresh-btn" onclick={fetchAvailable} title="Refresh from GitHub">
        <svg xmlns="http://www.w3.org/2000/svg" width="13" height="13" viewBox="0 0 24 24"
          fill="none" stroke="currentColor" stroke-width="2.5"
          stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <polyline points="23 4 23 10 17 10"></polyline>
          <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"></path>
        </svg>
      </button>
    </div>

    {#if loadingAvailable}
      <p class="state-msg">Fetching releases from GitHub…</p>
    {:else if availableError}
      <p class="state-msg error">{availableError}</p>
    {:else if available.length === 0}
      <p class="state-msg muted">No releases found.</p>
    {:else}
      <ul class="item-list">
        {#each available as rel (rel.tag)}
          {@const inProgress = downloads.has(rel.tag)}
          {@const progress = downloads.get(rel.tag)}
          {@const alreadyInstalled = installedNames.has(rel.tag)}
          {@const dlError = downloadErrors.get(rel.tag)}

          <li class="item-row release-row">
            <div class="item-info">
              <span class="item-name">{rel.tag}</span>
              <span class="item-meta">
                {fmtDate(rel.published_at)}
                {#if rel.size_bytes}
                  · {fmtSize(rel.size_bytes)}
                {/if}
              </span>

              {#if inProgress && progress}
                <!-- Progress bar -->
                <div class="progress-wrap">
                  <div class="progress-bar">
                    <div class="progress-fill" style="width: {pct(progress)}%"></div>
                  </div>
                  <span class="progress-label">
                    {pct(progress).toFixed(0)}%
                    {fmtSize(progress.bytesDownloaded)} / {fmtSize(progress.totalBytes)}
                  </span>
                </div>
              {/if}

              {#if dlError}
                <span class="dl-error">{dlError}</span>
              {/if}
            </div>

            {#if alreadyInstalled}
              <span class="installed-badge">Installed ✓</span>
            {:else if inProgress}
              <button class="action-btn install-btn" disabled aria-label="Downloading…">
                <span class="mini-spinner"></span>
              </button>
            {:else}
              <button
                class="action-btn install-btn"
                onclick={() => handleInstall(rel)}
              >
                ↓ Install
              </button>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}
  </section>

</div>

<style>
  .pm-root {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  /* ── Section ────────────────────────────────────────────────────────────── */
  .pm-section {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }

  .section-hd {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .section-title {
    margin: 0;
    font-size: 0.72rem;
    text-transform: uppercase;
    letter-spacing: 0.07em;
    color: #5050a0;
    font-weight: 700;
    flex: 1;
  }

  .refresh-btn {
    background: none;
    border: 1px solid transparent;
    border-radius: 5px;
    color: #4040a0;
    cursor: pointer;
    padding: 0.25rem;
    display: flex;
    align-items: center;
    transition: color 0.15s, border-color 0.15s, background 0.15s;
  }

  .refresh-btn:hover {
    color: #9090e0;
    border-color: #2a2a50;
    background: #1a1a38;
  }

  /* ── State messages ─────────────────────────────────────────────────────── */
  .state-msg {
    margin: 0;
    font-size: 0.82rem;
    color: #5050a0;
    font-style: italic;
  }

  .state-msg.error { color: #d06060; font-style: normal; }
  .state-msg.muted { color: #3a3a70; }

  /* ── Item list ──────────────────────────────────────────────────────────── */
  .item-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .item-row {
    display: flex;
    align-items: flex-start;
    gap: 0.6rem;
    background: #0e0e24;
    border: 1px solid #1a1a38;
    border-radius: 7px;
    padding: 0.55rem 0.75rem;
    transition: border-color 0.15s;
  }

  .item-row:hover {
    border-color: #2a2a50;
  }

  .item-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
  }

  .item-name {
    font-size: 0.88rem;
    font-weight: 600;
    color: #c0c0ff;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .item-path {
    font-size: 0.7rem;
    color: #3a3a70;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-family: 'Courier New', monospace;
  }

  .item-meta {
    font-size: 0.72rem;
    color: #4040a0;
    margin-top: 0.05rem;
  }

  /* ── Progress bar ───────────────────────────────────────────────────────── */
  .progress-wrap {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    margin-top: 0.35rem;
  }

  .progress-bar {
    height: 4px;
    background: #1a1a38;
    border-radius: 2px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, #4040c0, #8040e0);
    border-radius: 2px;
    transition: width 0.3s ease;
  }

  .progress-label {
    font-size: 0.68rem;
    color: #6060a0;
    font-variant-numeric: tabular-nums;
  }

  /* ── Errors ─────────────────────────────────────────────────────────────── */
  .dl-error {
    font-size: 0.72rem;
    color: #e06060;
    margin-top: 0.25rem;
    line-height: 1.35;
  }

  /* ── Badges ─────────────────────────────────────────────────────────────── */
  .installed-badge {
    font-size: 0.7rem;
    font-weight: 600;
    color: #40a040;
    background: #0d200d;
    border: 1px solid #205020;
    border-radius: 10px;
    padding: 0.15rem 0.5rem;
    white-space: nowrap;
    align-self: center;
    flex-shrink: 0;
  }

  .system-badge {
    font-size: 0.68rem;
    color: #5050a0;
    background: #0e0e20;
    border: 1px solid #1e1e48;
    border-radius: 10px;
    padding: 0.15rem 0.5rem;
    white-space: nowrap;
    align-self: center;
    flex-shrink: 0;
  }

  /* ── Action buttons ─────────────────────────────────────────────────────── */
  .action-btn {
    border-radius: 6px;
    font-size: 0.78rem;
    font-weight: 500;
    cursor: pointer;
    padding: 0.3rem 0.65rem;
    white-space: nowrap;
    flex-shrink: 0;
    align-self: center;
    transition: background 0.15s, border-color 0.15s, opacity 0.15s;
    display: flex;
    align-items: center;
    gap: 0.3rem;
  }

  .install-btn {
    background: #1a2a1a;
    border: 1px solid #2a6a2a;
    color: #60c060;
  }

  .install-btn:hover:not(:disabled) {
    background: #1f381f;
    border-color: #40a040;
  }

  .install-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .del-btn {
    background: none;
    border: 1px solid transparent;
    color: #704040;
    min-width: 2rem;
    justify-content: center;
  }

  .del-btn:hover {
    background: #2a1010;
    border-color: #803030;
    color: #e06060;
  }

  .del-btn.confirming {
    background: #803030;
    border-color: #a04040;
    color: #fff;
    font-size: 0.72rem;
  }

  /* ── Mini spinner ─────────────────────────────────────────────────────── */
  .mini-spinner {
    width: 12px;
    height: 12px;
    border: 2px solid transparent;
    border-top-color: #60c060;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
