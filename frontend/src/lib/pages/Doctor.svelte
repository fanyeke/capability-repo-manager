<script lang="ts">
  import { _ } from 'svelte-i18n';
  import { currentPage, navigateTo } from '$lib/stores/uiStore';
  import { repos, loadRepos } from '$lib/stores/repoStore';
  import { runDoctor, report, isLoading, error, clearReport } from '$lib/stores/doctorStore';
  import { ArrowLeft, Stethoscope, RefreshCw, Shield, ShieldAlert, ShieldCheck, AlertTriangle, AlertCircle } from 'lucide-svelte';
  import Button from '$lib/components/Button.svelte';
  import Card from '$lib/components/Card.svelte';
  import Banner from '$lib/components/Banner.svelte';
  import Select from '$lib/components/Select.svelte';
  import Skeleton from '$lib/components/Skeleton.svelte';
  import MetricCard from '$lib/components/MetricCard.svelte';
  import MetricGrid from '$lib/components/MetricGrid.svelte';
  import StatusPill from '$lib/components/StatusPill.svelte';
  import DoctorReportComp from '$lib/components/DoctorReport.svelte';

  let selectedRepo = $state<string | null>(null);
  loadRepos();
  const repoOptions = $derived($repos.map((r) => ({ value: r.id, label: r.name })));

  async function handleRunDoctor() {
    if (!selectedRepo) return;
    await runDoctor(selectedRepo);
  }

  function scoreColor(score: number): 'success' | 'warning' | 'danger' {
    if (score >= 80) return 'success';
    if (score >= 50) return 'warning';
    return 'danger';
  }

  const critical = $derived($report?.issues.filter((i) => i.severity === 'critical') ?? []);
  const warnings = $derived($report?.issues.filter((i) => i.severity === 'warning') ?? []);
  const infos = $derived($report?.issues.filter((i) => i.severity === 'info') ?? []);
</script>

