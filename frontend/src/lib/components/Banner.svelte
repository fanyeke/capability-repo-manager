<script lang="ts">
  import { CheckCircle, AlertTriangle, AlertCircle, Info, X } from 'lucide-svelte';

  let {
    type = 'info',
    dismissible = true,
    visible = true,
    onDismiss,
    class: className = '',
    children,
  }: {
    type?: 'success' | 'warning' | 'error' | 'info';
    dismissible?: boolean;
    visible?: boolean;
    onDismiss?: () => void;
    class?: string;
    children?: import('svelte').Snippet;
  } = $props();

  let dismissed = $state(false);
  const isVisible = $derived(visible && !dismissed);

  const iconMap = {
    success: CheckCircle,
    warning: AlertTriangle,
    error: AlertCircle,
    info: Info,
  };
</script>

{#if isVisible}
  <div class="banner banner-{type} {className}">
    <svelte:component this={iconMap[type]} size={16} class="banner-icon" />
    <span class="banner-content">{#if children}{@render children()}{/if}</span>
    {#if dismissible}
      <button
        class="banner-close"
        onclick={() => { dismissed = true; onDismiss?.(); }}
        aria-label="Dismiss"
      >
        <X size={14} />
      </button>
    {/if}
  </div>
{/if}

<style>
  .banner {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-md);
    font-size: var(--font-size-sm);
    line-height: 1.4;
  }
  .banner-icon { flex-shrink: 0; }

  .banner-success {
    background: var(--color-success-bg);
    color: var(--color-success);
  }
  .banner-warning {
    background: var(--color-warning-bg);
    color: var(--color-warning);
  }
  .banner-error {
    background: var(--color-danger-bg);
    color: var(--color-danger);
  }
  .banner-info {
    background: var(--color-info-bg);
    color: var(--color-info);
  }

  .banner-content { flex: 1; }
  .banner-close {
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    cursor: pointer;
    padding: 2px;
    border-radius: var(--radius-sm);
    opacity: 0.7;
    flex-shrink: 0;
    color: inherit;
  }
  .banner-close:hover { opacity: 1; }
</style>
