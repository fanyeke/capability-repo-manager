<script lang="ts">
  import { currentPage, navigateToRepo } from '$lib/stores/uiStore';
  import { repos, filteredRepos, scanRepositories, loadRepos, selectRepo, isLoading, repoFilter } from '$lib/stores/repoStore';
  import { _ } from 'svelte-i18n';
  import { onMount } from 'svelte';
  import { RefreshCw, GitBranch, AlertTriangle, Shield, Pin, Activity, ArrowRight } from 'lucide-svelte';
  import Button from '$lib/components/Button.svelte';
  import IconButton from '$lib/components/IconButton.svelte';
  import Card from '$lib/components/Card.svelte';
  import MetricCard from '$lib/components/MetricCard.svelte';
  import MetricGrid from '$lib/components/MetricGrid.svelte';
  import SearchInput from '$lib/components/SearchInput.svelte';
  import Select from '$lib/components/Select.svelte';
  import Badge from '$lib/components/Badge.svelte';
  import StatusPill from '$lib/components/StatusPill.svelte';
  import Skeleton from '$lib/components/Skeleton.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import Section from '$lib/components/Section.svelte';

  onMount(() => { isLoading.set(false); loadRepos(); });

  async function handleScan() {
    const { invoke } = await import('@tauri-apps/api/core');
    const settings = await invoke<{ scan_roots: string[] }>('get_settings');
    if (settings.scan_roots.length > 0) {
      scanRepositories(settings.scan_roots).catch(() => {});
    } else { currentPage.set('guidedsetup'); }
  }

  function handleSelectRepo(repoId: string) {
    selectRepo(repoId);
    navigateToRepo(repoId);
  }

  function setSearch(val: string) {
    repoFilter.set({ ...$repoFilter, search: val || undefined });
  }

  let compactView = $state(false);
  const totalRepos = $derived($repos.length);
  const dirtyRepos = $derived($repos.filter((r) => r.dirty_state !== 'clean').length);
  const pinnedRepos = $derived($repos.filter((r) => r.pinned));
  const failedRepos = $derived($repos.filter((r) => r.capability_index_status === 'parse_failed'));
  const avgDoctor = $derived($repos.length > 0 ? Math.round($repos.reduce((s, r) => s + (r.doctor_score ?? 0), 0) / $repos.length) : 0);
  const totalCaps = $derived($repos.reduce((s, r) => { const c = r.capability_counts; return s + (c.skill ?? 0) + (c.mcp ?? 0) + (c.hook ?? 0) + (c.rule ?? 0) + (c.agent ?? 0); }, 0));
  const sortOptions = [
    { value: 'name', label: $_('common.sort_name') },
    { value: 'last_indexed_at', label: $_('common.sort_date') },
    { value: 'dirty_state', label: $_('common.sort_state') },
  ];
</script>

