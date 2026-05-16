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
    strategies,
    setStrategy,
  } from '$lib/stores/migrationStore';
  import MigrationPlanComp from '$lib/components/MigrationPlan.svelte';
  import PackLibrary from '$lib/components/PackLibrary.svelte';

  let selectedPack = $state<string | null>(null);
  let selectedTarget = $state<string | null>(null);
  let step = $state<'select' | 'plan' | 'execute' | 'done'>('select');

  async function handleBuildPlan() {
    if (!selectedPack || !selectedTarget) return;
    await buildMigrationPlan(selectedPack, selectedTarget);
    step = 'plan';
  }

  async function handleExecute() {
    await applyMigrationPlan();
    step = 'done';
  }

  function handleReset() {
    selectedPack = null;
    selectedTarget = null;
    step = 'select';
  }

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
          <button
            class="primary-btn"
            onclick={handleBuildPlan}
            disabled={!selectedPack || !selectedTarget || $isLoading}
          >
            {$_('pack_apply.build_plan')}
          </button>
        </div>
      </div>
    {:else if step === 'plan'}
      <div class="plan-section">
        <h2>{$_('pack_apply.plan_title')}</h2>
        <p class="plan-desc">
          {$_('pack_apply.plan_desc')}
        </p>

        <MigrationPlanComp plan={$plan} strategies={$strategies} onSetStrategy={setStrategy} />

        <div class="plan-actions">
          <button class="back-btn" onclick={handleReset}>{$_('pack_apply.back')}</button>
          <button class="execute-btn" onclick={handleExecute} disabled={$isLoading}>
            {#if $isLoading}{$_('pack_apply.executing')}{:else}{$_('pack_apply.execute')}{/if}
          </button>
        </div>
      </div>
    {:else if step === 'done'}
      <div class="done-section">
        {#if $report}
          <h2>{$_('pack_apply.complete_title')}</h2>
          <div class="report-summary">
            <p><strong>{$_('pack_apply.status')}</strong> {$report.status}</p>
            <div class="report-stats">
              <span class="stat stat-added">{$_('pack_apply.added')} {$report.summary.added}</span>
              <span class="stat stat-overwritten"
                >{$_('pack_apply.overwritten')} {$report.summary.overwritten}</span
              >
              <span class="stat stat-skipped"
                >{$_('pack_apply.skipped')} {$report.summary.skipped}</span
              >
              <span class="stat stat-failed"
                >{$_('pack_apply.failed')} {$report.summary.failed}</span
              >
            </div>
          </div>

          {#if $report.status === 'failed'}
            <div class="error-list">
              {#each $report.items.filter((i) => i.status === 'failed') as item (item.resource_name)}
                <p class="error-item">{item.resource_name}: {item.error}</p>
              {/each}
            </div>
          {/if}
        {/if}

        <div class="done-actions">
          <button class="primary-btn" onclick={handleReset}>{$_('pack_apply.apply_another')}</button
          >
          <button class="nav-btn" onclick={() => currentPage.set('doctor')}
            >{$_('doctor.run')}</button
          >
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
    padding: 16px 24px;
    border-bottom: 1px solid #e2e8f0;
    background: #fff;
  }
  .page-header h1 {
    margin: 0;
  }
  .back-btn {
    padding: 8px 16px;
    background: #fff;
    border: 1px solid #e2e8f0;
    border-radius: 6px;
    cursor: pointer;
  }
  .apply-content {
    flex: 1;
    padding: 24px;
    background: #f8fafc;
  }
  .selection-section {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 24px;
    max-width: 800px;
    margin: 0 auto;
  }
  .select-panel {
    background: #fff;
    padding: 20px;
    border-radius: 8px;
    border: 1px solid #e2e8f0;
  }
  .select-panel h2 {
    margin: 0 0 12px 0;
    font-size: 1rem;
  }
  .select-panel select {
    width: 100%;
    padding: 10px;
    border: 1px solid #e2e8f0;
    border-radius: 6px;
    font-size: 0.9rem;
    box-sizing: border-box;
  }
  .select-actions {
    display: flex;
    justify-content: center;
    margin-top: 24px;
  }
  .primary-btn {
    padding: 12px 32px;
    background: var(--primary, #3b82f6);
    color: #fff;
    border: none;
    border-radius: 8px;
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
    background: #fff;
    padding: 24px;
    border-radius: 8px;
    border: 1px solid #e2e8f0;
  }
  .plan-section h2,
  .done-section h2 {
    margin: 0 0 12px 0;
  }
  .plan-desc {
    color: #64748b;
    margin-bottom: 16px;
  }
  .plan-actions,
  .done-actions {
    display: flex;
    gap: 12px;
    margin-top: 24px;
    justify-content: flex-end;
  }
  .execute-btn {
    padding: 12px 24px;
    background: #16a34a;
    color: #fff;
    border: none;
    border-radius: 8px;
    font-weight: 600;
    cursor: pointer;
  }
  .execute-btn:disabled {
    opacity: 0.6;
  }
  .nav-btn {
    padding: 8px 16px;
    background: #fff;
    border: 1px solid #e2e8f0;
    border-radius: 6px;
    cursor: pointer;
  }
  .report-summary {
    background: #f8fafc;
    padding: 16px;
    border-radius: 6px;
  }
  .report-stats {
    display: flex;
    gap: 12px;
    margin-top: 8px;
  }
  .stat {
    padding: 4px 12px;
    border-radius: 4px;
    font-size: 0.85rem;
  }
  .stat-added {
    background: #dcfce7;
    color: #166534;
  }
  .stat-overwritten {
    background: #fef3c7;
    color: #92400e;
  }
  .stat-skipped {
    background: #f1f5f9;
    color: #475569;
  }
  .stat-failed {
    background: #fee2e2;
    color: #991b1b;
  }
  .error-list {
    margin-top: 16px;
    padding: 12px;
    background: #fee2e2;
    border-radius: 6px;
  }
  .error-item {
    margin: 4px 0;
    font-size: 0.85rem;
    color: #991b1b;
  }
  .loading,
  .empty-text {
    text-align: center;
    color: #64748b;
    padding: 16px 0;
  }
</style>
