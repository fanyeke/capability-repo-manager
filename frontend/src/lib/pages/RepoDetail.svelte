<script lang="ts">
  import { _ } from 'svelte-i18n';
  import { currentPage, navigateTo } from '$lib/stores/uiStore';
  import { selectedRepoDetail, refreshRepository, removeRepository, isLoading } from '$lib/stores/repoStore';
  import { resources, typeGroups, selectedType, filteredResources, loadCapabilityInventory, setTypeFilter } from '$lib/stores/capabilityStore';
  import { ArrowLeft, RefreshCw, Trash2, Package, Stethoscope } from 'lucide-svelte';
  import Button from '$lib/components/Button.svelte';
  import Card from '$lib/components/Card.svelte';
  import MetricCard from '$lib/components/MetricCard.svelte';
  import Tabs from '$lib/components/Tabs.svelte';
  import Badge from '$lib/components/Badge.svelte';
  import StatusPill from '$lib/components/StatusPill.svelte';
  import Skeleton from '$lib/components/Skeleton.svelte';
  import ResourceIcon from '$lib/components/ResourceIcon.svelte';
  import ResourceDetail from '$lib/components/ResourceDetail.svelte';

  let { repoId }: { repoId: string } = $props();

  let activeTab = $state<'overview' | 'capabilities'>('overview');
  let selectedResourceId = $state<string | null>(null);
  let detail = $derived($selectedRepoDetail);

  $effect(() => {
    if (detail?.repo.id) {
      loadCapabilityInventory(detail.repo.id);
    }
  });

  const capabilityTabs = [
    { id: 'overview', label: $_('repo.overview') },
    { id: 'capabilities', label: $_('repo.capabilities_tab') },
  ];

  function totalCapabilities() {
    if (!$resources) return 0;
    let count = 0;
    for (const group of Object.values($resources)) {
      count += (group as any[]).length;
    }
    return count;
  }
</script>