<div class="dashboard">
  <MetricGrid>
    <MetricCard label={$_('dashboard.total_repos')} value={totalRepos} icon={GitBranch} />
    <MetricCard label={$_('dashboard.modified')} value={dirtyRepos} icon={AlertTriangle} trend={dirtyRepos > 0 ? 'up' : 'neutral'} />
    <MetricCard label={$_('dashboard.total_caps')} value={totalCaps} icon={Activity} />
    <MetricCard label={$_('dashboard.avg_doctor')} value={avgDoctor} icon={Shield} trend={avgDoctor >= 80 ? 'up' : avgDoctor >= 50 ? 'neutral' : 'down'} />
  </MetricGrid>

  <div class="toolbar">
    <div class="toolbar-left">
      <SearchInput value={$repoFilter.search ?? ''} placeholder={$_('common.search')} onInput={setSearch} />
      <Select options={sortOptions} value={$repoFilter.sort_by ?? 'name'} onChange={(val) => repoFilter.set({ ...$repoFilter, sort_by: val as any })} />
    </div>
    <div class="toolbar-right">
      <IconButton icon={compactView ? GitBranch : Activity} label={compactView ? $_('dashboard.card_view') : $_('dashboard.compact_view')} variant="secondary" size="sm" onclick={() => compactView = !compactView} />
      <Button variant="secondary" size="sm" onclick={handleScan} loading={$isLoading}><RefreshCw size={14} /> {$_('dashboard.scan')}</Button>
    </div>
  </div>

  {#if $isLoading}
    <div class="loading-grid"><Skeleton variant="card" /><Skeleton variant="card" /><Skeleton variant="card" /></div>
  {:else if $filteredRepos.length === 0}
    <EmptyState icon={GitBranch} title={$_('dashboard.no_repos')} description={$_('dashboard.no_repos_hint')} action={{ label: $_('dashboard.add_repos'), onClick: () => currentPage.set('guidedsetup') }} />
  {:else}
    <div class="workspace">
      <div class="workspace-left">
        {#if pinnedRepos.length > 0}
          <Section title={$_('dashboard.pinned')}>
            <div class="mini-list">{#each pinnedRepos as repo (repo.id)}<button class="mini-item" onclick={() => handleSelectRepo(repo.id)}><Pin size={12} /> <span class="mini-name">{repo.name}</span><StatusPill status={repo.dirty_state === 'clean' ? 'clean' : 'modified'} /></button>{/each}</div>
          </Section>
        {/if}
        {#if failedRepos.length > 0}
          <Section title={$_('dashboard.needs_attention')}>
            <div class="mini-list">{#each failedRepos as repo (repo.id)}<button class="mini-item warning" onclick={() => handleSelectRepo(repo.id)}><AlertTriangle size={12} /> <span class="mini-name">{repo.name}</span><Badge variant="danger">parse error</Badge></button>{/each}</div>
          </Section>
        {/if}
        {#if dirtyRepos > 0}
          <Section title={$_('dashboard.recently_modified')}>
            <div class="mini-list">{#each $repos.filter((r) => r.dirty_state !== 'clean').slice(0, 5) as repo (repo.id)}<button class="mini-item" onclick={() => handleSelectRepo(repo.id)}><span class="mini-name">{repo.name}</span><StatusPill status="modified" /></button>{/each}</div>
          </Section>
        {/if}
      </div>
      <div class="workspace-right">
        <Card padding="md">
          {#snippet title()}{$_('dashboard.health_overview')}{/snippet}
          <div class="health-stats"><div class="health-row"><span>{$_('repo.capabilities')}</span><strong>{totalCaps}</strong></div><div class="health-row"><span>{$_('dashboard.doctor_avg')}</span><strong>{avgDoctor}%</strong></div><div class="health-row"><span>{$_('dashboard.with_issues')}</span><strong>{failedRepos.length + dirtyRepos}</strong></div></div>
          <div class="health-cta"><Button variant="secondary" size="sm" onclick={() => currentPage.set('doctor')}><Shield size={14} /> {$_('doctor.run')} <ArrowRight size={14} /></Button></div>
        </Card>
      </div>
    </div>

    <Section title={$_('dashboard.all_repos')}>
      <div class="repo-list" class:compact={compactView}>
        {#each $filteredRepos as repo (repo.id)}
          {#if compactView}
            <Card hoverable padding="sm" class="compact-card" onclick={() => handleSelectRepo(repo.id)}>
              <div class="compact-row">{#if repo.pinned}<Pin size={10} />{/if}<span class="compact-name">{repo.name}</span><span class="compact-path">{repo.path}</span><div class="compact-meta"><StatusPill status={repo.dirty_state === 'clean' ? 'clean' : 'modified'} />{#if repo.doctor_score !== null && repo.doctor_score < 80}<Badge variant="warning">{repo.doctor_score}</Badge>{/if}</div></div>
            </Card>
          {:else}
            <Card hoverable padding="md" onclick={() => handleSelectRepo(repo.id)}>
              {#snippet title()}<div class="repo-card-header">{#if repo.pinned}<Pin size={12} />{/if}<span class="repo-name">{repo.name}</span><StatusPill status={repo.dirty_state === 'clean' ? 'clean' : 'modified'} label={repo.dirty_state} /></div>{/snippet}
              <p class="repo-path">{repo.path}</p>
              <div class="repo-meta">{#if repo.branch}<Badge variant="info">{repo.branch}</Badge>{/if}{#if repo.doctor_score !== null}<Badge variant={repo.doctor_score >= 80 ? 'success' : 'warning'}>{$_('doctor.score')}: {repo.doctor_score}</Badge>{/if}{#if repo.capability_index_status === 'parse_failed'}<Badge variant="danger">{$_('repo.parse_failed')}</Badge>{/if}</div>
            </Card>
          {/if}
        {/each}
      </div>
    </Section>
  {/if}
</div>

<style>
  .dashboard { display: flex; flex-direction: column; gap: var(--space-4); padding-top: var(--space-4); }
  .toolbar { display: flex; align-items: center; justify-content: space-between; gap: var(--space-3); }
  .toolbar-left { display: flex; align-items: center; gap: var(--space-2); flex: 1; }
  .toolbar-right { display: flex; gap: var(--space-2); flex-shrink: 0; }
  .loading-grid { display: flex; flex-direction: column; gap: var(--space-3); }
  .workspace { display: grid; grid-template-columns: 1fr 240px; gap: var(--space-4); }
  .workspace-left { display: flex; flex-direction: column; gap: var(--space-4); }
  .workspace-right { display: flex; flex-direction: column; gap: var(--space-3); }
  .mini-list { display: flex; flex-direction: column; gap: 2px; }
  .mini-item { display: flex; align-items: center; gap: var(--space-2); padding: var(--space-1) var(--space-2); background: none; border: none; border-radius: var(--radius-sm); color: var(--text-secondary); font-size: var(--font-size-sm); cursor: pointer; width: 100%; text-align: left; }
  .mini-item:hover { background: var(--bg-hover); color: var(--text-primary); }
  .mini-item.warning { color: var(--color-warning); }
  .mini-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .health-stats { display: flex; flex-direction: column; gap: var(--space-2); margin-bottom: var(--space-3); }
  .health-row { display: flex; justify-content: space-between; font-size: var(--font-size-sm); color: var(--text-secondary); }
  .repo-list { display: flex; flex-direction: column; gap: var(--space-2); }
  .repo-list.compact { gap: 1px; }
  .compact-card { cursor: pointer; }
  .compact-row { display: flex; align-items: center; gap: var(--space-3); font-size: var(--font-size-sm); }
  .compact-name { font-weight: 500; color: var(--text-primary); min-width: 150px; }
  .compact-path { color: var(--text-muted); font-family: var(--font-mono); font-size: var(--font-size-xs); flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .compact-meta { display: flex; align-items: center; gap: var(--space-2); flex-shrink: 0; }
  .repo-card-header { display: flex; align-items: center; gap: var(--space-2); width: 100%; }
  .repo-name { font-size: var(--font-size-md); font-weight: 600; color: var(--text-primary); flex: 1; }
  .repo-path { font-size: var(--font-size-xs); font-family: var(--font-mono); color: var(--text-muted); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; margin: 0; }
  .repo-meta { display: flex; flex-wrap: wrap; gap: var(--space-1); }
</style>
