<script lang="ts">
  let {
    status,
    label,
    class: className = '',
  }: {
    status: 'success' | 'warning' | 'danger' | 'info' | 'clean' | 'modified';
    label?: string;
    class?: string;
  } = $props();
</script>

<span class="pill pill-{status} {className}">
  <span class="dot"></span>
  {#if label}
    <span class="pill-label">{label}</span>
  {/if}
</span>

<style>
  .pill {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: var(--font-size-xs);
    color: var(--text-muted);
  }
  .dot {
    width: 6px;
    height: 6px;
    border-radius: var(--radius-full);
    flex-shrink: 0;
  }
  .pill-label {
    text-transform: capitalize;
  }

  .pill-success .dot { background: var(--color-success); }
  .pill-warning .dot, .pill-modified .dot { background: var(--color-warning); }
  .pill-danger .dot { background: var(--color-danger); }
  .pill-info .dot { background: var(--color-info); }
  .pill-clean .dot { background: var(--color-clean); }
</style>
