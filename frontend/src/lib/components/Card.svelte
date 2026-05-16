<script lang="ts">
  let {
    padding = 'md',
    hoverable = false,
    class: className = '',
    title,
    actions,
    children,
    onclick,
  }: {
    padding?: 'none' | 'sm' | 'md' | 'lg';
    hoverable?: boolean;
    class?: string;
    title?: import('svelte').Snippet;
    actions?: import('svelte').Snippet;
    children?: import('svelte').Snippet;
    onclick?: (e: MouseEvent) => void;
  } = $props();
</script>

<div class="card card-pad-{padding} {hoverable ? 'card-hoverable' : ''} {className}" role={onclick ? 'button' : undefined} tabindex={onclick ? 0 : undefined} {onclick}>
  {#if title || actions}
    <div class="card-header">
      {#if title}
        <div class="card-title">{@render title()}</div>
      {/if}
      {#if actions}
        <div class="card-actions">{@render actions()}</div>
      {/if}
    </div>
  {/if}
  {#if children}
    <div class="card-body">{@render children()}</div>
  {/if}
</div>

<style>
  .card {
    background: var(--bg-card);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    transition: border-color var(--transition-fast), box-shadow var(--transition-fast);
  }
  .card-hoverable:hover {
    border-color: var(--border-strong);
    box-shadow: var(--shadow-sm);
  }
  .card-pad-none { padding: 0; }
  .card-pad-sm { padding: var(--space-2); }
  .card-pad-md { padding: var(--space-4); }
  .card-pad-lg { padding: var(--space-6); }

  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
    padding-bottom: var(--space-3);
    border-bottom: 1px solid var(--border-default);
    margin-bottom: var(--space-3);
  }
  .card-title {
    font-size: var(--font-size-sm);
    font-weight: 600;
    color: var(--text-primary);
  }
  .card-actions {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    flex-shrink: 0;
  }
  .card-body {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
  }
</style>
