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
    gap: var(--space-4);
  }
  .plan-summary {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: var(--space-3);
    background: var(--bg-elevated);
    padding: var(--space-4);
    border-radius: var(--radius-md);
  }
  .summary-item .label {
    display: block;
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .has-conflicts {
    color: var(--color-danger);
    font-weight: 600;
  }
  .conflicts-section {
    background: var(--color-warning-bg);
    border: 1px solid var(--color-warning);
    padding: var(--space-3);
    border-radius: var(--radius-md);
  }
  .conflicts-section h4 {
    margin: 0 0 var(--space-2) 0;
    color: var(--color-warning);
  }
  .conflict-item {
    display: flex;
    gap: var(--space-2);
    font-size: var(--font-size-sm);
    padding: var(--space-1) 0;
  }
  .conflict-name {
    font-weight: 600;
  }
  .conflict-type {
    color: var(--text-muted);
  }
  .conflict-reason {
    color: var(--color-warning);
  }
  .plan-table {
    width: 100%;
    border-collapse: collapse;
  }
  .plan-table th {
    text-align: left;
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    padding: var(--space-2);
    border-bottom: 1px solid var(--border-default);
  }
  .plan-table td {
    padding: var(--space-2);
    border-bottom: 1px solid var(--bg-hover);
    font-size: var(--font-size-sm);
  }
  .action-badge {
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    font-size: var(--font-size-xs);
  }
  .action-add {
    background: var(--color-success-bg);
    color: var(--color-success);
  }
  .action-overwrite {
    background: var(--color-warning-bg);
    color: var(--color-warning);
  }
  .action-skip {
    background: var(--bg-hover);
    color: var(--text-secondary);
  }
  .action-unresolved {
    background: var(--color-danger-bg);
    color: var(--color-danger);
  }
  .action-rename {
    background: var(--color-primary-bg);
    color: var(--color-primary-text);
  }
  .action-merge {
    background: var(--color-primary-bg);
    color: var(--color-primary-text);
  }
  .strategy-auto {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
  }
  select {
    padding: 3px var(--space-2);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-default);
    font-size: var(--font-size-sm);
  }
  code {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
  }
  .empty-text {
    text-align: center;
    color: var(--text-muted);
    padding: 40px 0;
  }
</style>
