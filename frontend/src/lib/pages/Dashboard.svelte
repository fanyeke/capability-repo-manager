<script lang="ts">
  import { currentPage, navigateTo, navigateToRepo } from "$lib/stores/uiStore";
  import { repos, filteredRepos, scanRepositories, loadRepos, selectRepo, isLoading } from "$lib/stores/repoStore";
  import RepoList from "$lib/components/RepoList.svelte";

  let filter = $state({});

  async function handleScan() {
    const settings = await getSettings();
    if (settings.scan_roots.length > 0) {
      await scanRepositories(settings.scan_roots);
    } else {
      currentPage.set("guidedsetup");
    }
  }

  async function getSettings() {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke<{
      scan_roots: string[];
      scan_depth: number;
    }>("get_settings");
  }

  function handleSelectRepo(repoId: string) {
    navigateToRepo(repoId);
  }

  function handleFilterChange(newFilter: any) {
    filter = newFilter;
  }
</script>

<div class="dashboard">
  <header class="dashboard-header">
    <h1>Capability Repo Manager</h1>
    <nav class="nav-links">
      <button class="nav-btn" onclick={() => currentPage.set("settings")}>Settings</button>
      <button class="nav-btn" onclick={() => currentPage.set("packexport")}>Export Pack</button>
      <button class="nav-btn" onclick={() => currentPage.set("packapply")}>Apply Pack</button>
      <button class="nav-btn" onclick={() => currentPage.set("doctor")}>Doctor</button>
      <button class="nav-btn" onclick={() => currentPage.set("compare")}>Compare</button>
    </nav>
  </header>

  <main class="dashboard-content">
    {#if $isLoading}
      <p class="loading">Loading repositories...</p>
    {:else}
      <RepoList
        repos={$filteredRepos}
        filter={filter}
        isLoading={$isLoading}
        onSelectRepo={handleSelectRepo}
        onFilterChange={handleFilterChange}
        onScan={handleScan}
      />
    {/if}
  </main>
</div>

<style>
  .dashboard {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
  }
  .dashboard-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 24px;
    border-bottom: 1px solid var(--border-color, #e2e8f0);
    background: #fff;
  }
  .dashboard-header h1 {
    margin: 0;
    font-size: 1.25rem;
  }
  .nav-links {
    display: flex;
    gap: 8px;
  }
  .nav-btn {
    padding: 8px 16px;
    background: #fff;
    border: 1px solid #e2e8f0;
    border-radius: 6px;
    font-size: 0.85rem;
    cursor: pointer;
  }
  .nav-btn:hover {
    background: #f8fafc;
  }
  .dashboard-content {
    flex: 1;
    padding: 24px;
    background: #f8fafc;
  }
  .loading {
    text-align: center;
    color: #64748b;
    padding: 40px 0;
  }
</style>