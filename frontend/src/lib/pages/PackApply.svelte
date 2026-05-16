<script lang="ts">
  import { _ } from 'svelte-i18n';
  import { currentPage, navigateTo } from '$lib/stores/uiStore';
  import { repos, loadRepos } from '$lib/stores/repoStore';
  import { packs, loadPacks, selectedPackId, isLoading } from '$lib/stores/packStore';
  import {
    plan,
    report,
    buildMigrationPlan,
    applyMigrationPlan,
    executionStatus,
    strategies,
    setStrategy,
  } from '$lib/stores/migrationStore';
  import { CheckCircle, AlertTriangle, AlertCircle, ArrowLeft } from 'lucide-svelte';
  import Button from '$lib/components/Button.svelte';
  import Card from '$lib/components/Card.svelte';
  import Banner from '$lib/components/Banner.svelte';
  import StatusPill from '$lib/components/StatusPill.svelte';
  import MigrationPlanComp from '$lib/components/MigrationPlan.svelte';
  import PackLibrary from '$lib/components/PackLibrary.svelte';

  let selectedPack = $state<string | null>(null);
  let selectedTarget = $state<string | null>(null);
  let step = $state<'select' | 'plan' | 'executing' | 'report'>('select');

  async function handleBuildPlan() {
    if (!selectedPack || !selectedTarget) return;
    await buildMigrationPlan(selectedPack, selectedTarget);
    step = 'plan';
  }

  async function handleExecute() {
    step = 'executing';
    await applyMigrationPlan();
    step = 'report';
  }

  function handleReset() {
    selectedPack = null;
    selectedTarget = null;
    step = 'select';
  }

  const hasUnresolvedConflicts = $derived(
    $plan?.conflicts.some((c) => {
      const strategy = $strategies.find((s) => c.resource_name === s.resource_id.split('-')[0]);
      return !strategy;
    }) ?? false,
  );

  const hasMissingDeps = $derived(
    ($plan?.missing_dependencies.length ?? 0) > 0,
  );

  loadRepos();
  loadPacks();
</script>

