<script lang="ts">
  import { _ } from 'svelte-i18n';
  import type { DoctorReport as DoctorReportType, DoctorIssue } from '$lib/types';
  import { groupIssuesBySeverity, getScoreColor } from '$lib/stores/doctorStore';

  function getScoreLabel(score: number): string {
    if (score >= 80) return 'doctor.healthy';
    if (score >= 50) return 'doctor.needs_attention';
    return 'doctor.critical';
  }

  let {
    report,
  }: {
    report: DoctorReportType | null;
  } = $props();

  let grouped = $derived(report ? groupIssuesBySeverity(report.issues) : null);
  let scoreColor = $derived(report ? getScoreColor(report.score) : '#94a3b8');
  let scoreLabel = $derived(report ? getScoreLabel(report.score) : '');
</script>

{#if !report}
  <p class="empty-text">{$_('doctor.no_health_report')}</p>
{:else}
  <div class="doctor-report">
    <div class="score-section">
      <div class="score-circle" style="border-color: {scoreColor}; color: {scoreColor}">
        <span class="score-value">{report.score}</span>
        <span class="score-label">{scoreLabel ? $_(scoreLabel) : ''}</span>
      </div>
      <p class="score-date">
        {$_('doctor.diagnosed_at', {
          values: { date: new Date(report.created_at).toLocaleString() },
        })}
      </p>
    </div>

    <div class="issues-section">
      {#each ['critical', 'warning', 'info'] as severity}
        {#if grouped?.[severity]?.length}
          <div class="severity-group">
            <h4 class="severity-{severity}">
              {$_(
                severity === 'critical'
                  ? 'doctor.critical'
                  : severity === 'warning'
                    ? 'doctor.warning'
                    : 'doctor.info',
              )} ({grouped[severity].length})
            </h4>
            <ul class="issue-list">
              {#each grouped[severity] as issue (issue.code)}
                <li class="issue-item severity-{severity}">
                  <div class="issue-header">
                    <code class="issue-code">{issue.code}</code>
                  </div>
                  <p class="issue-message">{issue.message}</p>
                  {#if issue.recommendation}
                    <p class="issue-recommendation">{issue.recommendation}</p>
                  {/if}
                  {#if issue.resource_ref}
                    <span class="issue-resource"
                      >{$_('doctor.resource_ref', { values: { ref: issue.resource_ref } })}</span
                    >
                  {/if}
                </li>
              {/each}
            </ul>
          </div>
        {/if}
      {/each}

      {#if report.issues.length === 0}
        <p class="no-issues">{$_('doctor.no_issues_found')}</p>
      {/if}
    </div>
  </div>
{/if}

<style>
  .doctor-report {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
  }
  .score-section {
    text-align: center;
    padding: var(--space-6) 0;
  }
  .score-circle {
    width: 120px;
    height: 120px;
    border-radius: 50%;
    border: 6px solid;
    display: inline-flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
  }
  .score-value {
    font-size: 2rem;
    font-weight: 700;
  }
  .score-label {
    font-size: var(--font-size-xs);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .score-date {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    margin-top: var(--space-2);
  }
  .issues-section {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }
  .severity-group h4 {
    margin: 0 0 var(--space-2) 0;
    font-size: var(--font-size-sm);
  }
  .severity-critical {
    color: var(--color-danger);
  }
  .severity-warning {
    color: var(--color-warning);
  }
  .severity-info {
    color: var(--color-primary);
  }
  .issue-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }
  .issue-item {
    padding: 10px 14px;
    border-radius: var(--radius-md);
    border-left: 4px solid;
  }
  .issue-item.severity-critical {
    background: var(--color-danger-bg);
    border-color: var(--color-danger);
  }
  .issue-item.severity-warning {
    background: var(--color-warning-bg);
    border-color: var(--color-warning);
  }
  .issue-item.severity-info {
    background: var(--color-info-bg);
    border-color: var(--color-info);
  }
  .issue-code {
    font-size: var(--font-size-xs);
    font-weight: 600;
  }
  .severity-critical .issue-code {
    color: var(--color-danger);
  }
  .severity-warning .issue-code {
    color: var(--color-warning);
  }
  .severity-info .issue-code {
    color: var(--color-primary);
  }
  .issue-message {
    margin: var(--space-1) 0;
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
  }
  .issue-recommendation {
    margin: 2px 0;
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    font-style: italic;
  }
  .issue-resource {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
  }
  .no-issues {
    color: var(--color-success);
    text-align: center;
    font-size: 0.95rem;
    padding: var(--space-4) 0;
  }
  .empty-text {
    text-align: center;
    color: var(--text-muted);
    padding: 40px 0;
  }
</style>
