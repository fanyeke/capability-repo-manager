<script lang="ts">
  import { _ } from "svelte-i18n";
  import type {
    CompareResult,
    CapabilityResource,
    DiffItem,
  } from "$lib/types";
  import ResourceDetail from "./ResourceDetail.svelte";

  let {
    result,
    selectedCategory,
    onSelectCategory,
  }: {
    result: CompareResult | null;
    selectedCategory: "missing" | "extra" | "modified" | "same";
    onSelectCategory: (cat: "missing" | "extra" | "modified" | "same") => void;
  } = $props();

  let selectedResource = $state<CapabilityResource | null>(null);

  let currentItems = $derived(result ? result[selectedCategory] : []);
  let categories = $derived([
    { key: "missing" as const, labelKey: "compare.missing" as const, count: result?.missing.length ?? 0, color: "#dc2626" },
    { key: "extra" as const, labelKey: "compare.extra" as const, count: result?.extra.length ?? 0, color: "#d97706" },
    { key: "modified" as const, labelKey: "compare.modified" as const, count: result?.modified.length ?? 0, color: "#2563eb" },
    { key: "same" as const, labelKey: "compare.same" as const, count: result?.same.length ?? 0, color: "#16a34a" },
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
        {#if selectedCategory === "modified"}
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
          {#if selectedCategory === "modified"}
            {@const diff = currentItems.find(
              (i) => (i as DiffItem).source_resource.id === selectedResource?.id
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
    gap: 16px;
  }
  .category-tabs {
    display: flex;
    gap: 4px;
  }
  .cat-tab {
    padding: 8px 16px;
    border: 2px solid transparent;
    border-radius: 6px;
    background: #fff;
    cursor: pointer;
    font-size: 0.85rem;
    display: flex;
    gap: 6px;
    align-items: center;
  }
  .cat-tab.active {
    border-color: var(--cat-color);
    background: color-mix(in srgb, var(--cat-color) 8%, #fff);
  }
  .cat-count {
    font-size: 0.7rem;
    padding: 1px 6px;
    border-radius: 10px;
    background: #f1f5f9;
  }
  .compare-content {
    display: grid;
    grid-template-columns: 250px 1fr;
    gap: 16px;
    min-height: 300px;
  }
  .item-list {
    border-right: 1px solid #e2e8f0;
    overflow-y: auto;
    max-height: 500px;
  }
  .item-btn {
    width: 100%;
    padding: 8px 12px;
    border: none;
    background: none;
    text-align: left;
    cursor: pointer;
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-radius: 4px;
  }
  .item-btn:hover { background: #f1f5f9; }
  .item-btn.selected { background: #e0e7ff; }
  .item-name { font-size: 0.85rem; font-weight: 500; }
  .item-type {
    font-size: 0.7rem;
    padding: 1px 6px;
    border-radius: 4px;
    background: #e2e8f0;
    color: #475569;
  }
  .detail-panel {
    overflow-y: auto;
    max-height: 500px;
  }
  .diff-divider {
    margin: 16px 0 8px;
    padding: 4px 12px;
    background: #f8fafc;
    font-size: 0.8rem;
    color: #64748b;
    font-weight: 600;
    text-align: center;
  }
  .select-hint {
    text-align: center;
    color: #94a3b8;
    padding: 40px 0;
  }
  .empty-text {
    text-align: center;
    color: #64748b;
    padding: 40px 0;
  }
</style>
