<script lang="ts">
  let {
    variant = 'default',
    size = 'sm',
    class: className = '',
    children,
  }: {
    variant?: 'default' | 'success' | 'warning' | 'danger' | 'info';
    size?: 'sm' | 'md';
    class?: string;
    children?: import('svelte').Snippet;
  } = $props();
</script>

<span class="badge badge-{variant} badge-{size} {className}">
  {#if children}
    {@render children()}
  {/if}
</span>

<style>
  .badge {
    display: inline-flex;
    align-items: center;
    font-weight: 500;
    white-space: nowrap;
    border-radius: var(--radius-full);
    line-height: 1;
  }
  .badge-sm {
    padding: 2px 8px;
    font-size: var(--font-size-xs);
  }
  .badge-md {
    padding: 3px 10px;
    font-size: var(--font-size-sm);
  }

  .badge-default {
    background: var(--bg-elevated);
    color: var(--text-secondary);
  }
  .badge-success {
    background: var(--color-success-bg);
    color: var(--color-success);
  }
  .badge-warning {
    background: var(--color-warning-bg);
    color: var(--color-warning);
  }
  .badge-danger {
    background: var(--color-danger-bg);
    color: var(--color-danger);
  }
  .badge-info {
    background: var(--color-info-bg);
    color: var(--color-info);
  }
</style>
