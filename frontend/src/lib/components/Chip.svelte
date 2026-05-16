<script lang="ts">
  import { X } from 'lucide-svelte';

  let {
    items,
    onRemove,
    class: className = '',
  }: {
    items: string[];
    onRemove: (item: string) => void;
    class?: string;
  } = $props();
</script>

{#if items.length > 0}
  <div class="chips {className}">
    {#each items as item (item)}
      <span class="chip">
        <span class="chip-text">{item}</span>
        <button class="chip-remove" onclick={() => onRemove(item)} aria-label="Remove {item}">
          <X size={12} />
        </button>
      </span>
    {/each}
  </div>
{/if}

<style>
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1);
  }
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 3px 8px;
    background: var(--color-primary-bg);
    color: var(--color-primary-text);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-xs);
    line-height: 1;
  }
  .chip-remove {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    color: inherit;
    cursor: pointer;
    padding: 0;
    opacity: 0.7;
    border-radius: var(--radius-sm);
  }
  .chip-remove:hover {
    opacity: 1;
  }
  .chip-text {
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
