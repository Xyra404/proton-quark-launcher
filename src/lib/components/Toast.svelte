<script lang="ts">
  interface Props {
    message: string;
    ondismiss: () => void;
  }

  let { message, ondismiss }: Props = $props();
</script>

<div class="toast" role="alert" aria-live="assertive">
  <span class="toast-icon">⚠</span>
  <p class="toast-message">{message}</p>
  <button class="toast-close" onclick={ondismiss} aria-label="Dismiss error">✕</button>
</div>

<style>
  .toast {
    position: fixed;
    bottom: 1.5rem;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: 0.75rem;
    background: #2a1a1a;
    border: 1px solid #c0392b;
    border-left: 4px solid #e74c3c;
    border-radius: 8px;
    padding: 0.85rem 1.1rem;
    max-width: min(520px, 90vw);
    width: max-content;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.6);
    z-index: 1000;
    animation: slide-up 0.2s ease-out;
  }

  @keyframes slide-up {
    from { opacity: 0; transform: translateX(-50%) translateY(12px); }
    to   { opacity: 1; transform: translateX(-50%) translateY(0); }
  }

  .toast-icon {
    font-size: 1.15rem;
    color: #e74c3c;
    flex-shrink: 0;
  }

  .toast-message {
    margin: 0;
    font-size: 0.9rem;
    color: #f0c0c0;
    line-height: 1.4;
    flex: 1;
  }

  .toast-close {
    background: none;
    border: none;
    color: #a08080;
    font-size: 0.9rem;
    cursor: pointer;
    padding: 0.2rem 0.3rem;
    border-radius: 4px;
    flex-shrink: 0;
    transition: color 0.15s;
  }

  .toast-close:hover {
    color: #f0c0c0;
  }
</style>
