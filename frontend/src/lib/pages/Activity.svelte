<script lang="ts">
  import { onMount } from 'svelte';
  import { events, isLoading, error, loadEvents } from '$lib/stores/eventStore';
  import { currentPage } from '$lib/stores/uiStore';
  import { _ } from 'svelte-i18n';

  let typeFilter = $state<string>('');

  onMount(() => {
    loadEvents();
  });

  function handleFilterChange() {
    loadEvents(typeFilter || undefined);
  }

  function getStatusClass(status: string): string {
    switch (status) {
      case 'success':
        return 'status-success';
      case 'failure':
        return 'status-failure';
      case 'partial_failure':
        return 'status-partial';
      default:
        return 'status-unknown';
    }
  }

  function getTypeIcon(opType: string): string {
    switch (opType) {
      case 'scan_repositories':
        return '🔍';
      case 'refresh_repository':
        return '🔄';
      case 'export_pack':
        return '📦';
      case 'delete_pack':
        return '🗑️';
      case 'build_migration_plan':
        return '📋';
      case 'apply_migration':
        return '▶️';
      case 'rollback_migration':
        return '⏪';
      case 'run_doctor':
        return '🏥';
      default:
        return '📄';
    }
  }

  function formatTimestamp(ts: string): string {
    try {
      const d = new Date(ts);
      return d.toLocaleString();
    } catch {
      return ts;
    }
  }
</script>

<div class="activity-page">
  <header class="page-header">
    <button class="back-btn" onclick={() => currentPage.set('dashboard')}>
      &larr; {$_('nav.back')}
    </button>
    <h1>{$_('activity.title')}</h1>
  </header>

  <div class="filter-bar">
    <select bind:value={typeFilter} onchange={handleFilterChange} class="type-filter">
      <option value="">{$_('activity.all_types')}</option>
      <option value="scan_repositories">{$_('activity.scan')}</option>
      <option value="refresh_repository">{$_('activity.refresh')}</option>
      <option value="export_pack">{$_('activity.export_pack')}</option>
      <option value="delete_pack">{$_('activity.delete_pack')}</option>
      <option value="build_migration_plan">{$_('activity.migration_plan')}</option>
      <option value="apply_migration">{$_('activity.apply_migration')}</option>
      <option value="rollback_migration">{$_('activity.rollback')}</option>
      <option value="run_doctor">{$_('activity.doctor')}</option>
    </select>
    <button class="refresh-btn" onclick={() => loadEvents(typeFilter || undefined)}>
      {$_('activity.refresh')}
    </button>
  </div>

  <main class="activity-content">
    {#if $isLoading}
      <p class="loading">{$_('activity.loading')}</p>
    {:else if $error}
      <div class="error-box">
        <p>Error: {$error}</p>
        <button class="retry-btn" onclick={() => loadEvents(typeFilter || undefined)}
          >{$_('activity.retry')}</button
        >
      </div>
    {:else if $events.length === 0}
      <div class="empty-state">
        <p>{$_('activity.empty')}</p>
      </div>
    {:else}
      <div class="event-list">
        {#each $events as event (event.id)}
          <div class="event-card">
            <div class="event-icon">{getTypeIcon(event.operation_type)}</div>
            <div class="event-body">
              <div class="event-header">
                <span class="event-type">{event.operation_type.replace(/_/g, ' ')}</span>
                <span class="event-status {getStatusClass(event.status)}"
                  >{#if event.status === 'success'}{$_(
                      'activity.status_success',
                    )}{:else if event.status === 'failure'}{$_(
                      'activity.status_failed',
                    )}{:else if event.status === 'partial_failure'}{$_(
                      'activity.status_partial',
                    )}{:else}{event.status}{/if}</span
                >
              </div>
              {#if event.summary}
                <p class="event-summary">{event.summary}</p>
              {/if}
              <div class="event-meta">
                <span class="event-time">{formatTimestamp(event.created_at)}</span>
                {#if event.repo_id}
                  <span class="event-ref">repo: {event.repo_id}</span>
                {/if}
                {#if event.pack_id}
                  <span class="event-ref">pack: {event.pack_id}</span>
                {/if}
              </div>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </main>
</div>

<style>
  .activity-page {
    min-height: 100vh;
    background: var(--bg-elevated);
  }
  .page-header {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-4) var(--space-6);
    border-bottom: 1px solid var(--border-default);
    background: var(--bg-card);
  }
  .page-header h1 {
    margin: 0;
    font-size: var(--font-size-xl);
  }
  .back-btn {
    padding: 6px var(--space-3);
    background: var(--bg-card);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    font-size: var(--font-size-sm);
    cursor: pointer;
  }
  .back-btn:hover {
    background: var(--bg-elevated);
  }
  .filter-bar {
    display: flex;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-6);
    background: var(--bg-card);
    border-bottom: 1px solid var(--border-default);
  }
  .type-filter {
    padding: 6px var(--space-3);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    font-size: var(--font-size-sm);
  }
  .refresh-btn {
    padding: 6px var(--space-4);
    background: var(--color-primary);
    color: var(--text-primary);
    border: none;
    border-radius: var(--radius-md);
    font-size: var(--font-size-sm);
    cursor: pointer;
  }
  .refresh-btn:hover {
    background: var(--color-primary-hover);
  }
  .activity-content {
    padding: var(--space-6);
    max-width: 800px;
    margin: 0 auto;
  }
  .loading {
    text-align: center;
    color: var(--text-muted);
    padding: 40px 0;
  }
  .error-box {
    text-align: center;
    padding: var(--space-6);
    background: var(--color-danger-bg);
    border: 1px solid var(--color-danger);
    border-radius: var(--radius-md);
    color: var(--color-danger);
  }
  .retry-btn {
    margin-top: var(--space-2);
    padding: 6px var(--space-4);
    background: var(--color-danger);
    color: var(--text-primary);
    border: none;
    border-radius: var(--radius-md);
    cursor: pointer;
  }
  .empty-state {
    text-align: center;
    padding: 60px var(--space-6);
    color: var(--text-muted);
  }
  .event-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }
  .event-card {
    display: flex;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4);
    background: var(--bg-card);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    align-items: flex-start;
  }
  .event-icon {
    font-size: var(--font-size-xl);
    line-height: 1.5;
    flex-shrink: 0;
  }
  .event-body {
    flex: 1;
    min-width: 0;
  }
  .event-header {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: var(--space-1);
  }
  .event-type {
    font-weight: 600;
    font-size: var(--font-size-md);
    text-transform: capitalize;
  }
  .event-status {
    font-size: var(--font-size-xs);
    padding: 2px var(--space-2);
    border-radius: var(--radius-full);
    font-weight: 500;
  }
  .status-success {
    background: var(--color-success-bg);
    color: var(--color-success);
  }
  .status-failure {
    background: var(--color-danger-bg);
    color: var(--color-danger);
  }
  .status-partial {
    background: var(--color-warning-bg);
    color: var(--color-warning);
  }
  .status-unknown {
    background: var(--bg-hover);
    color: var(--text-secondary);
  }
  .event-summary {
    margin: 2px 0;
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
  }
  .event-meta {
    display: flex;
    gap: var(--space-3);
    margin-top: var(--space-1);
    font-size: 0.78rem;
    color: var(--text-muted);
  }
  .event-ref {
    background: var(--bg-hover);
    padding: 1px 6px;
    border-radius: var(--radius-sm);
  }
</style>
