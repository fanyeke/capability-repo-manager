<script lang="ts">
  import { _ } from 'svelte-i18n';
  import type { PackSummary } from '$lib/types';

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
          <div
            class="pack-header"
            onclick={() => onSelectPack(pack.id)}
            onkeydown={() => {}}
            role="button"
            tabindex="0"
          >
            <h3>{pack.name}</h3>
            <span class="version-tag">v{pack.version}</span>
          </div>

          <div
            class="pack-body"
            onclick={() => onSelectPack(pack.id)}
            onkeydown={() => {}}
            role="button"
            tabindex="0"
          >
            {#if pack.description}<p class="description">{pack.description}</p>{/if}

            <div class="pack-meta">
              <span class="type-badge type-{pack.pack_type}">{pack.pack_type}</span>
              <span class="resource-count"
                >{$_('pack.resources_count', { values: { n: pack.resource_count } })}</span
              >
            </div>

            {#if pack.source_repo_name}
              <p class="source">{$_('pack.from', { values: { name: pack.source_repo_name } })}</p>
            {/if}
          </div>

          <div class="pack-footer">
            <span class="created">{new Date(pack.created_at).toLocaleDateString()}</span>
            <button
              class="delete-btn"
              onclick={(e) => {
                e.stopPropagation();
                onDeletePack(pack.id);
              }}
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
    gap: var(--space-4);
  }
  .pack-card {
    background: var(--bg-card);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    overflow: hidden;
  }
  .pack-header {
    padding: var(--space-3) var(--space-4);
    cursor: pointer;
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid var(--bg-hover);
  }
  .pack-header h3 {
    margin: 0;
    font-size: var(--font-size-lg);
    font-weight: 600;
  }
  .version-tag {
    background: var(--color-primary-bg);
    color: var(--color-primary-text);
    padding: 2px var(--space-2);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-xs);
  }
  .pack-body {
    padding: var(--space-3) var(--space-4);
    cursor: pointer;
  }
  .description {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    margin: 0 0 var(--space-2) 0;
  }
  .pack-meta {
    display: flex;
    gap: var(--space-2);
    align-items: center;
    margin-bottom: var(--space-1);
  }
  .type-badge {
    padding: 2px var(--space-2);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-xs);
    text-transform: capitalize;
  }
  .type-project {
    background: var(--color-info-bg);
    color: var(--color-info);
  }
  .type-blueprint {
    background: var(--color-primary-bg);
    color: var(--color-primary-text);
  }
  .type-baseline {
    background: var(--color-danger-bg);
    color: var(--color-danger);
  }
  .resource-count {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
  }
  .source {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    margin: var(--space-1) 0 0 0;
  }
  .pack-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-2) var(--space-4);
    border-top: 1px solid var(--bg-hover);
    font-size: var(--font-size-xs);
    color: var(--text-muted);
  }
  .delete-btn {
    padding: 3px 10px;
    border: 1px solid var(--color-danger-bg);
    background: var(--bg-card);
    color: var(--color-danger);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-xs);
    cursor: pointer;
  }
  .delete-btn:hover {
    background: var(--color-danger-bg);
  }
  .loading-text,
  .empty-text {
    text-align: center;
    color: var(--text-muted);
    padding: 40px 0;
  }
</style>
