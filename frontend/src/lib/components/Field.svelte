<script lang="ts">
  let {
    label,
    hint,
    error: fieldError,
    class: className = '',
    children,
  }: {
    label: string;
    hint?: string;
    error?: string;
    class?: string;
    children?: import('svelte').Snippet;
  } = $props();
</script>

<div class="field {className}">
  <label class="field-label">{label}</label>
  {#if children}
    <div class="field-control">{@render children()}</div>
  {/if}
  {#if hint && !fieldError}
    <p class="field-hint">{hint}</p>
  {/if}
  {#if fieldError}
    <p class="field-error">{fieldError}</p>
  {/if}
</div>

<style>
  .field {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }
  .field-label {
    font-size: var(--font-size-sm);
    font-weight: 500;
    color: var(--text-secondary);
  }
  .field-control {
    display: flex;
    flex-direction: column;
  }
  .field-hint {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    margin: 0;
  }
  .field-error {
    font-size: var(--font-size-xs);
    color: var(--color-danger);
    margin: 0;
  }
</style>
