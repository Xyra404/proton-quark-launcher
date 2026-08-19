<script lang="ts">
  import CustomProtonPaths from './CustomProtonPaths.svelte';

  interface Props {
    open: boolean;
    onclose: () => void;
  }

  let { open = $bindable(), onclose }: Props = $props();

  function handleBackdropClick(e: MouseEvent) {
    if ((e.target as HTMLElement).classList.contains('drawer-backdrop')) {
      onclose();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') onclose();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="drawer-backdrop" onclick={handleBackdropClick}>
    <div class="drawer-panel" role="dialog" aria-modal="true" aria-label="Settings">
      <div class="drawer-header">
        <div class="drawer-title">
          <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <circle cx="12" cy="12" r="3"></circle>
            <path d="M19.07 4.93a10 10 0 0 1 0 14.14"></path>
            <path d="M4.93 4.93a10 10 0 0 0 0 14.14"></path>
            <path d="M12 2v2"></path>
            <path d="M12 20v2"></path>
            <path d="M2 12h2"></path>
            <path d="M20 12h2"></path>
          </svg>
          Settings
        </div>
        <button class="close-btn" onclick={onclose} aria-label="Close settings">
          <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      </div>

      <div class="drawer-body">
        <section class="settings-section">
          <h3 class="settings-section-heading">Proton Runtimes</h3>
          <p class="settings-section-desc">
            Manage how Proton installations are discovered. Custom folders are merged with
            auto-detected Steam and system paths when you select a Proton version.
          </p>
          <CustomProtonPaths />
        </section>
      </div>
    </div>
  </div>
{/if}

<style>
  /* ── Backdrop ─────────────────────────────────────────────────────────────── */
  .drawer-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.55);
    backdrop-filter: blur(2px);
    z-index: 200;
    animation: backdrop-in 0.2s ease-out;
  }

  @keyframes backdrop-in {
    from { opacity: 0; }
    to   { opacity: 1; }
  }

  /* ── Panel ────────────────────────────────────────────────────────────────── */
  .drawer-panel {
    position: absolute;
    top: 0;
    right: 0;
    bottom: 0;
    width: min(420px, 92vw);
    background: #12122a;
    border-left: 1px solid #252550;
    display: flex;
    flex-direction: column;
    box-shadow: -12px 0 48px rgba(0, 0, 0, 0.7);
    animation: drawer-in 0.22s cubic-bezier(0.22, 1, 0.36, 1);
  }

  @keyframes drawer-in {
    from { transform: translateX(100%); opacity: 0.6; }
    to   { transform: translateX(0);    opacity: 1;   }
  }

  /* ── Header ───────────────────────────────────────────────────────────────── */
  .drawer-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1.1rem 1.4rem;
    border-bottom: 1px solid #1e1e42;
    flex-shrink: 0;
  }

  .drawer-title {
    display: flex;
    align-items: center;
    gap: 0.55rem;
    font-size: 1.05rem;
    font-weight: 700;
    color: #d0d0ff;
    letter-spacing: 0.01em;
  }

  .drawer-title svg {
    color: #8080e0;
  }

  .close-btn {
    background: none;
    border: 1px solid transparent;
    border-radius: 7px;
    color: #5050a0;
    cursor: pointer;
    padding: 0.35rem;
    display: flex;
    align-items: center;
    transition: color 0.15s, border-color 0.15s, background 0.15s;
  }

  .close-btn:hover {
    color: #c0c0ff;
    border-color: #2a2a60;
    background: #1a1a38;
  }

  /* ── Body ─────────────────────────────────────────────────────────────────── */
  .drawer-body {
    flex: 1;
    overflow-y: auto;
    padding: 1.4rem;
    display: flex;
    flex-direction: column;
    gap: 1.75rem;
  }

  /* ── Section ──────────────────────────────────────────────────────────────── */
  .settings-section {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .settings-section-heading {
    margin: 0;
    font-size: 0.72rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: #5050a0;
    font-weight: 700;
    padding-bottom: 0.6rem;
    border-bottom: 1px solid #1a1a38;
  }

  .settings-section-desc {
    margin: 0;
    font-size: 0.82rem;
    color: #6060a0;
    line-height: 1.45;
  }
</style>
