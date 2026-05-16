<script lang="ts">
  import { _ } from "svelte-i18n";
  import type { CapabilityResource } from "$lib/types";

  let {
    resources,
    selectedType,
    selectedResourceId,
    onSelectType,
    onSelectResource,
  }: {
    resources: { type: string; label: string; count: number }[];
    selectedType: string | null;
    selectedResourceId: string | null;
    onSelectType: (type: string | null) => void;
    onSelectResource: (id: string) => void;
  } = $props();
</script>

<aside class="capability-sidebar">
  <ul class="type-list">
    <li>
      <button
        class="type-btn"
        class:active={selectedType === null}
        onclick={() => onSelectType(null)}
      >
        {$_('capability.all')}
        <span class="count">{resources.reduce((s, g) => s + g.count, 0)}</span>
      </button>
    </li>
    {#each resources as group (group.type)}
      <li>
        <button
          class="type-btn"
          class:active={selectedType === group.type}
          onclick={() => onSelectType(group.type)}
        >
          {group.label}
          <span class="count">{group.count}</span>
        </button>
      </li>
    {/each}
  </ul>
</aside>

<style>
  .capability-sidebar {
    min-width: 200px;
    border-right: 1px solid var(--border-color, #e2e8f0);
    padding-right: 12px;
  }
  .type-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }
  .type-btn {
    width: 100%;
    padding: 8px 12px;
    border: none;
    background: none;
    text-align: left;
    font-size: 0.9rem;
    cursor: pointer;
    border-radius: 6px;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .type-btn:hover {
    background: #f1f5f9;
  }
  .type-btn.active {
    background: var(--primary, #3b82f6);
    color: #fff;
  }
  .count {
    font-size: 0.75rem;
    padding: 1px 6px;
    border-radius: 10px;
    background: rgba(0, 0, 0, 0.08);
  }
  .type-btn.active .count {
    background: rgba(255, 255, 255, 0.25);
  }
</style>