<div class="pack-apply-page">
  <header class="page-header">
    <h1>{$_('pack_apply.title')}</h1>
    <button class="back-btn" onclick={() => navigateTo('dashboard')}>{$_('nav.back')}</button>
  </header>

  <main class="apply-content">
    {#if step === 'select'}
      <div class="selection-section">
        <div class="select-panel">
          <h2>{$_('pack_apply.select_pack')}</h2>
          {#if $isLoading}
            <p class="loading">{$_('pack_apply.loading_packs')}</p>
          {:else if $packs.length === 0}
            <p class="empty-text">{$_('pack_apply.no_packs')}</p>
          {:else}
            <select bind:value={selectedPack}>
              <option value="">{$_('pack_apply.choose_pack')}</option>
              {#each $packs as pack (pack.id)}
                <option value={pack.id}
                  >{pack.name} v{pack.version} ({$_('pack.resources_count', {
                    values: { n: pack.resource_count },
                  })})</option
                >
              {/each}
            </select>
          {/if}
        </div>
        <div class="select-panel">
          <h2>{$_('pack_apply.select_target')}</h2>
          {#if $isLoading}
            <p class="loading">{$_('pack_apply.loading_repos')}</p>
          {:else}
            <select bind:value={selectedTarget}>
              <option value="">{$_('pack_apply.choose_repo')}</option>
              {#each $repos as repo (repo.id)}
                <option value={repo.id}>{repo.name} — {repo.path}</option>
              {/each}
            </select>
          {/if}
        </div>
        <div class="select-actions">
          <button class="primary-btn" onclick={handleBuildPlan}
            disabled={!selectedPack || !selectedTarget || $isLoading}>
            {$_('pack_apply.build_plan')}
          </button>
        </div>
      </div>
    {:else if step === 'plan'}
      <div class="plan-section">
        <h2>{$_('pack_apply.plan_title')}</h2>
        <p class="plan-desc">{$_('pack_apply.plan_desc')}</p>
        <MigrationPlanComp plan={$plan} strategies={$strategies} onSetStrategy={setStrategy} />
        <div class="plan-actions">
          <button class="back-btn" onclick={handleReset}>{$_('pack_apply.back')}</button>
          <button class="execute-btn" onclick={handleExecute} disabled={$isLoading}>
            {#if $isLoading}{$_('pack_apply.executing')}{:else}{$_('pack_apply.execute')}{/if}
          </button>
        </div>
      </div>
    {:else if step === 'executing'}
      <Card padding="lg">
        <StatusPill status="info" label={$_('pack_apply.executing')} />
        <p>{$_('pack_apply.executing_desc')}</p>
      </Card>
    {:else if step === 'report'}
      <div class="report-section">
        <Banner type={$executionStatus === 'success' ? 'success' : $executionStatus === 'partial' ? 'warning' : 'error'} dismissible={false}>
          {$_('pack_apply.report_title', { values: { status: $executionStatus } })}
        </Banner>
        {#if $report}
          <Card padding="md">
            {#snippet title()}{$_('pack_apply.report_summary')}{/snippet}
            <div class="report-stats">
              <span class="stat">+{$report.summary.added}</span>
              <span class="stat">~{$report.summary.overwritten}</span>
              <span class="stat">−{$report.summary.skipped}</span>
              <span class="stat">!{$report.summary.failed}</span>
            </div>
          </Card>
        {/if}
        <div class="report-actions">
          <Button variant="primary" onclick={handleReset}>{$_('pack_apply.apply_another')}</Button>
          <Button variant="ghost" onclick={() => currentPage.set('doctor')}>{$_('doctor.run')}</Button>
          {#if $executionStatus === 'failed' || $executionStatus === 'partial'}
            <Button variant="secondary" onclick={() => step = 'plan'}><ArrowLeft size={14} /> {$_('pack_apply.back_to_plan')}</Button>
          {/if}
        </div>
      </div>
    {/if}
  </main>
</div>

<style>
  .pack-apply-page {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
  }
  .page-header {
    display: flex;
    justify-content: space-between;
    padding: var(--space-4) var(--space-6);
    border-bottom: 1px solid var(--border-default);
    background: var(--bg-card);
  }
  .page-header h1 {
    margin: 0;
  }
  .back-btn {
    padding: var(--space-2) var(--space-4);
    background: var(--bg-card);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    cursor: pointer;
  }
  .apply-content {
    flex: 1;
    padding: var(--space-6);
    background: var(--bg-elevated);
  }
  .selection-section {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-6);
    max-width: 800px;
    margin: 0 auto;
  }
  .select-panel {
    background: var(--bg-card);
    padding: var(--space-5);
    border-radius: var(--radius-md);
    border: 1px solid var(--border-default);
  }
  .select-panel h2 {
    margin: 0 0 var(--space-3) 0;
    font-size: var(--font-size-lg);
  }
  .select-panel select {
    width: 100%;
    padding: 10px;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    font-size: var(--font-size-md);
    box-sizing: border-box;
  }
  .select-actions {
    display: flex;
    justify-content: center;
    margin-top: var(--space-6);
  }
  .primary-btn {
    padding: var(--space-3) var(--space-8);
    background: var(--color-primary);
    color: var(--text-primary);
    border: none;
    border-radius: var(--radius-md);
    font-weight: 600;
    cursor: pointer;
  }
  .primary-btn:disabled {
    opacity: 0.6;
  }
  .plan-section,
  .done-section {
    max-width: 900px;
    margin: 0 auto;
    background: var(--bg-card);
    padding: var(--space-6);
    border-radius: var(--radius-md);
    border: 1px solid var(--border-default);
  }
  .plan-section h2,
  .done-section h2 {
    margin: 0 0 var(--space-3) 0;
  }
  .plan-desc {
    color: var(--text-muted);
    margin-bottom: var(--space-4);
  }
  .plan-actions,
  .done-actions {
    display: flex;
    gap: var(--space-3);
    margin-top: var(--space-6);
    justify-content: flex-end;
  }
  .execute-btn {
    padding: var(--space-3) var(--space-6);
    background: var(--color-success);
    color: var(--text-primary);
    border: none;
    border-radius: var(--radius-md);
    font-weight: 600;
    cursor: pointer;
  }
  .execute-btn:disabled {
    opacity: 0.6;
  }
  .nav-btn {
    padding: var(--space-2) var(--space-4);
    background: var(--bg-card);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    cursor: pointer;
  }
  .report-summary {
    background: var(--bg-elevated);
    padding: var(--space-4);
    border-radius: var(--radius-md);
  }
  .report-stats {
    display: flex;
    gap: var(--space-3);
    margin-top: var(--space-2);
  }
  .stat {
    padding: var(--space-1) var(--space-3);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-sm);
  }
  .stat-added {
    background: var(--color-success-bg);
    color: var(--color-success);
  }
  .stat-overwritten {
    background: var(--color-warning-bg);
    color: var(--color-warning);
  }
  .stat-skipped {
    background: var(--bg-hover);
    color: var(--text-secondary);
  }
  .stat-failed {
    background: var(--color-danger-bg);
    color: var(--color-danger);
  }
  .error-list {
    margin-top: var(--space-4);
    padding: var(--space-3);
    background: var(--color-danger-bg);
    border-radius: var(--radius-md);
  }
  .error-item {
    margin: var(--space-1) 0;
    font-size: var(--font-size-sm);
    color: var(--color-danger);
  }
  .loading,
  .empty-text {
    text-align: center;
    color: var(--text-muted);
    padding: var(--space-4) 0;
  }
</style>
