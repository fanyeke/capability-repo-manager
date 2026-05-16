<script lang="ts">
  import { _ } from 'svelte-i18n';
  import type { CompareResult, CapabilityResource, DiffItem } from '$lib/types';
  import ResourceDetail from './ResourceDetail.svelte';

  let {
    result,
    selectedCategory,
    onSelectCategory,
  }: {
    result: CompareResult | null;
    selectedCategory: 'missing' | 'extra' | 'modified' | 'same';
    onSelectCategory: (cat: 'missing' | 'extra' | 'modified' | 'same') => void;
  } = $props();

  let selectedResource = $state<CapabilityResource | null>(null);

  let currentItems = $derived(result ? result[selectedCategory] : []);
  let categories = $derived([
    {
      key: 'missing' as const,
      labelKey: 'compare.missing' as const,
      count: result?.missing.length ?? 0,
      color: '#dc2626',
    },
    {
      key: 'extra' as const,
      labelKey: 'compare.extra' as const,
      count: result?.extra.length ?? 0,
      color: '#d97706',
    },
    {
      key: 'modified' as const,
      labelKey: 'compare.modified' as const,
      count: result?.modified.length ?? 0,
      color: '#2563eb',
    },
    {
      key: 'same' as const,
      labelKey: 'compare.same' as const,
      count: result?.same.length ?? 0,
      color: '#16a34a',
    },
  ]);
</script>

{#if !result}
  <p class="empty-text">{$_('compare.no_result')}</p>
{:else}
  <div class="compare-view">
    <div class="category-tabs">
      {#each categories as cat}
        <button
          class="cat-tab"
          class:active={selectedCategory === cat.key}
          style="--cat-color: {cat.color}"
          onclick={() => onSelectCategory(cat.key)}
        >
          {$_(cat.labelKey)}
          <span class="cat-count">{cat.count}</span>
        </button>
      {/each}
    </div>

    <div class="compare-content">
      <div class="item-list">
        {#if selectedCategory === 'modified'}
          {#each currentItems as item (item.name + item.type)}
            {@const diff = item as DiffItem}
            <button
              class="item-btn"
              class:selected={selectedResource?.id === diff.source_resource.id}
              onclick={() => (selectedResource = diff.source_resource)}
            >
              <span class="item-name">{diff.name}</span>
              <span class="item-type">{diff.type}</span>
            </button>
          {/each}
        {:else}
          {#each currentItems as resource, i (resource.name + i)}
            {@const r = resource as CapabilityResource}
            <button
              class="item-btn"
              class:selected={selectedResource?.id === r.id}
              onclick={() => (selectedResource = r)}
            >
              <span class="item-name">{r.name}</span>
              <span class="item-type">{r.type}</span>
            </button>
          {/each}
        {/if}
      </div>

      <div class="detail-panel">
        {#if selectedResource}
          <ResourceDetail resource={selectedResource} />
          {#if selectedCategory === 'modified'}
            {@const diff = currentItems.find(
              (i) => (i as DiffItem).source_resource.id === selectedResource?.id,
            )}
            {#if diff}
              <div class="diff-divider">{$_('compare.vs_target')}</div>
              <ResourceDetail resource={(diff as DiffItem).target_resource} />
            {/if}
          {/if}
        {:else}
          <p class="select-hint">{$_('compare.select_item')}</p>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .compare-view {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }
  .category-tabs {
    display: flex;
    gap: var(--space-1);
  }
  .cat-tab {
    padding: var(--space-2) var(--space-4);
    border: 2px solid transparent;
    border-radius: var(--radius-md);
    background: var(--bg-card);
    cursor: pointer;
    font-size: var(--font-size-sm);
    display: flex;
    gap: 6px;
    align-items: center;
  }
  .cat-tab.active {
    border-color: var(--cat-color);
    background: color-mix(in srgb, var(--cat-color) 8%, var(--bg-card));
  }
  .cat-count {
    font-size: var(--font-size-xs);
    padding: 1px 6px;
    border-radius: var(--radius-full);
    background: var(--bg-hover);
  }
  .compare-content {
    display: grid;
    grid-template-columns: 250px 1fr;
    gap: var(--space-4);
    min-height: 300px;
  }
  .item-list {
    border-right: 1px solid var(--border-default);
    overflow-y: auto;
    max-height: 500px;
  }
  .item-btn {
    width: 100%;
    padding: var(--space-2) var(--space-3);
    border: none;
    background: none;
    text-align: left;
    cursor: pointer;
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-radius: var(--radius-sm);
  }
  .item-btn:hover {
    background: var(--bg-hover);
  }
  .item-btn.selected {
    background: var(--color-primary-bg);
  }
  .item-name {
    font-size: var(--font-size-sm);
    font-weight: 500;
  }
  .item-type {
    font-size: var(--font-size-xs);
    padding: 1px 6px;
    border-radius: var(--radius-sm);
    background: var(--border-default);
    color: var(--text-secondary);
  }
  .detail-panel {
    overflow-y: auto;
    max-height: 500px;
  }
  .diff-divider {
    margin: var(--space-4) 0 var(--space-2);
    padding: var(--space-1) var(--space-3);
    background: var(--bg-elevated);
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    font-weight: 600;
    text-align: center;
  }
  .select-hint {
    text-align: center;
    color: var(--text-muted);
    padding: 40px 0;
  }
  .empty-text {
    text-align: center;
    color: var(--text-muted);
    padding: 40px 0;
  }
</style>
