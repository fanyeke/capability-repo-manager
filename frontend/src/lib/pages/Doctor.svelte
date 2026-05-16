<script lang="ts">
  import { _ } from 'svelte-i18n';
  import { currentPage, navigateTo } from '$lib/stores/uiStore';
  import { repos, loadRepos, selectedRepoId } from '$lib/stores/repoStore';
  import { runDoctor, report, isLoading, error, clearReport } from '$lib/stores/doctorStore';
  import DoctorReportComp from '$lib/components/DoctorReport.svelte';

  let selectedRepo = $state<string | null>(null);

  loadRepos();

  async function handleRunDoctor() {
    if (!selectedRepo) return;
    await runDoctor(selectedRepo);
  }
</script>

<div class="doctor-page">
  <header class="page-header">
    <h1>{$_('doctor.title')}</h1>
    <button class="back-btn" onclick={() => navigateTo('dashboard')}>{$_('nav.back')}</button>
  </header>

  <main class="doctor-content">
    <div class="select-section">
      <h2>{$_('doctor.select_repo')}</h2>
      <select bind:value={selectedRepo}>
        <option value="">{$_('doctor.choose_repo')}</option>
        {#each $repos as repo (repo.id)}
          <option value={repo.id}>{repo.name}</option>
        {/each}
      </select>
      <button class="run-btn" onclick={handleRunDoctor} disabled={!selectedRepo || $isLoading}>
        {#if $isLoading}{$_('doctor.running')}{:else}{$_('doctor.run')}{/if}
      </button>
    </div>

    {#if $error}
      <div class="error-box">
        <p>{$error}</p>
        <button onclick={() => error.set(null)}>{$_('doctor.dismiss')}</button>
      </div>
    {/if}

    {#if $report}
      <div class="report-section">
        <h2>{$_('doctor.health_report', { values: { name: selectedRepo } })}</h2>
        <DoctorReportComp report={$report} />
        <div class="report-actions">
          <button class="nav-btn" onclick={() => currentPage.set('repo:' + selectedRepo)}
            >{$_('doctor.view_repo')}</button
          >
          <button class="nav-btn" onclick={clearReport}>{$_('doctor.run_again')}</button>
        </div>
      </div>
    {:else}
      <div class="doctor-info">
        <h3>What does Doctor check?</h3>
        <ul>
          <li><strong>Skill Structure</strong> — {$_('doctor.check_skill_structure')}</li>
          <li><strong>Hook Targets</strong> — {$_('doctor.check_hook_targets')}</li>
          <li><strong>Env Placeholders</strong> — {$_('doctor.check_env_placeholders')}</li>
          <li><strong>MCP Config</strong> — {$_('doctor.check_mcp_config')}</li>
        </ul>
        <p class="score-explanation">
          {$_('doctor.score_explanation')}
        </p>
      </div>
    {/if}
  </main>
</div>

<style>
  .doctor-page {
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
  .doctor-content {
    flex: 1;
    padding: 24px;
    background: #f8fafc;
    max-width: 800px;
    margin: 0 auto;
  }
  .select-section {
    background: #fff;
    padding: 20px;
    border-radius: 8px;
    border: 1px solid #e2e8f0;
    margin-bottom: 24px;
  }
  .select-section h2 {
    margin: 0 0 12px 0;
    font-size: 1rem;
  }
  .select-section select {
    width: 100%;
    padding: 10px;
    border: 1px solid #e2e8f0;
    border-radius: 6px;
    font-size: 0.9rem;
    box-sizing: border-box;
    margin-bottom: 12px;
  }
  .run-btn {
    padding: 12px 24px;
    background: var(--primary, #3b82f6);
    color: #fff;
    border: none;
    border-radius: 8px;
    font-weight: 600;
    cursor: pointer;
  }
  .run-btn:disabled {
    opacity: 0.6;
  }
  .error-box {
    background: #fee2e2;
    padding: 12px;
    border-radius: 6px;
    margin-bottom: 24px;
  }
  .error-box p {
    margin: 0 0 8px 0;
    color: #991b1b;
  }
  .error-box button {
    padding: 4px 12px;
    background: #fff;
    border: 1px solid #fecaca;
    border-radius: 4px;
    font-size: 0.8rem;
  }
  .report-section {
    background: #fff;
    padding: 24px;
    border-radius: 8px;
    border: 1px solid #e2e8f0;
  }
  .report-section h2 {
    margin: 0 0 16px 0;
    font-size: 1rem;
  }
  .report-actions {
    display: flex;
    gap: 8px;
    margin-top: 24px;
  }
  .nav-btn {
    padding: 8px 16px;
    background: #fff;
    border: 1px solid #e2e8f0;
    border-radius: 6px;
    cursor: pointer;
  }
  .doctor-info {
    background: #fff;
    padding: 20px;
    border-radius: 8px;
    border: 1px solid #e2e8f0;
  }
  .doctor-info h3 {
    margin: 0 0 12px 0;
    font-size: 0.95rem;
  }
  .doctor-info ul {
    margin: 0;
    padding-left: 20px;
  }
  .doctor-info li {
    font-size: 0.85rem;
    color: #475569;
    margin-bottom: 6px;
  }
  .score-explanation {
    margin-top: 16px;
    font-size: 0.8rem;
    color: #64748b;
    background: #f8fafc;
    padding: 8px;
    border-radius: 4px;
  }
</style>