<div class="repo-detail">
  {#if !detail}
    <div class="loading-state">
      <Skeleton variant="card" />
      <Skeleton variant="card" />
      <Skeleton variant="card" />
    </div>
  {:else}
    <div class="repo-header">
      <div class="header-top">
        <Button variant="ghost" size="sm" onclick={() => navigateTo('dashboard')}>
          <ArrowLeft size={14} />
          {$_('repo.back')}
        </Button>
      </div>
      <div class="header-meta">
        <div class="header-text">
          <h1 class="repo-title">{detail.repo.name}</h1>
          <p class="repo-path">{detail.repo.path}</p>
          <div class="repo-tags">
            {#if detail.repo.current_branch}
              <StatusPill status="info" label={detail.repo.current_branch} />
            {/if}
            <StatusPill
              status={detail.repo.dirty_state === 'clean' ? 'clean' : 'modified'}
              label={detail.repo.dirty_state}
            />
          </div>
        </div>
        <div class="header-actions">
          <Button variant="secondary" size="sm" onclick={() => refreshRepository(repoId)} disabled={$isLoading}>
            <RefreshCw size={14} />
            {$_('repo.refresh')}
          </Button>
          <Button variant="secondary" size="sm" onclick={() => currentPage.set('packexport')}>
            <Package size={14} />
            {$_('repo.export_pack')}
          </Button>
          <Button variant="secondary" size="sm" onclick={() => currentPage.set('doctor')}>
            <Stethoscope size={14} />
            {$_('repo.run_doctor')}
          </Button>
          <Button variant="danger" size="sm" onclick={() => removeRepository(repoId)} disabled={$isLoading}>
            <Trash2 size={14} />
            {$_('repo.remove')}
          </Button>
        </div>
      </div>
      <div class="metric-row">
        <MetricCard label={$_('repo.total_capabilities')} value={totalCapabilities()} icon={Package} />
        <MetricCard label="MCP" value={$resources?.mcp?.length ?? 0} />
        <MetricCard label="Hooks" value={$resources?.hooks?.length ?? 0} />
        <MetricCard label="Rules" value={$resources?.rules?.length ?? 0} />
      </div>
    </div>

    <Tabs tabs={capabilityTabs} active={activeTab} onChange={(id) => activeTab = id as 'overview' | 'capabilities'} />

    {#if activeTab === 'overview'}
      <div class="overview-grid">
        <Card padding="md">
          {#snippet title()}{$_('repo.repo_info')}{/snippet}
          <dl class="info-list">
            <dt>{$_('repo.name')}</dt>
            <dd>{detail.repo.name}</dd>
            <dt>{$_('repo.path')}</dt>
            <dd class="mono">{detail.repo.path}</dd>
            <dt>{$_('repo.remote')}</dt>
            <dd>{detail.repo.remote_url ?? 'N/A'}</dd>
            <dt>{$_('repo.branch')}</dt>
            <dd>{detail.repo.current_branch ?? 'N/A'}</dd>
            <dt>{$_('repo.last_indexed')}</dt>
            <dd>{new Date(detail.repo.last_indexed_at).toLocaleString()}</dd>
          </dl>
        </Card>
        <Card padding="md">
          {#snippet title()}{$_('repo.capability_summary')}{/snippet}
          <dl class="info-list">
            {#each Object.entries(detail.capabilities) as [type, items]}
              {#if (items as any[]).length > 0}
                <dt>{type}</dt>
                <dd>{(items as any[]).length}</dd>
              {/if}
            {/each}
          </dl>
        </Card>
      </div>
    {:else}
      <div class="capabilities-layout">
        <aside class="cap-sidebar">
          <div class="type-list">
            <button
              class="type-btn"
              class:active={$selectedType === null}
              onclick={() => { setTypeFilter(null); selectedResourceId = null; }}
            >
              <span>{$_('capability.all')}</span>
              <Badge size="sm">{$typeGroups.reduce((s, g) => s + g.count, 0)}</Badge>
            </button>
            {#each $typeGroups as group (group.type)}
              <button
                class="type-btn"
                class:active={$selectedType === group.type}
                onclick={() => { setTypeFilter(group.type); selectedResourceId = null; }}
              >
                <ResourceIcon type={group.type as any} size={14} />
                <span>{group.label}</span>
                <Badge size="sm">{group.count}</Badge>
              </button>
            {/each}
          </div>
          {#if $filteredResources.length > 0}
            <div class="resource-list">
              {#each $filteredResources as resource (resource.id)}
                <button
                  class="resource-btn"
                  class:active={selectedResourceId === resource.id}
                  onclick={() => selectedResourceId = resource.id}
                >
                  <ResourceIcon type={resource.type} size={14} />
                  <span class="resource-name">{resource.name}</span>
                  {#if resource.error_message}
                    <Badge variant="danger" size="sm">!</Badge>
                  {/if}
                </button>
              {/each}
            </div>
          {:else if $selectedType}
            <p class="empty-hint">{$_('capability.no_resources')}</p>
          {/if}
        </aside>
        <div class="cap-content">
          {#if selectedResourceId && $resources}
            {#each Object.values($resources) as group}
              {#each group as resource (resource.id)}
                {#if resource.id === selectedResourceId}
                  <ResourceDetail resource={resource} />
                {/if}
              {/each}
            {/each}
          {:else}
            <p class="select-hint">{$_('repo.select_capability')}</p>
          {/if}
        </div>
      </div>
    {/if}
  {/if}
</div>

<style>
  .repo-detail {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding-top: var(--space-4);
  }
  .loading-state {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    padding-top: var(--space-8);
  }
  .repo-header {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }
  .header-top { display: flex; }
  .header-meta {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: var(--space-4);
  }
  .repo-title {
    font-size: var(--font-size-xl);
    font-weight: 700;
    margin: 0 0 var(--space-1);
  }
  .repo-path {
    font-family: var(--font-mono);
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    margin: 0 0 var(--space-2);
  }
  .repo-tags {
    display: flex;
    gap: var(--space-2);
  }
  .header-actions {
    display: flex;
    gap: var(--space-2);
    flex-shrink: 0;
    flex-wrap: wrap;
  }
  .metric-row {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
    gap: var(--space-3);
  }
  .overview-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: var(--space-4);
  }
  .info-list {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: var(--space-2);
    font-size: var(--font-size-sm);
  }
  .info-list dt {
    color: var(--text-muted);
  }
  .info-list dd {
    color: var(--text-secondary);
    margin: 0;
  }
  .info-list dd.mono {
    font-family: var(--font-mono);
    font-size: var(--font-size-xs);
    word-break: break-all;
  }
  .capabilities-layout {
    display: grid;
    grid-template-columns: 200px 1fr;
    gap: var(--space-4);
    min-height: 400px;
  }
  .cap-sidebar {
    background: var(--bg-card);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    padding: var(--space-2);
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }
  .type-list {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  .type-btn {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-2);
    border: none;
    background: none;
    color: var(--text-secondary);
    font-size: var(--font-size-sm);
    cursor: pointer;
    border-radius: var(--radius-sm);
    text-align: left;
    width: 100%;
    transition: background var(--transition-fast), color var(--transition-fast);
  }
  .type-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }
  .type-btn.active {
    background: var(--color-primary-bg);
    color: var(--color-primary-text);
  }
  .type-btn span { flex: 1; }
  .resource-list {
    display: flex;
    flex-direction: column;
    gap: 1px;
    border-top: 1px solid var(--border-default);
    padding-top: var(--space-1);
    margin-top: var(--space-1);
  }
  .resource-btn {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-1) var(--space-2);
    border: none;
    background: none;
    color: var(--text-secondary);
    font-size: var(--font-size-sm);
    cursor: pointer;
    border-radius: var(--radius-sm);
    text-align: left;
    width: 100%;
    transition: background var(--transition-fast);
  }
  .resource-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }
  .resource-btn.active {
    background: var(--bg-active);
    color: var(--text-primary);
  }
  .resource-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .empty-hint, .select-hint {
    text-align: center;
    color: var(--text-muted);
    padding: var(--space-8);
    font-size: var(--font-size-sm);
  }
  .cap-content {
    background: var(--bg-card);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    padding: var(--space-4);
  }
</style>
