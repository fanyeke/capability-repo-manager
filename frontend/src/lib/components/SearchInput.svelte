<script lang="ts">
  import { Search, X } from 'lucide-svelte';

  let {
    value = '',
    placeholder = 'Search...',
    onInput,
    class: className = '',
  }: {
    value?: string;
    placeholder?: string;
    onInput: (val: string) => void;
    class?: string;
  } = $props();
</script>

<div class="search-input {className}">
  <Search size={14} class="search-icon" />
  <input
    type="text"
    {placeholder}
    value={value}
    oninput={(e) => onInput((e.target as HTMLInputElement).value)}
    class="search-field"
  />
  {#if value}
    <button class="search-clear" onclick={() => onInput('')} aria-label="Clear">
      <X size={14} />
    </button>
  {/if}
</div>

<style>
  .search-input {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: 6px 10px;
    background: var(--bg-card);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    transition: border-color var(--transition-fast);
  }
  .search-input:focus-within {
    border-color: var(--color-primary);
  }
  .search-icon {
    color: var(--text-muted);
    flex-shrink: 0;
  }
  .search-field {
    flex: 1;
    background: none;
    border: none;
    color: var(--text-primary);
    font-size: var(--font-size-sm);
    outline: none;
    min-width: 0;
  }
  .search-field::placeholder {
    color: var(--text-muted);
  }
  .search-clear {
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 0;
    width: 18px;
    height: 18px;
    border-radius: var(--radius-sm);
    flex-shrink: 0;
  }
  .search-clear:hover {
    background: var(--bg-hover);
    color: var(--text-secondary);
  }
</style>
