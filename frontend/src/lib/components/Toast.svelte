<script lang="ts">
  import { toasts, dismissToast } from '$lib/stores/toastStore';
  import { X, CheckCircle, AlertTriangle, AlertCircle, Info } from 'lucide-svelte';

  const iconMap = {
    success: CheckCircle,
    error: AlertCircle,
    warning: AlertTriangle,
    info: Info,
  };
</script>

{#if $toasts.length > 0}
  <div class="toast-container">
    {#each $toasts as toast (toast.id)}
      <div class="toast toast-{toast.type}" role="alert">
        <svelte:component this={iconMap[toast.type]} size={16} class="toast-icon" />
        <span class="toast-message">{toast.message}</span>
        <button class="toast-close" onclick={() => dismissToast(toast.id)} aria-label="Close">
          <X size={14} />
        </button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .toast-container {
    position: fixed;
    bottom: var(--space-6);
    right: var(--space-6);
    z-index: 9999;
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    max-width: 400px;
  }
  .toast {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-md);
    background: var(--bg-elevated);
    border: 1px solid var(--border-default);
    box-shadow: var(--shadow-md);
    font-size: var(--font-size-sm);
    color: var(--text-primary);
    animation: toast-in 200ms ease;
  }
  @keyframes toast-in {
    from { opacity: 0; transform: translateY(8px); }
    to { opacity: 1; transform: translateY(0); }
  }
  .toast-icon { flex-shrink: 0; }
  .toast-success .toast-icon { color: var(--color-success); }
  .toast-error .toast-icon { color: var(--color-danger); }
  .toast-warning .toast-icon { color: var(--color-warning); }
  .toast-info .toast-icon { color: var(--color-info); }
  .toast-success { border-left: 3px solid var(--color-success); }
  .toast-error { border-left: 3px solid var(--color-danger); }
  .toast-warning { border-left: 3px solid var(--color-warning); }
  .toast-info { border-left: 3px solid var(--color-info); }

  .toast-message { flex: 1; }
  .toast-close {
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 2px;
    border-radius: var(--radius-sm);
    flex-shrink: 0;
  }
  .toast-close:hover {
    color: var(--text-secondary);
    background: var(--bg-hover);
  }
</style>
