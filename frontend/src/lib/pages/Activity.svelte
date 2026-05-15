<script lang="ts">
  import { onMount } from "svelte";
  import { events, isLoading, error, loadEvents } from "$lib/stores/eventStore";
  import { currentPage } from "$lib/stores/uiStore";
  import { _ } from "svelte-i18n";

  let typeFilter = $state<string>("");

  onMount(() => {
    loadEvents();
  });

  function handleFilterChange() {
    loadEvents(typeFilter || undefined);
  }

  function getStatusClass(status: string): string {
    switch (status) {
      case "success": return "status-success";
      case "failure": return "status-failure";
      case "partial_failure": return "status-partial";
      default: return "status-unknown";
    }
  }

  function getStatusLabel(status: string): string {
    switch (status) {
      case "success": return "Success";
      case "failure": return "Failed";
      case "partial_failure": return "Partial";
      default: return status;
    }
  }

  function getTypeIcon(opType: string): string {
    switch (opType) {
      case "scan_repositories": return "🔍";
      case "refresh_repository": return "🔄";
      case "export_pack": return "📦";
      case "delete_pack": return "🗑️";
      case "build_migration_plan": return "📋";
      case "apply_migration": return "▶️";
      case "rollback_migration": return "⏪";
      case "run_doctor": return "🏥";
      default: return "📄";
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
    <button class="back-btn" onclick={() => currentPage.set("dashboard")}>
      &larr; {$_('nav.back')}
    </button>
    <h1>Activity History</h1>
  </header>

  <div class="filter-bar">
    <select
      bind:value={typeFilter}
      onchange={handleFilterChange}
      class="type-filter"
    >
      <option value="">All Types</option>
      <option value="scan_repositories">Scan</option>
      <option value="refresh_repository">Refresh</option>
      <option value="export_pack">Export Pack</option>
      <option value="delete_pack">Delete Pack</option>
      <option value="build_migration_plan">Migration Plan</option>
      <option value="apply_migration">Apply Migration</option>
      <option value="rollback_migration">Rollback</option>
      <option value="run_doctor">Doctor</option>
    </select>
    <button class="refresh-btn" onclick={() => loadEvents(typeFilter || undefined)}>
      Refresh
    </button>
  </div>

  <main class="activity-content">
    {#if $isLoading}
      <p class="loading">Loading...</p>
    {:else if $error}
      <div class="error-box">
        <p>Error: {$error}</p>
        <button class="retry-btn" onclick={() => loadEvents(typeFilter || undefined)}>Retry</button>
      </div>
    {:else if $events.length === 0}
      <div class="empty-state">
        <p>No activity recorded yet. Run a scan or other operation to see events here.</p>
      </div>
    {:else}
      <div class="event-list">
        {#each $events as event (event.id)}
          <div class="event-card">
            <div class="event-icon">{getTypeIcon(event.operation_type)}</div>
            <div class="event-body">
              <div class="event-header">
                <span class="event-type">{event.operation_type.replace(/_/g, " ")}</span>
                <span class="event-status {getStatusClass(event.status)}">{getStatusLabel(event.status)}</span>
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
    background: #f8fafc;
  }
  .page-header {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 16px 24px;
    border-bottom: 1px solid #e2e8f0;
    background: #fff;
  }
  .page-header h1 {
    margin: 0;
    font-size: 1.25rem;
  }
  .back-btn {
    padding: 6px 12px;
    background: #fff;
    border: 1px solid #e2e8f0;
    border-radius: 6px;
    font-size: 0.85rem;
    cursor: pointer;
  }
  .back-btn:hover {
    background: #f8fafc;
  }
  .filter-bar {
    display: flex;
    gap: 8px;
    padding: 12px 24px;
    background: #fff;
    border-bottom: 1px solid #e2e8f0;
  }
  .type-filter {
    padding: 6px 12px;
    border: 1px solid #e2e8f0;
    border-radius: 6px;
    font-size: 0.85rem;
  }
  .refresh-btn {
    padding: 6px 16px;
    background: #3b82f6;
    color: #fff;
    border: none;
    border-radius: 6px;
    font-size: 0.85rem;
    cursor: pointer;
  }
  .refresh-btn:hover {
    background: #2563eb;
  }
  .activity-content {
    padding: 24px;
    max-width: 800px;
    margin: 0 auto;
  }
  .loading {
    text-align: center;
    color: #64748b;
    padding: 40px 0;
  }
  .error-box {
    text-align: center;
    padding: 24px;
    background: #fef2f2;
    border: 1px solid #fecaca;
    border-radius: 8px;
    color: #b91c1c;
  }
  .retry-btn {
    margin-top: 8px;
    padding: 6px 16px;
    background: #ef4444;
    color: #fff;
    border: none;
    border-radius: 6px;
    cursor: pointer;
  }
  .empty-state {
    text-align: center;
    padding: 60px 24px;
    color: #64748b;
  }
  .event-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .event-card {
    display: flex;
    gap: 12px;
    padding: 12px 16px;
    background: #fff;
    border: 1px solid #e2e8f0;
    border-radius: 8px;
    align-items: flex-start;
  }
  .event-icon {
    font-size: 1.25rem;
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
    gap: 8px;
    margin-bottom: 4px;
  }
  .event-type {
    font-weight: 600;
    font-size: 0.9rem;
    text-transform: capitalize;
  }
  .event-status {
    font-size: 0.75rem;
    padding: 2px 8px;
    border-radius: 10px;
    font-weight: 500;
  }
  .status-success {
    background: #dcfce7;
    color: #166534;
  }
  .status-failure {
    background: #fef2f2;
    color: #b91c1c;
  }
  .status-partial {
    background: #fef9c3;
    color: #854d0e;
  }
  .status-unknown {
    background: #f1f5f9;
    color: #475569;
  }
  .event-summary {
    margin: 2px 0;
    font-size: 0.85rem;
    color: #475569;
  }
  .event-meta {
    display: flex;
    gap: 12px;
    margin-top: 4px;
    font-size: 0.78rem;
    color: #94a3b8;
  }
  .event-ref {
    background: #f1f5f9;
    padding: 1px 6px;
    border-radius: 4px;
  }
</style>