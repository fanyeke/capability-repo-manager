<script lang="ts">
  import { _ } from "svelte-i18n";
  import type { PackSummary } from "$lib/types";

  let {
    packs,
    isLoading = false,
    onSelectPack,
    onDeletePack,
  }: {
    packs: PackSummary[];
    isLoading?: boolean;
    onSelectPack: (id: string) => void;
    onDeletePack: (id: string) => void;
  } = $props();
</script>

<div class="pack-library">
  {#if isLoading}
    <p class="loading-text">{$_('pack.loading')}</p>
  {:else if packs.length === 0}
    <p class="empty-text">{$_('pack.no_packs_library')}</p>
  {:else}
    <div class="pack-grid">
      {#each packs as pack (pack.id)}
        <article class="pack-card">
          <div class="pack-header" onclick={() => onSelectPack(pack.id)} onkeydown={() => {}} role="button" tabindex="0">
            <h3>{pack.name}</h3>
            <span class="version-tag">v{pack.version}</span>
          </div>

          <div class="pack-body" onclick={() => onSelectPack(pack.id)} onkeydown={() => {}} role="button" tabindex="0">
            {#if pack.description}<p class="description">{pack.description}</p>{/if}

            <div class="pack-meta">
              <span class="type-badge type-{pack.pack_type}">{pack.pack_type}</span>
              <span class="resource-count">{$_('pack.resources_count', { values: { n: pack.resource_count } })}</span>
            </div>

            {#if pack.source_repo_name}
              <p class="source">{$_('pack.from', { values: { name: pack.source_repo_name } })}</p>
            {/if}
          </div>

          <div class="pack-footer">
            <span class="created">{new Date(pack.created_at).toLocaleDateString()}</span>
            <button
              class="delete-btn"
              onclick={(e) => { e.stopPropagation(); onDeletePack(pack.id); }}
            >
              {$_('pack.delete')}
            </button>
          </div>
        </article>
      {/each}
    </div>
  {/if}
</div>

<style>
  .pack-library {
    width: 100%;
  }
  .pack-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 16px;
  }
  .pack-card {
    background: var(--card-bg, #fff);
    border: 1px solid var(--border-color, #e2e8f0);
    border-radius: 8px;
    overflow: hidden;
  }
  .pack-header {
    padding: 12px 16px;
    cursor: pointer;
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid #f1f5f9;
  }
  .pack-header h3 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
  }
  .version-tag {
    background: #e0e7ff;
    color: #3730a3;
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 0.75rem;
  }
  .pack-body {
    padding: 12px 16px;
    cursor: pointer;
  }
  .description {
    font-size: 0.85rem;
    color: #475569;
    margin: 0 0 8px 0;
  }
  .pack-meta {
    display: flex;
    gap: 8px;
    align-items: center;
    margin-bottom: 4px;
  }
  .type-badge {
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 0.7rem;
    text-transform: capitalize;
  }
  .type-project { background: #dbeafe; color: #1e40af; }
  .type-blueprint { background: #ede9fe; color: #5b21b6; }
  .type-baseline { background: #fce7f3; color: #831843; }
  .resource-count {
    font-size: 0.8rem;
    color: #64748b;
  }
  .source {
    font-size: 0.75rem;
    color: #94a3b8;
    margin: 4px 0 0 0;
  }
  .pack-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 16px;
    border-top: 1px solid #f1f5f9;
    font-size: 0.75rem;
    color: #94a3b8;
  }
  .delete-btn {
    padding: 3px 10px;
    border: 1px solid #fecaca;
    background: #fff;
    color: #dc2626;
    border-radius: 4px;
    font-size: 0.75rem;
    cursor: pointer;
  }
  .delete-btn:hover {
    background: #fef2f2;
  }
  .loading-text, .empty-text {
    text-align: center;
    color: #64748b;
    padding: 40px 0;
  }
</style>
