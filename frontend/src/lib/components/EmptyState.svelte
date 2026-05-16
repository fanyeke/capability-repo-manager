<script lang="ts">
  import { Inbox, type Icon as IconType } from 'lucide-svelte';

  let {
    icon = Inbox,
    title,
    description,
    action,
    class: className = '',
  }: {
    icon?: typeof IconType;
    title: string;
    description?: string;
    action?: { label: string; onClick: () => void };
    class?: string;
  } = $props();
</script>

<div class="empty-state {className}">
  <svelte:component this={icon} size={40} class="empty-icon" />
  <h3 class="empty-title">{title}</h3>
  {#if description}
    <p class="empty-description">{description}</p>
  {/if}
  {#if action}
    <button class="empty-action" onclick={action.onClick}>{action.label}</button>
  {/if}
</div>

<style>
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-8);
    text-align: center;
    min-height: 200px;
  }
  .empty-icon {
    color: var(--text-muted);
    margin-bottom: var(--space-3);
    opacity: 0.5;
  }
  .empty-title {
    font-size: var(--font-size-lg);
    font-weight: 600;
    color: var(--text-secondary);
    margin: 0 0 var(--space-1);
  }
  .empty-description {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    margin: 0 0 var(--space-4);
    max-width: 360px;
  }
  .empty-action {
    padding: 6px 16px;
    background: var(--color-primary);
    color: var(--text-inverse);
    border: none;
    border-radius: var(--radius-md);
    font-size: var(--font-size-sm);
    cursor: pointer;
    font-family: var(--font-sans);
  }
  .empty-action:hover {
    background: var(--color-primary-hover);
  }
</style>
