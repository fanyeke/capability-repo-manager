<script lang="ts">
  import { _ } from "svelte-i18n";
  import type { DoctorReport as DoctorReportType, DoctorIssue } from "$lib/types";
  import { groupIssuesBySeverity, getScoreColor } from "$lib/stores/doctorStore";

  function getScoreLabel(score: number): string {
    if (score >= 80) return "doctor.healthy";
    if (score >= 50) return "doctor.needs_attention";
    return "doctor.critical";
  }

  let {
    report,
  }: {
    report: DoctorReportType | null;
  } = $props();

  let grouped = $derived(report ? groupIssuesBySeverity(report.issues) : null);
  let scoreColor = $derived(report ? getScoreColor(report.score) : "#94a3b8");
  let scoreLabel = $derived(report ? getScoreLabel(report.score) : "");
</script>

{#if !report}
  <p class="empty-text">{$_('doctor.no_health_report')}</p>
{:else}
  <div class="doctor-report">
    <div class="score-section">
      <div class="score-circle" style="border-color: {scoreColor}; color: {scoreColor}">
        <span class="score-value">{report.score}</span>
        <span class="score-label">{scoreLabel ? $_(scoreLabel) : ""}</span>
      </div>
      <p class="score-date">{$_('doctor.diagnosed_at', { values: { date: new Date(report.created_at).toLocaleString() } })}</p>
    </div>

    <div class="issues-section">
      {#each ["critical", "warning", "info"] as severity}
        {#if grouped?.[severity]?.length}
          <div class="severity-group">
            <h4 class="severity-{severity}">
              {$_(severity === "critical" ? 'doctor.critical' : severity === "warning" ? 'doctor.warning' : 'doctor.info')} ({grouped[severity].length})
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
                    <span class="issue-resource">{$_('doctor.resource_ref', { values: { ref: issue.resource_ref } })}</span>
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
    gap: 24px;
  }
  .score-section {
    text-align: center;
    padding: 24px 0;
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
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .score-date {
    font-size: 0.8rem;
    color: #64748b;
    margin-top: 8px;
  }
  .issues-section {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .severity-group h4 {
    margin: 0 0 8px 0;
    font-size: 0.85rem;
  }
  .severity-critical { color: #dc2626; }
  .severity-warning { color: #d97706; }
  .severity-info { color: #2563eb; }
  .issue-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .issue-item {
    padding: 10px 14px;
    border-radius: 6px;
    border-left: 4px solid;
  }
  .issue-item.severity-critical {
    background: #fef2f2;
    border-color: #fca5a5;
  }
  .issue-item.severity-warning {
    background: #fffbeb;
    border-color: #fcd34d;
  }
  .issue-item.severity-info {
    background: #eff6ff;
    border-color: #93c5fd;
  }
  .issue-code {
    font-size: 0.7rem;
    font-weight: 600;
  }
  .severity-critical .issue-code { color: #991b1b; }
  .severity-warning .issue-code { color: #92400e; }
  .severity-info .issue-code { color: #1e40af; }
  .issue-message {
    margin: 4px 0;
    font-size: 0.85rem;
    color: #334155;
  }
  .issue-recommendation {
    margin: 2px 0;
    font-size: 0.8rem;
    color: #475569;
    font-style: italic;
  }
  .issue-resource {
    font-size: 0.7rem;
    color: #94a3b8;
  }
  .no-issues {
    color: #16a34a;
    text-align: center;
    font-size: 0.95rem;
    padding: 16px 0;
  }
  .empty-text {
    text-align: center;
    color: #64748b;
    padding: 40px 0;
  }
</style>
