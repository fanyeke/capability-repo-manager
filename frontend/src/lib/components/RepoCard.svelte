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
  onkeydown={() => {}}
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
    background: var(--card-bg, #fff);
    border: 1px solid var(--border-color, #e2e8f0);
    border-radius: 8px;
    padding: 16px;
    cursor: pointer;
    transition:
      box-shadow 0.15s,
      border-color 0.15s;
  }
  .repo-card:hover {
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    border-color: var(--primary, #3b82f6);
  }
  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }
  .repo-name {
    margin: 0;
    font-size: 1.05rem;
    font-weight: 600;
  }
  .dirty-badge {
    font-size: 0.7rem;
    padding: 2px 8px;
    border-radius: 12px;
    text-transform: uppercase;
  }
  .dirty-badge.clean {
    background: #dcfce7;
    color: #166534;
  }
  .dirty-badge.modified {
    background: #fef3c7;
    color: #92400e;
  }
  .repo-path {
    color: #64748b;
    font-size: 0.8rem;
    margin: 4px 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .branch-tag {
    display: inline-block;
    background: #ede9fe;
    color: #5b21b6;
    font-size: 0.75rem;
    padding: 1px 8px;
    border-radius: 4px;
    margin-bottom: 8px;
  }
  .capability-counts {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
    margin: 8px 0;
  }
  .count-badge {
    font-size: 0.7rem;
    padding: 2px 6px;
    border-radius: 4px;
    background: #f1f5f9;
    color: #475569;
  }
  .card-footer {
    margin-top: 8px;
    font-size: 0.7rem;
    color: #94a3b8;
  }
</style>
