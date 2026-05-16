<script lang="ts">
  import { currentPage, navigateToRepo } from '$lib/stores/uiStore';
  import { repos, filteredRepos, scanRepositories, loadRepos, selectRepo, isLoading, repoFilter } from '$lib/stores/repoStore';
  import { _ } from 'svelte-i18n';
  import { onMount } from 'svelte';
  import { RefreshCw, Search as SearchIcon, GitBranch, AlertTriangle, Shield, LayoutGrid, List } from 'lucide-svelte';
  import Button from '$lib/components/Button.svelte';
  import IconButton from '$lib/components/IconButton.svelte';
  import Card from '$lib/components/Card.svelte';
  import MetricCard from '$lib/components/MetricCard.svelte';
  import SearchInput from '$lib/components/SearchInput.svelte';
  import Select from '$lib/components/Select.svelte';
  import Badge from '$lib/components/Badge.svelte';
  import StatusPill from '$lib/components/StatusPill.svelte';
  import Skeleton from '$lib/components/Skeleton.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import ResourceIcon from '$lib/components/ResourceIcon.svelte';

  onMount(() => {
    isLoading.set(false);
    loadRepos();
  });

  async function handleScan() {
    const { invoke } = await import('@tauri-apps/api/core');
    const settings = await invoke<{ scan_roots: string[] }>('get_settings');
    if (settings.scan_roots.length > 0) {
      scanRepositories(settings.scan_roots).catch(() => {});
    } else {
      currentPage.set('guidedsetup');
    }
  }

  function handleSelectRepo(repoId: string) {
    selectRepo(repoId);
    navigateToRepo(repoId);
  }

  function handleFilterChange(newFilter: any) {
    repoFilter.set(newFilter);
  }

  function setSearch(val: string) {
    handleFilterChange({ ...$repoFilter, search: val || undefined });
  }

  let compactView = $state(false);

  function toggleView() {
    compactView = !compactView;
  }

  const sortOptions = [
    { value: 'name', label: $_('common.sort_name') },
    { value: 'last_indexed_at', label: $_('common.sort_date') },
    { value: 'dirty_state', label: $_('common.sort_state') },
  ];

  const totalRepos = $derived($repos.length);
  const dirtyRepos = $derived($repos.filter((r) => r.dirty_state !== 'clean').length);
  const avgDoctor = $derived(
    $repos.length > 0
      ? Math.round($repos.reduce((s, r) => s + (r.doctor_score ?? 0), 0) / $repos.length)
      : 0,
  );
</script>

<div class="dashboard">
  <div class="metric-row">
    <MetricCard label={$_('dashboard.total_repos')} value={totalRepos} icon={GitBranch} />
    <MetricCard label={$_('dashboard.modified')} value={dirtyRepos} icon={AlertTriangle} trend={dirtyRepos > 0 ? 'up' : 'neutral'} />
    <MetricCard label={$_('dashboard.needs_scan')} value={$repos.filter((r) => r.capability_index_status !== 'fresh').length} icon={RefreshCw} />
    <MetricCard label={$_('dashboard.avg_doctor')} value={avgDoctor} icon={Shield} trend={avgDoctor >= 80 ? 'up' : avgDoctor >= 50 ? 'neutral' : 'down'} />
  </div>

  <div class="toolbar">
    <div class="toolbar-left">
      <SearchInput value={$repoFilter.search ?? ''} placeholder={$_('common.search')} onInput={setSearch} />
      <Select
        options={sortOptions}
        value={$repoFilter.sort_by ?? 'name'}
        onChange={(val) => handleFilterChange({ ...$repoFilter, sort_by: val })}
      />
    </div>
    <div class="toolbar-right">
      <IconButton icon={compactView ? LayoutGrid : List} label={compactView ? $_('dashboard.card_view') : $_('dashboard.compact_view')} variant="secondary" size="sm" onclick={toggleView} />
      <Button variant="secondary" size="sm" onclick={handleScan} loading={$isLoading}>
        <RefreshCw size={14} />
        {$_('dashboard.scan')}
      </Button>
    </div>
  </div>

  <div class="repo-list" class:compact={compactView}>
    {#if $isLoading}
      {#each Array(4) as _, i (i)}
        <Skeleton variant="card" />
      {/each}
    {:else if $filteredRepos.length === 0}
      <EmptyState
        icon={GitBranch}
        title={$_('dashboard.no_repos')}
        description={$_('dashboard.no_repos_hint')}
        action={{ label: $_('dashboard.add_repos'), onClick: () => currentPage.set('guidedsetup') }}
      />
    {:else}
      {#each $filteredRepos as repo (repo.id)}
        {#if compactView}
          <Card hoverable padding="sm" class="compact-card" onclick={() => handleSelectRepo(repo.id)}>
            <div class="compact-row">
              <span class="compact-name">{repo.name}</span>
              <span class="compact-path">{repo.path}</span>
              <div class="compact-meta">
                <StatusPill status={repo.dirty_state === 'clean' ? 'clean' : 'modified'} label={repo.dirty_state} />
                {#if repo.capability_counts.skill}<Badge variant="default">{repo.capability_counts.skill}</Badge>{/if}
                {#if repo.capability_counts.mcp}<Badge variant="default">{repo.capability_counts.mcp}</Badge>{/if}
              </div>
            </div>
          </Card>
        {:else}
          <Card hoverable padding="md" class="repo-card" onclick={() => handleSelectRepo(repo.id)}>
            {#snippet title()}
              <div class="repo-card-header">
                <span class="repo-name">{repo.name}</span>
                <StatusPill status={repo.dirty_state === 'clean' ? 'clean' : 'modified'} label={repo.dirty_state} />
              </div>
            {/snippet}
            <div class="repo-card-body">
              <p class="repo-path" title={repo.path}>{repo.path}</p>
              <div class="repo-meta">
                {#if repo.branch}
                  <Badge variant="info">{repo.branch}</Badge>
                {/if}
                {#if repo.capability_counts.skill}
                  <Badge variant="default">{$_('repo.skills_count', { values: { n: repo.capability_counts.skill } })}</Badge>
                {/if}
                {#if repo.capability_counts.mcp}
                  <Badge variant="default">{$_('repo.mcp_count', { values: { n: repo.capability_counts.mcp } })}</Badge>
                {/if}
              </div>
            </div>
          </Card>
        {/if}
      {/each}
    {/if}
  </div>
</div>

<style>
  .dashboard {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding-top: var(--space-4);
  }
  .metric-row {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
    gap: var(--space-3);
  }
  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
  }
  .toolbar-left {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    flex: 1;
  }
  .toolbar-right {
    flex-shrink: 0;
  }
  .repo-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }
  .repo-list.compact {
    gap: 1px;
  }
  .compact-card { cursor: pointer; }
  .compact-row {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    font-size: var(--font-size-sm);
  }
  .compact-name {
    font-weight: 500;
    color: var(--text-primary);
    min-width: 150px;
  }
  .compact-path {
    color: var(--text-muted);
    font-family: var(--font-mono);
    font-size: var(--font-size-xs);
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .compact-meta {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    flex-shrink: 0;
  }
  .repo-card {
    cursor: pointer;
  }
  .repo-card-header {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    width: 100%;
  }
  .repo-name {
    font-size: var(--font-size-md);
    font-weight: 600;
    color: var(--text-primary);
    flex: 1;
  }
  .repo-card-body {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }
  .repo-path {
    font-size: var(--font-size-xs);
    font-family: var(--font-mono);
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    margin: 0;
  }
  .repo-meta {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1);
  }
</style>
