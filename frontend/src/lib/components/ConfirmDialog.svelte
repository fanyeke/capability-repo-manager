<script lang="ts">
  import { _ } from 'svelte-i18n';
  import { AlertTriangle, X } from 'lucide-svelte';
  import Button from './Button.svelte';

  let {
    open = false,
    title = 'Confirm',
    message,
    confirmLabel = 'Confirm',
    variant = 'danger',
    loading = false,
    onConfirm,
    onCancel,
  }: {
    open?: boolean;
    title?: string;
    message: string;
    confirmLabel?: string;
    variant?: 'danger' | 'primary';
    loading?: boolean;
    onConfirm: () => void;
    onCancel: () => void;
  } = $props();
</script>

{#if open}
  <div class="dialog-overlay" onclick={onCancel} role="presentation">
    <div class="dialog-panel" onclick={(e) => e.stopPropagation()} role="dialog" aria-modal="true" aria-label={title}>
      <div class="dialog-header">
        <div class="dialog-title-row">
          <AlertTriangle size={18} class="dialog-icon dialog-icon-{variant}" />
          <h3 class="dialog-title">{title}</h3>
        </div>
        <button class="dialog-close" onclick={onCancel} aria-label="Close">
          <X size={16} />
        </button>
      </div>
      <p class="dialog-message">{message}</p>
      <div class="dialog-actions">
        <Button variant="ghost" onclick={onCancel}>{$_('common.cancel')}</Button>
        <Button variant={variant} onclick={onConfirm} loading={loading}>
          {confirmLabel}
        </Button>
      </div>
    </div>
  </div>
{/if}

<style>
  .dialog-overlay {
    position: fixed;
    inset: 0;
    z-index: 9999;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(2px);
  }
  .dialog-panel {
    background: var(--bg-panel);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-lg);
    padding: var(--space-6);
    max-width: 420px;
    width: 90%;
    box-shadow: var(--shadow-lg);
  }
  .dialog-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    margin-bottom: var(--space-3);
  }
  .dialog-title-row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }
  .dialog-icon {
    flex-shrink: 0;
  }
  .dialog-icon-danger { color: var(--color-danger); }
  .dialog-icon-primary { color: var(--color-primary); }
  .dialog-title {
    margin: 0;
    font-size: var(--font-size-lg);
    font-weight: 600;
  }
  .dialog-close {
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 4px;
    border-radius: var(--radius-sm);
  }
  .dialog-close:hover {
    color: var(--text-secondary);
    background: var(--bg-hover);
  }
  .dialog-message {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    line-height: 1.5;
    margin: 0 0 var(--space-5);
  }
  .dialog-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-2);
  }
</style>
