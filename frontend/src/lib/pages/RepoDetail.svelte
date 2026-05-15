<script lang="ts">
  import { currentPage, navigateTo } from "$lib/stores/uiStore";
  import { selectedRepoDetail, refreshRepository, removeRepository, isLoading } from "$lib/stores/repoStore";
  import { resources, typeGroups, selectedType, loadCapabilityInventory, setTypeFilter } from "$lib/stores/capabilityStore";
  import CapabilityList from "$lib/components/CapabilityList.svelte";
  import ResourceDetail from "$lib/components/ResourceDetail.svelte";

  let { repoId }: { repoId: string } = $props();

  let activeTab = $state<"overview" | "capabilities">("overview");
  let selectedResourceId = $state<string | null>(null);
  let detail = $derived($selectedRepoDetail);

  $effect(() => {
    if (detail?.repo.id) {
      loadCapabilityInventory(detail.repo.id);
    }
  });
</script>

<div class="repo-detail-page">
  <header class="repo-header">
    <div class="header-top">
      <button class="back-btn" onclick={() => navigateTo("dashboard")}>Back</button>
    </div>
    {#if detail}
      <div class="header-info">
        <h1>{detail.repo.name}</h1>
        <p class="repo-path">{detail.repo.path}</p>
        <div class="repo-tags">
          {#if detail.repo.current_branch}
            <span class="tag branch">{detail.repo.current_branch}</span>
          {/if}
          <span class="tag state state-{detail.repo.dirty_state}">
            {detail.repo.dirty_state}
          </span>
        </div>
      </div>
      <div class="header-actions">
        <button class="action-btn" onclick={() => refreshRepository(repoId)} disabled={$isLoading}>
          Refresh
        </button>
        <button class="action-btn danger" onclick={() => removeRepository(repoId)} disabled={$isLoading}>
          Remove
        </button>
        <button class="action-btn" onclick={() => currentPage.set("packexport")}>
          Export Pack
        </button>
        <button class="action-btn" onclick={() => currentPage.set("doctor")}>
          Run Doctor
        </button>
      </div>
    {:else}
      <p class="loading">Loading repository...</p>
    {/if}
  </header>

  <div class="tabs">
    <button class="tab-btn" class:active={activeTab === "overview"} onclick={() => activeTab = "overview"}>
      Overview
    </button>
    <button class="tab-btn" class:active={activeTab === "capabilities"} onclick={() => activeTab = "capabilities"}>
      Capabilities
    </button>
  </div>

  <main class="repo-content">
    {#if $isLoading}
      <p class="loading">Loading...</p>
    {:else if activeTab === "overview"}
      {#if detail}
        <div class="overview-grid">
          <div class="overview-card">
            <h3>Repository Info</h3>
            <dl class="info-list">
              <dt>Name</dt>
              <dd>{detail.repo.name}</dd>
              <dt>Path</dt>
              <dd>{detail.repo.path}</dd>
              <dt>Remote</dt>
              <dd>{detail.repo.remote_url ?? "N/A"}</dd>
              <dt>Branch</dt>
              <dd>{detail.repo.current_branch ?? "N/A"}</dd>
              <dt>HEAD</dt>
              <dd>{detail.repo.head_commit ?? "N/A"}</dd>
              <dt>Last Indexed</dt>
              <dd>{new Date(detail.repo.last_indexed_at).toLocaleString()}</dd>
            </dl>
          </div>
          <div class="overview-card">
            <h3>Capability Summary</h3>
            <dl class="info-list">
              {#each Object.entries(detail.capabilities) as [type, items]}
                {#if (items as any[]).length > 0}
                  <dt>{type}</dt>
                  <dd>{(items as any[]).length}</dd>
                {/if}
              {/each}
            </dl>
          </div>
        </div>
      {/if}
    {:else}
      <div class="capabilities-layout">
        <CapabilityList
          resources={$typeGroups}
          selectedType={$selectedType}
          selectedResourceId={selectedResourceId}
          onSelectType={setTypeFilter}
          onSelectResource={(id) => selectedResourceId = id}
        />
        <div class="content-panel">
          {#if selectedResourceId && $resources}
            {@const allResources = [...$resources.skills, ...$resources.mcp, ...$resources.hooks, ...$resources.rules, ...$resources.agents]}
            {@const resource = allResources.find(r => r.id === selectedResourceId)}
            <ResourceDetail resource={resource ?? null} />
          {:else}
            <p class="select-hint">Select a capability to view details</p>
          {/if}
        </div>
      </div>
    {/if}
  </main>
</div>

<style>
  .repo-detail-page {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
  }
  .repo-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    padding: 16px 24px;
    border-bottom: 1px solid #e2e8f0;
    background: #fff;
  }
  .header-info h1 {
    margin: 0;
  }
  .repo-path {
    color: #64748b;
    font-size: 0.85rem;
    margin: 4px 0;
  }
  .repo-tags {
    display: flex;
    gap: 6px;
    margin-top: 8px;
  }
  .tag {
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 0.75rem;
  }
  .tag.branch { background: #ede9fe; color: #5b21b6; }
  .tag.state-clean { background: #dcfce7; color: #166534; }
  .tag.state-modified { background: #fef3c7; color: #92400e; }
  .tag.state-unknown { background: #f1f5f9; color: #64748b; }
  .header-actions {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
  .action-btn {
    padding: 8px 16px;
    background: #fff;
    border: 1px solid #e2e8f0;
    border-radius: 6px;
    cursor: pointer;
  }
  .action-btn.danger {
    color: #dc2626;
    border-color: #fecaca;
  }
  .header-top {
    width: 100%;
    margin-bottom: 8px;
  }
  .back-btn {
    padding: 8px 16px;
    background: var(--primary, #3b82f6);
    color: #fff;
    border: none;
    border-radius: 6px;
  }
  .tabs {
    display: flex;
    border-bottom: 1px solid #e2e8f0;
    padding: 0 24px;
    background: #fff;
  }
  .tab-btn {
    padding: 12px 20px;
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    cursor: pointer;
    font-size: 0.9rem;
  }
  .tab-btn.active {
    border-bottom-color: var(--primary, #3b82f6);
    color: var(--primary, #3b82f6);
  }
  .repo-content {
    flex: 1;
    padding: 24px;
    background: #f8fafc;
  }
  .overview-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 16px;
  }
  .overview-card {
    background: #fff;
    padding: 20px;
    border-radius: 8px;
    border: 1px solid #e2e8f0;
  }
  .overview-card h3 {
    margin: 0 0 12px 0;
    font-size: 0.95rem;
    color: #475569;
  }
  .info-list {
    display: grid;
    gap: 8px;
  }
  .info-list dt {
    font-size: 0.8rem;
    color: #64748b;
  }
  .info-list dd {
    font-size: 0.85rem;
    margin-left: 0;
  }
  .capabilities-layout {
    display: grid;
    grid-template-columns: 200px 1fr;
    gap: 16px;
    min-height: 400px;
    background: #fff;
    border-radius: 8px;
    border: 1px solid #e2e8f0;
  }
  .content-panel {
    padding: 16px;
  }
  .select-hint, .loading {
    text-align: center;
    color: #64748b;
    padding: 40px 0;
  }
</style>