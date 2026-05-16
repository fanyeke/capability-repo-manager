<script lang="ts">
  let {
    title,
    description,
    actions,
  }: {
    title: string;
    description?: string;
    actions?: import('svelte').Snippet;
  } = $props();
</script>

<div class="page-header">
  <div class="header-text">
    <h1 class="page-title">{title}</h1>
    {#if description}
      <p class="page-description">{description}</p>
    {/if}
  </div>
  {#if actions}
    <div class="header-actions">
      {@render actions()}
    </div>
  {/if}
</div>

<style>
  .page-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-4);
    padding: var(--space-5) 0 var(--space-4);
  }
  .page-title {
    margin: 0;
    font-size: var(--font-size-xl);
    font-weight: 600;
    color: var(--text-primary);
  }
  .page-description {
    margin: var(--space-1) 0 0;
    font-size: var(--font-size-sm);
    color: var(--text-muted);
  }
  .header-actions {
    display: flex;
    gap: var(--space-2);
    flex-shrink: 0;
  }
</style>
