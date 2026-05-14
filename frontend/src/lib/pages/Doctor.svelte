<script lang="ts">
  import { currentPage, navigateTo } from "$lib/stores/uiStore";
  import { repos, loadRepos, selectedRepoId } from "$lib/stores/repoStore";
  import { runDoctor, report, isLoading, error, clearReport } from "$lib/stores/doctorStore";
  import DoctorReportComp from "$lib/components/DoctorReport.svelte";

  let selectedRepo = $state<string | null>(null);

  loadRepos();

  async function handleRunDoctor() {
    if (!selectedRepo) return;
    await runDoctor(selectedRepo);
  }
</script>

<div class="doctor-page">
  <header class="page-header">
    <h1>Repository Doctor</h1>
    <button class="back-btn" onclick={() => navigateTo("dashboard")}>Back to Dashboard</button>
  </header>

  <main class="doctor-content">
    <div class="select-section">
      <h2>Select Repository to Diagnose</h2>
      <select bind:value={selectedRepo}>
        <option value="">Choose a repository...</option>
        {#each $repos as repo (repo.id)}
          <option value={repo.id}>{repo.name}</option>
        {/each}
      </select>
      <button
        class="run-btn"
        onclick={handleRunDoctor}
        disabled={!selectedRepo || $isLoading}
      >
        {#if $isLoading}Running...{:else}Run Doctor{/if}
      </button>
    </div>

    {#if $error}
      <div class="error-box">
        <p>{$error}</p>
        <button onclick={() => error.set(null)}>Dismiss</button>
      </div>
    {/if}

    {#if $report}
      <div class="report-section">
        <h2>Health Report for {selectedRepo}</h2>
        <DoctorReportComp report={$report} />
        <div class="report-actions">
          <button class="nav-btn" onclick={() => currentPage.set("repo:" + selectedRepo)}>View Repository</button>
          <button class="nav-btn" onclick={clearReport}>Run Again</button>
        </div>
      </div>
    {:else}
      <div class="doctor-info">
        <h3>What does Doctor check?</h3>
        <ul>
          <li><strong>Skill Structure</strong> — Validates skill directory and SKILL.md presence</li>
          <li><strong>Hook Targets</strong> — Checks referenced scripts exist</li>
          <li><strong>Env Placeholders</strong> — Detects unresolved <code>${'{VAR}'}</code> placeholders</li>
          <li><strong>MCP Config</strong> — Validates MCP server configuration</li>
        </ul>
        <p class="score-explanation">
          <strong>Scoring:</strong> Start at 100. Critical issues: -20 each, Warning: -5 each, Info: -1 each.
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
  .page-header h1 { margin: 0; }
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
  .run-btn:disabled { opacity: 0.6; }
  .error-box {
    background: #fee2e2;
    padding: 12px;
    border-radius: 6px;
    margin-bottom: 24px;
  }
  .error-box p { margin: 0 0 8px 0; color: #991b1b; }
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