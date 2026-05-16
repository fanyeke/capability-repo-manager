<script lang="ts">
  import { _ } from 'svelte-i18n';
  import type { MigrationPlan, MigrationPlanItem, MigrationConflict } from '$lib/types';

  let {
    plan,
    strategies,
    onSetStrategy,
  }: {
    plan: MigrationPlan | null;
    strategies: { resource_id: string; action: string }[];
    onSetStrategy: (resourceId: string, action: string) => void;
  } = $props();

  function truncate(path: string | null): string {
    if (!path) return '—';
    return path.length > 40 ? '...' + path.slice(-37) : path;
  }

  function getStrategy(resourceId: string): string {
    return strategies.find((s) => s.resource_id === resourceId)?.action ?? 'skip';
  }
</script>

{#if !plan}
  <p class="empty-text">{$_('migration.no_plan')}</p>
{:else}
  <div class="plan-container">
    <div class="plan-summary">
      <div class="summary-item">
        <span class="label">{$_('migration.source')}</span>
        <span>{plan.source_type}: {plan.source_id}</span>
      </div>
      <div class="summary-item">
        <span class="label">{$_('migration.target_repo')}</span>
        <span>{plan.target_repo_id}</span>
      </div>
      <div class="summary-item">
        <span class="label">{$_('migration.items')}</span>
        <span>{plan.items.length}</span>
      </div>
      <div class="summary-item">
        <span class="label">{$_('migration.conflicts')}</span>
        <span class:has-conflicts={plan.conflicts.length > 0}>
          {plan.conflicts.length}
        </span>
      </div>
    </div>

    {#if plan.conflicts.length > 0}
      <div class="conflicts-section">
        <h4>{$_('migration.conflicts')}</h4>
        {#each plan.conflicts as conflict (conflict.resource_name + conflict.resource_type)}
          <div class="conflict-item">
            <span class="conflict-name">{conflict.resource_name}</span>
            <span class="conflict-type">{conflict.resource_type}</span>
            <span class="conflict-reason">{conflict.reason}</span>
          </div>
        {/each}
      </div>
    {/if}

    <table class="plan-table">
      <thead>
        <tr>
          <th>{$_('migration.resource')}</th>
          <th>{$_('migration.type')}</th>
          <th>{$_('migration.action')}</th>
          <th>{$_('migration.strategy')}</th>
        </tr>
      </thead>
      <tbody>
        {#each plan.items as item (item.resource_id)}
          <tr>
            <td class="resource-name">{item.resource_id}</td>
            <td><span class="action-badge action-{item.action}">{item.action}</span></td>
            <td>
              {#if item.source_path}
                <code title={item.source_path}>{truncate(item.source_path)}</code>
              {:else}
                —
              {/if}
            </td>
            <td>
              {#if item.action === 'unresolved' || item.action === 'overwrite'}
                <select
                  value={getStrategy(item.resource_id)}
                  onchange={(e) =>
                    onSetStrategy(item.resource_id, (e.target as HTMLSelectElement).value)}
                >
                  <option value="skip">{$_('migration.skip')}</option>
                  <option value="overwrite">{$_('migration.overwrite')}</option>
                </select>
              {:else}
                <span class="strategy-auto">{item.action}</span>
              {/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{/if}

<style>
  .plan-container {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .plan-summary {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 12px;
    background: #f8fafc;
    padding: 16px;
    border-radius: 8px;
  }
  .summary-item .label {
    display: block;
    font-size: 0.7rem;
    color: #64748b;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .has-conflicts {
    color: #dc2626;
    font-weight: 600;
  }
  .conflicts-section {
    background: #fffbeb;
    border: 1px solid #fde68a;
    padding: 12px;
    border-radius: 6px;
  }
  .conflicts-section h4 {
    margin: 0 0 8px 0;
    color: #92400e;
  }
  .conflict-item {
    display: flex;
    gap: 8px;
    font-size: 0.85rem;
    padding: 4px 0;
  }
  .conflict-name {
    font-weight: 600;
  }
  .conflict-type {
    color: #64748b;
  }
  .conflict-reason {
    color: #b45309;
  }
  .plan-table {
    width: 100%;
    border-collapse: collapse;
  }
  .plan-table th {
    text-align: left;
    font-size: 0.75rem;
    color: #64748b;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    padding: 8px;
    border-bottom: 1px solid #e2e8f0;
  }
  .plan-table td {
    padding: 8px;
    border-bottom: 1px solid #f1f5f9;
    font-size: 0.85rem;
  }
  .action-badge {
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 0.7rem;
  }
  .action-add {
    background: #dcfce7;
    color: #166534;
  }
  .action-overwrite {
    background: #fef3c7;
    color: #92400e;
  }
  .action-skip {
    background: #f1f5f9;
    color: #475569;
  }
  .action-unresolved {
    background: #fee2e2;
    color: #991b1b;
  }
  .action-rename {
    background: #e0e7ff;
    color: #3730a3;
  }
  .action-merge {
    background: #ede9fe;
    color: #5b21b6;
  }
  .strategy-auto {
    font-size: 0.8rem;
    color: #64748b;
  }
  select {
    padding: 3px 8px;
    border-radius: 4px;
    border: 1px solid #e2e8f0;
    font-size: 0.8rem;
  }
  code {
    font-size: 0.75rem;
    color: #64748b;
  }
  .empty-text {
    text-align: center;
    color: #64748b;
    padding: 40px 0;
  }
</style>
