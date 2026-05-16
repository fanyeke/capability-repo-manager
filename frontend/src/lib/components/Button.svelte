<script lang="ts">
  import { LoaderCircle } from 'lucide-svelte';

  let {
    variant = 'primary',
    size = 'md',
    disabled = false,
    loading = false,
    icon,
    class: className = '',
    onclick,
    children,
  }: {
    variant?: 'primary' | 'secondary' | 'ghost' | 'danger';
    size?: 'sm' | 'md' | 'lg';
    disabled?: boolean;
    loading?: boolean;
    icon?: any;
    class?: string;
    onclick?: (e: MouseEvent) => void;
    children?: import('svelte').Snippet;
  } = $props();
</script>

<button
  class="btn btn-{variant} btn-{size} {className}"
  {disabled}
  {onclick}
>
  {#if loading}
    <LoaderCircle class="spin" size={size === 'sm' ? 12 : size === 'lg' ? 18 : 14} />
  {:else if icon}
    <svelte:component this={icon} size={size === 'sm' ? 12 : size === 'lg' ? 18 : 14} />
  {/if}
  {#if children}
    <span class="btn-text">{@render children()}</span>
  {/if}
</button>

<style>
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    border: 1px solid transparent;
    border-radius: var(--radius-md);
    font-family: var(--font-sans);
    font-weight: 500;
    cursor: pointer;
    transition: background var(--transition-fast), color var(--transition-fast), border-color var(--transition-fast), box-shadow var(--transition-fast);
    white-space: nowrap;
    line-height: 1;
  }
  .btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .btn:focus-visible {
    outline: none;
    box-shadow: var(--shadow-glow);
  }
  .spin {
    animation: spin 1s linear infinite;
  }
  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  /* Sizes */
  .btn-sm { padding: 4px 10px; font-size: var(--font-size-xs); }
  .btn-md { padding: 6px 14px; font-size: var(--font-size-sm); }
  .btn-lg { padding: 8px 20px; font-size: var(--font-size-md); }

  /* Variants */
  .btn-primary {
    background: var(--color-primary);
    color: #fff;
  }
  .btn-primary:hover:not(:disabled) {
    background: var(--color-primary-hover);
  }

  .btn-secondary {
    background: var(--bg-card);
    color: var(--text-secondary);
    border-color: var(--border-default);
  }
  .btn-secondary:hover:not(:disabled) {
    background: var(--bg-hover);
    color: var(--text-primary);
    border-color: var(--border-strong);
  }

  .btn-ghost {
    background: none;
    color: var(--text-secondary);
  }
  .btn-ghost:hover:not(:disabled) {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .btn-danger {
    background: var(--color-danger);
    color: #fff;
  }
  .btn-danger:hover:not(:disabled) {
    opacity: 0.9;
  }
</style>
