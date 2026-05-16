<script lang="ts">
  import { _ } from 'svelte-i18n';
  import type { MigrationConflict } from '$lib/types';

  let {
    conflicts,
  }: {
    conflicts: MigrationConflict[];
  } = $props();
</script>

{#if conflicts.length === 0}
  <p class="no-conflicts">{$_('migration.no_conflicts')}</p>
{:else}
  <div class="conflict-list">
    <h4>{$_('migration.conflicts_title', { values: { n: conflicts.length } })}</h4>
    {#each conflicts as conflict (conflict.resource_name + conflict.resource_type)}
      <div class="conflict-card">
        <div class="conflict-header">
          <strong>{conflict.resource_name}</strong>
          <span class="type-tag">{conflict.resource_type}</span>
        </div>
        <p class="conflict-reason">
          {$_('migration.reason', { values: { reason: conflict.reason } })}
        </p>
        {#if conflict.recommended_actions.length > 0}
          <div class="recommended">
            <span class="rec-label">{$_('migration.recommended')}</span>
            {#each conflict.recommended_actions as action}
              <span class="rec-action">{action}</span>
            {/each}
          </div>
        {/if}
      </div>
    {/each}
  </div>
{/if}

<style>
  .conflict-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .conflict-list h4 {
    margin: 0;
    color: #92400e;
  }
  .conflict-card {
    background: #fffbeb;
    border: 1px solid #fde68a;
    padding: 10px 14px;
    border-radius: 6px;
  }
  .conflict-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 4px;
  }
  .conflict-header strong {
    font-size: 0.9rem;
  }
  .type-tag {
    font-size: 0.7rem;
    padding: 1px 6px;
    border-radius: 4px;
    background: #e0e7ff;
    color: #3730a3;
  }
  .conflict-reason {
    font-size: 0.8rem;
    color: #b45309;
    margin: 4px 0;
  }
  .recommended {
    display: flex;
    gap: 4px;
    align-items: center;
    flex-wrap: wrap;
  }
  .rec-label {
    font-size: 0.75rem;
    color: #92400e;
  }
  .rec-action {
    padding: 1px 6px;
    background: #fef3c7;
    border-radius: 3px;
    font-size: 0.7rem;
    color: #92400e;
  }
  .no-conflicts {
    color: #16a34a;
    font-size: 0.9rem;
    padding: 12px 0;
  }
</style>
