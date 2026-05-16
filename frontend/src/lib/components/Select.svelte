<script lang="ts">
  import { ChevronDown } from 'lucide-svelte';

  let {
    options,
    value,
    onChange,
    placeholder,
    class: className = '',
  }: {
    options: { value: string; label: string }[];
    value: string;
    onChange: (val: string) => void;
    placeholder?: string;
    class?: string;
  } = $props();
</script>

<div class="select-wrapper {className}">
  <select
    class="select-field"
    value={value}
    onchange={(e) => onChange((e.target as HTMLSelectElement).value)}
  >
    {#if placeholder}
      <option value="" disabled>{placeholder}</option>
    {/if}
    {#each options as opt (opt.value)}
      <option value={opt.value}>{opt.label}</option>
    {/each}
  </select>
  <ChevronDown size={14} class="select-chevron" />
</div>

<style>
  .select-wrapper {
    position: relative;
    display: inline-flex;
    align-items: center;
  }
  .select-field {
    appearance: none;
    -webkit-appearance: none;
    padding: 6px 30px 6px 10px;
    background: var(--bg-card);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    color: var(--text-primary);
    font-size: var(--font-size-sm);
    font-family: var(--font-sans);
    cursor: pointer;
    outline: none;
    transition: border-color var(--transition-fast);
    min-width: 100px;
  }
  .select-field:focus {
    border-color: var(--color-primary);
  }
  .select-field:hover {
    background: var(--bg-hover);
  }
  .select-chevron {
    position: absolute;
    right: 10px;
    pointer-events: none;
    color: var(--text-muted);
  }
</style>
