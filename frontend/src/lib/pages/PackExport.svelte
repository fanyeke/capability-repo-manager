<script lang="ts">
  import { currentPage, navigateTo } from "$lib/stores/uiStore";
  import { selectedRepoDetail } from "$lib/stores/repoStore";
  import { resources, loadCapabilityInventory, typeGroups } from "$lib/stores/capabilityStore";
  import { exportPack, isLoading } from "$lib/stores/packStore";

  let packName = $state("");
  let packVersion = $state("1.0.0");
  let packDescription = $state("");
  let packType = $state("project");
  let detail = $derived($selectedRepoDetail);
  let selectedIds = $state<Set<string>>(new Set());

  function toggleResource(id: string) {
    if (selectedIds.has(id)) {
      selectedIds.delete(id);
    } else {
      selectedIds.add(id);
    }
  }

  async function handleExport() {
    if (!detail?.repo.id || selectedIds.size === 0) return;

    await exportPack(
      detail.repo.id,
      { resource_ids: Array.from(selectedIds) },
      {
        name: packName,
        version: packVersion,
        description: packDescription || undefined,
        pack_type: packType,
      },
    );

    currentPage.set("packapply");
  }
</script>

<div class="pack-export-page">
  <header class="page-header">
    <h1>Export Capability Pack</h1>
    <button class="back-btn" onclick={() => navigateTo("dashboard")}>Back to Dashboard</button>
  </header>

  {#if !detail}
    <p class="empty-text">Select a repository first to export its capabilities.</p>
  {:else}
    <div class="export-content">
      <section class="resource-selection">
        <h2>Select Resources from {detail.repo.name}</h2>

        <div class="resource-groups">
          {#each $typeGroups as group (group.type)}
            {#if group.count > 0}
              <div class="resource-group">
                <h3>{group.label} ({group.count})</h3>
                <ul class="resource-list">
                  {#each ($resources as any)[group.type] as resource (resource.id)}
                    <li>
                      <label>
                        <input
                          type="checkbox"
                          checked={selectedIds.has(resource.id)}
                          onchange={() => toggleResource(resource.id)}
                        />
                        {resource.name}
                        {#if resource.error_message}
                          <span class="error-tag">error</span>
                        {/if}
                      </label>
                    </li>
                  {/each}
                </ul>
              </div>
            {/if}
          {/each}
        </div>
      </section>

      <section class="pack-metadata">
        <h2>Pack Metadata</h2>

        <div class="form-group">
          <label>Name *</label>
          <input type="text" bind:value={packName} required />
        </div>

        <div class="form-group">
          <label>Version *</label>
          <input type="text" bind:value={packVersion} placeholder="1.0.0" required />
        </div>

        <div class="form-group">
          <label>Description</label>
          <textarea bind:value={packDescription} rows={2}></textarea>
        </div>

        <div class="form-group">
          <label>Type</label>
          <select bind:value={packType}>
            <option value="project">Project</option>
            <option value="blueprint">Blueprint</option>
            <option value="baseline">Baseline</option>
          </select>
        </div>

        <div class="preview-section">
          <h3>Preview</h3>
          <p><strong>Selected:</strong> {selectedIds.size} resources</p>
          <p><strong>Source:</strong> {detail.repo.name}</p>
        </div>

        <div class="form-actions">
          <button
            class="export-btn"
            onclick={handleExport}
            disabled={$isLoading || !packName || selectedIds.size === 0}
          >
            {#if $isLoading}
              Exporting...
            {:else}
              Export Pack
            {/if}
          </button>
        </div>
      </section>
    </div>
  {/if}
</div>

<style>
  .pack-export-page {
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
  .export-content {
    display: grid;
    grid-template-columns: 1fr 350px;
    gap: 24px;
    padding: 24px;
    background: #f8fafc;
  }
  .resource-selection h2, .pack-metadata h2 {
    margin: 0 0 16px 0;
    font-size: 1.1rem;
  }
  .resource-groups {
    display: flex;
    flex-direction: column;
    gap: 16px;
    background: #fff;
    padding: 20px;
    border-radius: 8px;
    border: 1px solid #e2e8f0;
  }
  .resource-group h3 {
    margin: 0 0 8px 0;
    font-size: 0.9rem;
    color: #475569;
  }
  .resource-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .resource-list li label {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.85rem;
  }
  .error-tag {
    padding: 1px 6px;
    background: #fee2e2;
    color: #991b1b;
    border-radius: 4px;
    font-size: 0.7rem;
  }
  .pack-metadata {
    background: #fff;
    padding: 20px;
    border-radius: 8px;
    border: 1px solid #e2e8f0;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .form-group label {
    display: block;
    font-weight: 600;
    margin-bottom: 4px;
    font-size: 0.85rem;
  }
  .form-group input,
  .form-group textarea,
  .form-group select {
    width: 100%;
    padding: 8px 12px;
    border: 1px solid #e2e8f0;
    border-radius: 6px;
    font-size: 0.9rem;
    box-sizing: border-box;
  }
  .preview-section {
    background: #f8fafc;
    padding: 12px;
    border-radius: 6px;
    font-size: 0.85rem;
  }
  .preview-section h3 {
    margin: 0 0 8px 0;
    font-size: 0.8rem;
    color: #64748b;
  }
  .export-btn {
    padding: 12px 24px;
    background: var(--primary, #3b82f6);
    color: #fff;
    border: none;
    border-radius: 8px;
    font-weight: 600;
    cursor: pointer;
  }
  .export-btn:disabled {
    opacity: 0.6;
  }
  .empty-text {
    text-align: center;
    color: #64748b;
    padding: 40px 0;
  }
</style>