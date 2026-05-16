<script lang="ts">
  import { MoreHorizontal } from 'lucide-svelte';

  let {
    items,
    align = 'right',
    label = 'More',
    icon = MoreHorizontal,
  }: {
    items: { label: string; icon?: any; variant?: string; onclick: () => void }[];
    align?: 'left' | 'right';
    label?: string;
    icon?: any;
  } = $props();

  let open = $state(false);

  function handleBlur() {
    // Delay to allow click on item before closing
    setTimeout(() => open = false, 150);
  }
</script>

<div class="dropdown-wrap" onblur={handleBlur}>
  <button class="dropdown-trigger" onclick={() => open = !open} aria-label={label} aria-expanded={open}>
    <svelte:component this={icon} size={14} />
  </button>
  {#if open}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="dropdown-menu dropdown-{align}" onclick={() => open = false}>
      {#each items as item (item.label)}
        <button class="dropdown-item" class:danger={item.variant === 'danger'} onclick={item.onclick}>
          {#if item.icon}
            <svelte:component this={item.icon} size={14} />
          {/if}
          {item.label}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .dropdown-wrap {
    position: relative;
    display: inline-flex;
  }
  .dropdown-trigger {
    display: flex; align-items: center; justify-content: center;
    padding: 6px; border: 1px solid var(--border-default);
    background: var(--bg-card); color: var(--text-secondary);
    border-radius: var(--radius-md); cursor: pointer;
    transition: background var(--transition-fast);
  }
  .dropdown-trigger:hover { background: var(--bg-hover); color: var(--text-primary); }

  .dropdown-menu {
    position: absolute; top: 100%; margin-top: 4px;
    z-index: var(--z-dialog);
    background: var(--bg-elevated); border: 1px solid var(--border-default);
    border-radius: var(--radius-md); box-shadow: var(--shadow-md);
    min-width: 160px; padding: var(--space-1);
    display: flex; flex-direction: column; gap: 1px;
  }
  .dropdown-right { right: 0; }
  .dropdown-left { left: 0; }

  .dropdown-item {
    display: flex; align-items: center; gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    background: none; border: none; border-radius: var(--radius-sm);
    color: var(--text-secondary); font-size: var(--font-size-sm);
    cursor: pointer; text-align: left; width: 100%;
    transition: background var(--transition-fast);
  }
  .dropdown-item:hover { background: var(--bg-hover); color: var(--text-primary); }
  .dropdown-item.danger { color: var(--color-danger); }
  .dropdown-item.danger:hover { background: var(--color-danger-bg); }
</style>
