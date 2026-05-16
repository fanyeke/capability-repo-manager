<script lang="ts">
  import { _ } from 'svelte-i18n';
  import { currentPage, navigateTo } from '$lib/stores/uiStore';
  import { repos, loadRepos } from '$lib/stores/repoStore';
  import { runDoctor, report, isLoading, error, clearReport } from '$lib/stores/doctorStore';
  import { Stethoscope, ArrowLeft, RefreshCw } from 'lucide-svelte';
  import Button from '$lib/components/Button.svelte';
  import Card from '$lib/components/Card.svelte';
  import Banner from '$lib/components/Banner.svelte';
  import Select from '$lib/components/Select.svelte';
  import Skeleton from '$lib/components/Skeleton.svelte';
  import DoctorReportComp from '$lib/components/DoctorReport.svelte';

  let selectedRepo = $state<string | null>(null);

  loadRepos();

  async function handleRunDoctor() {
    if (!selectedRepo) return;
    await runDoctor(selectedRepo);
  }

  const repoOptions = $derived(
    $repos.map((r) => ({ value: r.id, label: r.name })),
  );
</script>

<div class="doctor-page">
  <div class="page-top">
    <Button variant="ghost" size="sm" onclick={() => navigateTo('dashboard')}>
      <ArrowLeft size={14} /> {$_('nav.back')}
    </Button>
    <h1 class="page-title">{$_('doctor.title')}</h1>
  </div>

  <Card padding="md">
    {#snippet title()}{$_('doctor.select_repo')}{/snippet}
    <div class="select-row">
      <Select
        options={[{ value: '', label: $_('doctor.choose_repo') }, ...repoOptions]}
        value={selectedRepo ?? ''}
        onChange={(val) => selectedRepo = val || null}
      />
      <Button
        variant="primary"
        onclick={handleRunDoctor}
        disabled={!selectedRepo || $isLoading}
        loading={$isLoading}
      >
        <Stethoscope size={14} />
        {$isLoading ? $_('doctor.running') : $_('doctor.run')}
      </Button>
    </div>
  </Card>

  {#if $error}
    <Banner type="error" onDismiss={() => error.set(null)} dismissible={true}>
      {$error}
    </Banner>
  {/if}

  {#if $isLoading && !$report}
    <Skeleton variant="card" />
    <Skeleton variant="text" />
  {:else if $report}
    <Card padding="md">
      {#snippet title()}{$_('doctor.health_report', { values: { name: selectedRepo ?? '' } })}{/snippet}
      <DoctorReportComp report={$report} />
      <div class="report-actions">
        <Button variant="secondary" size="sm" onclick={() => currentPage.set('repo:' + selectedRepo)}>
          {$_('doctor.view_repo')}
        </Button>
        <Button variant="ghost" size="sm" onclick={clearReport}>
          <RefreshCw size={14} /> {$_('doctor.run_again')}
        </Button>
      </div>
    </Card>
  {:else}
    <Card padding="md">
      {#snippet title()}What does Doctor check?{/snippet}
      <ul class="check-list">
        <li><strong>Skill Structure</strong> — {$_('doctor.check_skill_structure')}</li>
        <li><strong>Hook Targets</strong> — {$_('doctor.check_hook_targets')}</li>
        <li><strong>Env Placeholders</strong> — {$_('doctor.check_env_placeholders')}</li>
        <li><strong>MCP Config</strong> — {$_('doctor.check_mcp_config')}</li>
      </ul>
      <div class="score-note">{$_('doctor.score_explanation')}</div>
    </Card>
  {/if}
</div>

<style>
  .doctor-page {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding-top: var(--space-4);
    max-width: 800px;
  }
  .page-top {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }
  .page-title {
    margin: 0;
    font-size: var(--font-size-xl);
  }
  .select-row {
    display: flex;
    gap: var(--space-2);
    align-items: flex-start;
  }
  .report-actions {
    display: flex;
    gap: var(--space-2);
    margin-top: var(--space-4);
  }
  .check-list {
    margin: 0;
    padding-left: var(--space-4);
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    line-height: 1.8;
  }
  .score-note {
    margin-top: var(--space-3);
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    background: var(--bg-elevated);
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-sm);
  }
</style>
