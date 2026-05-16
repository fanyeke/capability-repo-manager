<script lang="ts">
  let {
    icon,
    variant = 'ghost',
    size = 'md',
    disabled = false,
    label,
    class: className = '',
    onclick,
  }: {
    icon: any;
    variant?: 'primary' | 'secondary' | 'ghost' | 'danger';
    size?: 'sm' | 'md' | 'lg';
    disabled?: boolean;
    label: string;
    class?: string;
    onclick?: (e: MouseEvent) => void;
  } = $props();
</script>

<button
  class="icon-btn icon-btn-{variant} icon-btn-{size} {className}"
  {disabled}
  aria-label={label}
  title={label}
  {onclick}
>
  <svelte:component this={icon} size={size === 'sm' ? 14 : size === 'lg' ? 20 : 16} />
</button>

<style>
  .icon-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border: 1px solid transparent;
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: background var(--transition-fast), color var(--transition-fast);
    flex-shrink: 0;
  }
  .icon-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .icon-btn:focus-visible {
    outline: none;
    box-shadow: var(--shadow-glow);
  }

  .icon-btn-sm { width: 26px; height: 26px; }
  .icon-btn-md { width: 30px; height: 30px; }
  .icon-btn-lg { width: 36px; height: 36px; }

  .icon-btn-primary {
    background: var(--color-primary);
    color: #fff;
  }
  .icon-btn-primary:hover:not(:disabled) {
    background: var(--color-primary-hover);
  }
  .icon-btn-secondary {
    background: var(--bg-card);
    color: var(--text-secondary);
    border-color: var(--border-default);
  }
  .icon-btn-secondary:hover:not(:disabled) {
    background: var(--bg-hover);
    color: var(--text-primary);
  }
  .icon-btn-ghost {
    background: none;
    color: var(--text-secondary);
  }
  .icon-btn-ghost:hover:not(:disabled) {
    background: var(--bg-hover);
    color: var(--text-primary);
  }
  .icon-btn-danger {
    background: var(--color-danger);
    color: #fff;
  }
</style>
