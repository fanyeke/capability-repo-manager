<script lang="ts">
  let {
    value = '',
    placeholder = '',
    type = 'text',
    disabled = false,
    onInput,
    class: className = '',
  }: {
    value?: string;
    placeholder?: string;
    type?: string;
    disabled?: boolean;
    onInput: (val: string) => void;
    class?: string;
  } = $props();
</script>

<input
  {type}
  {placeholder}
  {disabled}
  value={value}
  oninput={(e) => onInput((e.target as HTMLInputElement).value)}
  class="text-input {className}"
/>

<style>
  .text-input {
    width: 100%;
    padding: 8px 12px;
    background: var(--bg-card);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    color: var(--text-primary);
    font-size: var(--font-size-sm);
    font-family: var(--font-sans);
    outline: none;
    transition: border-color var(--transition-fast);
  }
  .text-input:focus {
    border-color: var(--color-primary);
  }
  .text-input::placeholder {
    color: var(--text-muted);
  }
  .text-input:disabled {
    opacity: 0.5;
  }
</style>
