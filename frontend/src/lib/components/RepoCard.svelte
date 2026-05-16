<script lang="ts">
  import { _ } from 'svelte-i18n';
  import type { RepositorySummary } from '$lib/types';

  let {
    repo,
    onSelect,
  }: {
    repo: RepositorySummary;
    onSelect: (id: string) => void;
  } = $props();
</script>

<article
  class="repo-card"
  onclick={() => onSelect(repo.id)}
  onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); onSelect(repo.id); } }}
  role="button"
  tabindex="0"
>
  <div class="card-header">
    <h3 class="repo-name">{repo.name}</h3>
    <span
      class="dirty-badge"
      class:clean={repo.dirty_state === 'clean'}
      class:modified={repo.dirty_state === 'modified'}
    >
      {repo.dirty_state}
    </span>
  </div>

  <p class="repo-path" title={repo.path}>{repo.path}</p>

  {#if repo.branch}
    <span class="branch-tag">{repo.branch}</span>
  {/if}

  <div class="capability-counts">
    {#if repo.capability_counts.skill}
      <span class="count-badge skill"
        >{$_('repo.skills_count', { values: { n: repo.capability_counts.skill } })}</span
      >
    {/if}
    {#if repo.capability_counts.mcp}
      <span class="count-badge mcp"
        >{$_('repo.mcp_count', { values: { n: repo.capability_counts.mcp } })}</span
      >
    {/if}
    {#if repo.capability_counts.hook}
      <span class="count-badge hook"
        >{$_('repo.hooks_count', { values: { n: repo.capability_counts.hook } })}</span
      >
    {/if}
    {#if repo.capability_counts.rule}
      <span class="count-badge rule"
        >{$_('repo.rules_count', { values: { n: repo.capability_counts.rule } })}</span
      >
    {/if}
    {#if repo.capability_counts.agent}
      <span class="count-badge agent"
        >{$_('repo.agents_count', { values: { n: repo.capability_counts.agent } })}</span
      >
    {/if}
  </div>

  <div class="card-footer">
    <span class="indexed-at"
      >{$_('repo.indexed', {
        values: { date: new Date(repo.last_indexed_at).toLocaleDateString() },
      })}</span
    >
  </div>
</article>

<style>
  .repo-card {
    background: var(--bg-card);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    padding: var(--space-4);
    cursor: pointer;
    transition:
      box-shadow var(--transition-fast),
      border-color var(--transition-fast);
  }
  .repo-card:hover {
    box-shadow: var(--shadow-sm);
    border-color: var(--color-primary);
  }
  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--space-2);
  }
  .repo-name {
    margin: 0;
    font-size: var(--font-size-md);
    font-weight: 600;
  }
  .dirty-badge {
    font-size: var(--font-size-xs);
    padding: 2px 8px;
    border-radius: var(--radius-full);
    text-transform: uppercase;
  }
  .dirty-badge.clean {
    background: var(--color-success-bg);
    color: var(--color-success);
  }
  .dirty-badge.modified {
    background: var(--color-warning-bg);
    color: var(--color-warning);
  }
  .repo-path {
    color: var(--text-muted);
    font-size: var(--font-size-xs);
    margin: var(--space-1) 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .branch-tag {
    display: inline-block;
    background: var(--color-primary-bg);
    color: var(--color-primary-text);
    font-size: var(--font-size-xs);
    padding: 1px 8px;
    border-radius: var(--radius-sm);
    margin-bottom: var(--space-2);
  }
  .capability-counts {
    display: flex;
    gap: var(--space-1);
    flex-wrap: wrap;
    margin: var(--space-2) 0;
  }
  .count-badge {
    font-size: var(--font-size-xs);
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    background: var(--bg-elevated);
    color: var(--text-secondary);
  }
  .card-footer {
    margin-top: var(--space-2);
    font-size: var(--font-size-xs);
    color: var(--text-muted);
  }
</style>
