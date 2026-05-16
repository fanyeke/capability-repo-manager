<script lang="ts">
  let {
    tabs,
    active,
    onChange,
    class: className = '',
  }: {
    tabs: { id: string; label: string }[];
    active: string;
    onChange: (id: string) => void;
    class?: string;
  } = $props();
</script>

<div class="tabs {className}">
  {#each tabs as tab (tab.id)}
    <button
      class="tab"
      class:active={active === tab.id}
      onclick={() => onChange(tab.id)}
    >
      {tab.label}
      {#if active === tab.id}
        <span class="tab-indicator"></span>
      {/if}
    </button>
  {/each}
</div>

<style>
  .tabs {
    display: flex;
    gap: 0;
    border-bottom: 1px solid var(--border-default);
  }
  .tab {
    position: relative;
    padding: var(--space-2) var(--space-4);
    background: none;
    border: none;
    color: var(--text-muted);
    font-size: var(--font-size-sm);
    font-weight: 500;
    cursor: pointer;
    transition: color var(--transition-fast);
    white-space: nowrap;
  }
  .tab:hover {
    color: var(--text-secondary);
  }
  .tab.active {
    color: var(--text-primary);
  }
  .tab-indicator {
    position: absolute;
    bottom: -1px;
    left: 0;
    right: 0;
    height: 2px;
    background: var(--color-primary);
    border-radius: 1px 1px 0 0;
  }
</style>