<div class="doctor-page">
  <div class="page-header">
    <Button variant="ghost" size="sm" onclick={() => navigateTo('dashboard')}><ArrowLeft size={14} /> {$_('nav.back')}</Button>
    <h1>{$_('doctor.title')}</h1>
  </div>

  <Card padding="md">
    <div class="selector-row">
      <Select options={[{ value: '', label: $_('doctor.choose_repo') }, ...repoOptions]} value={selectedRepo ?? ''} onChange={(val) => selectedRepo = val || null} />
      <Button variant="primary" onclick={handleRunDoctor} disabled={!selectedRepo || $isLoading} loading={$isLoading}>
        <Stethoscope size={14} /> {$isLoading ? $_('doctor.running') : $_('doctor.run')}
      </Button>
    </div>
  </Card>

  {#if $error}
    <Banner type="error" onDismiss={() => clearReport()} dismissible={true}>{$error}</Banner>
  {/if}

  {#if $isLoading && !$report}
    <MetricGrid><Skeleton variant="card" /><Skeleton variant="card" /><Skeleton variant="card" /><Skeleton variant="card" /></MetricGrid>
    <Skeleton variant="card" />
  {:else if $report}
    <div class="diagnostic-layout">
      <!-- Left: Score + Severity -->
      <div class="diagnostic-left">
        <Card padding="lg">
          <div class="score-ring">
            <svg viewBox="0 0 120 120" class="score-svg">
              <circle cx="60" cy="60" r="52" fill="none" stroke="var(--bg-elevated)" stroke-width="10" />
              <circle cx="60" cy="60" r="52" fill="none" stroke={$report.score >= 80 ? 'var(--color-success)' : $report.score >= 50 ? 'var(--color-warning)' : 'var(--color-danger)'} stroke-width="10" stroke-dasharray={2 * Math.PI * 52} stroke-dashoffset={2 * Math.PI * 52 * (1 - $report.score / 100)} transform="rotate(-90, 60, 60)" stroke-linecap="round" />
            </svg>
            <div class="score-text"><span class="score-value">{$report.score}</span><span class="score-unit">/100</span></div>
          </div>
          <div class="score-label">{$_('doctor.score')}</div>
        </Card>

        <Card padding="md">
          {#snippet title()}{$_('doctor.severity_summary')}{/snippet}
          <div class="severity-list">
            <div class="severity-row"><ShieldAlert size={14} class="sev-critical" /><span>{$_('doctor.critical')}</span><strong>{critical.length}</strong></div>
            <div class="severity-row"><AlertTriangle size={14} class="sev-warning" /><span>{$_('doctor.warning')}</span><strong>{warnings.length}</strong></div>
            <div class="severity-row"><AlertCircle size={14} class="sev-info" /><span>{$_('doctor.info')}</span><strong>{infos.length}</strong></div>
          </div>
        </Card>

        <div class="diagnostic-actions">
          <Button variant="secondary" size="sm" onclick={() => currentPage.set('repo:' + selectedRepo)}><ArrowLeft size={14} /> {$_('doctor.view_repo')}</Button>
          <Button variant="ghost" size="sm" onclick={clearReport}><RefreshCw size={14} /> {$_('doctor.run_again')}</Button>
        </div>
      </div>

      <!-- Right: Issues -->
      <div class="diagnostic-right">
        {#if critical.length > 0}
          <div class="issue-group">
            <h3 class="issue-group-title sev-critical">{$_('doctor.critical')}</h3>
            {#each critical as issue (issue.code)}
              <Card padding="sm" class="issue-card">{#snippet title()}<span class="issue-code">{issue.code}</span><StatusPill status="danger" />{/snippet}<p class="issue-msg">{issue.message}</p>{#if issue.recommendation}<p class="issue-rec">{issue.recommendation}</p>{/if}</Card>
            {/each}
          </div>
        {/if}
        {#if warnings.length > 0}
          <div class="issue-group">
            <h3 class="issue-group-title sev-warning">{$_('doctor.warning')}</h3>
            {#each warnings as issue (issue.code)}
              <Card padding="sm" class="issue-card">{#snippet title()}<span class="issue-code">{issue.code}</span><StatusPill status="warning" />{/snippet}<p class="issue-msg">{issue.message}</p>{#if issue.recommendation}<p class="issue-rec">{issue.recommendation}</p>{/if}</Card>
            {/each}
          </div>
        {/if}
        {#if infos.length > 0}
          <div class="issue-group">
            <h3 class="issue-group-title sev-info">{$_('doctor.info')}</h3>
            {#each infos as issue (issue.code)}
              <Card padding="sm" class="issue-card">{#snippet title()}<span class="issue-code">{issue.code}</span><StatusPill status="info" />{/snippet}<p class="issue-msg">{issue.message}</p>{#if issue.recommendation}<p class="issue-rec">{issue.recommendation}</p>{/if}</Card>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  {:else}
    <Card padding="md">
      {#snippet title()}What does Doctor check?{/snippet}
      <ul class="check-list"><li><strong>Skill Structure</strong> — {$_('doctor.check_skill_structure')}</li><li><strong>Hook Targets</strong> — {$_('doctor.check_hook_targets')}</li><li><strong>Env Placeholders</strong> — {$_('doctor.check_env_placeholders')}</li><li><strong>MCP Config</strong> — {$_('doctor.check_mcp_config')}</li></ul>
      <div class="score-note">{$_('doctor.score_explanation')}</div>
    </Card>
  {/if}
</div>

<style>
  .doctor-page { display: flex; flex-direction: column; gap: var(--space-4); padding-top: var(--space-4); max-width: 1000px; }
  .page-header { display: flex; align-items: center; gap: var(--space-2); } .page-header h1 { margin: 0; font-size: var(--font-size-xl); }
  .selector-row { display: flex; gap: var(--space-2); align-items: flex-start; }

  .diagnostic-layout { display: grid; grid-template-columns: 280px 1fr; gap: var(--space-4); }

  .diagnostic-left { display: flex; flex-direction: column; gap: var(--space-3); }
  .score-ring { position: relative; display: flex; align-items: center; justify-content: center; padding: var(--space-3); }
  .score-svg { width: 120px; height: 120px; }
  .score-text { position: absolute; text-align: center; }
  .score-value { font-size: 2rem; font-weight: 700; color: var(--text-primary); display: block; line-height: 1; }
  .score-unit { font-size: var(--font-size-xs); color: var(--text-muted); }
  .score-label { text-align: center; font-size: var(--font-size-sm); color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.05em; }

  .severity-list { display: flex; flex-direction: column; gap: var(--space-2); }
  .severity-row { display: flex; align-items: center; gap: var(--space-2); font-size: var(--font-size-sm); color: var(--text-secondary); }
  .severity-row strong { margin-left: auto; }
  .sev-critical { color: var(--color-danger); }
  .sev-warning { color: var(--color-warning); }
  .sev-info { color: var(--color-info); }

  .diagnostic-actions { display: flex; gap: var(--space-2); }
  .diagnostic-right { display: flex; flex-direction: column; gap: var(--space-3); }
  .issue-group-title { margin: 0 0 var(--space-2); font-size: var(--font-size-sm); text-transform: uppercase; letter-spacing: 0.05em; }
  .issue-card { cursor: default; }
  .issue-code { font-family: var(--font-mono); font-size: var(--font-size-xs); }
  .issue-msg { font-size: var(--font-size-sm); color: var(--text-secondary); margin: var(--space-1) 0 0; }
  .issue-rec { font-size: var(--font-size-xs); color: var(--text-muted); margin: var(--space-1) 0 0; padding: var(--space-1) var(--space-2); background: var(--bg-elevated); border-radius: var(--radius-sm); }

  .check-list { margin: 0; padding-left: var(--space-4); font-size: var(--font-size-sm); color: var(--text-secondary); line-height: 1.8; }
  .score-note { margin-top: var(--space-3); font-size: var(--font-size-xs); color: var(--text-muted); background: var(--bg-elevated); padding: var(--space-2) var(--space-3); border-radius: var(--radius-sm); }
</style>
