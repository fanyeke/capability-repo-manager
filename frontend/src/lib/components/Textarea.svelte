<script lang="ts">
  let {
    value = '',
    placeholder = '',
    rows = 3,
    disabled = false,
    onInput,
    class: className = '',
  }: {
    value?: string;
    placeholder?: string;
    rows?: number;
    disabled?: boolean;
    onInput: (val: string) => void;
    class?: string;
  } = $props();
</script>

<textarea
  {placeholder}
  {rows}
  {disabled}
  value={value}
  oninput={(e) => onInput((e.target as HTMLTextAreaElement).value)}
  class="textarea {className}"
></textarea>

<style>
  .textarea {
    width: 100%;
    padding: 8px 12px;
    background: var(--bg-card);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    color: var(--text-primary);
    font-size: var(--font-size-sm);
    font-family: var(--font-mono);
    outline: none;
    resize: vertical;
    transition: border-color var(--transition-fast);
  }
  .textarea:focus {
    border-color: var(--color-primary);
  }
  .textarea::placeholder {
    color: var(--text-muted);
  }
</style>
