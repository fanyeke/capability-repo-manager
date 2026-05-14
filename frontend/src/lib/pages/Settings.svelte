<script lang="ts">
  import { currentPage, navigateTo } from "$lib/stores/uiStore";

  let scanRoots = $state("");
  let scanDepth = $state("5");
  let ignorePatterns = $state("node_modules, .venv, vendor, .cache, build");
  let packStorageDir = $state("");
  let isLoading = $state(false);

  async function loadSettings() {
    isLoading = true;
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const settings = await invoke<{
        scan_roots: string[];
        scan_depth: number;
        ignore_patterns: string[];
        pack_storage_dir: string;
        file_watch_enabled: boolean;
      }>("get_settings");
      scanRoots = settings.scan_roots.join(", ");
      scanDepth = String(settings.scan_depth);
      ignorePatterns = settings.ignore_patterns.join(", ");
      packStorageDir = settings.pack_storage_dir;
    } catch (e) {
      console.error("Failed to load settings:", e);
    } finally {
      isLoading = false;
    }
  }

  async function saveSettings() {
    isLoading = true;
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("update_settings", {
        newSettings: {
          scan_roots: scanRoots.split(/[,\s]+/).filter((p) => p.trim()),
          scan_depth: parseInt(scanDepth, 10) || 5,
          ignore_patterns: ignorePatterns.split(/[,\s]+/).filter((p) => p.trim()),
          pack_storage_dir: packStorageDir.trim() || "~/.capability-repo-manager/packs",
          file_watch_enabled: false,
        },
      });
    } catch (e) {
      console.error("Failed to save settings:", e);
    } finally {
      isLoading = false;
    }
  }

  loadSettings();
</script>

<div class="settings-page">
  <header class="settings-header">
    <h1>Settings</h1>
    <button class="back-btn" onclick={() => navigateTo("dashboard")}>Back to Dashboard</button>
  </header>

  {#if isLoading}
    <p class="loading">Loading settings...</p>
  {:else}
    <form class="settings-form" onsubmit={(e) => { e.preventDefault(); saveSettings(); }}>
      <div class="form-group">
        <label>Scan Roots</label>
        <textarea
          bind:value={scanRoots}
          placeholder="/home/user/projects, /home/user/work"
          rows={2}
        ></textarea>
        <span class="hint">Comma-separated directories to scan for Git repositories</span>
      </div>

      <div class="form-group">
        <label>Scan Depth</label>
        <input type="number" bind:value={scanDepth} min="1" max="20" />
        <span class="hint">Maximum depth to traverse when scanning for repositories</span>
      </div>

      <div class="form-group">
        <label>Ignore Patterns</label>
        <textarea bind:value={ignorePatterns} rows={2}></textarea>
        <span class="hint">Directory names to skip when scanning (comma-separated)</span>
      </div>

      <div class="form-group">
        <label>Pack Storage Directory</label>
        <input type="text" bind:value={packStorageDir} placeholder="~/.capability-repo-manager/packs" />
        <span class="hint">Where exported capability packs are stored</span>
      </div>

      <div class="form-actions">
        <button type="submit" class="save-btn" disabled={isLoading}>Save Settings</button>
      </div>
    </form>
  {/if}
</div>

<style>
  .settings-page {
    max-width: 600px;
    margin: 0 auto;
    padding: 24px;
  }
  .settings-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 24px;
  }
  .settings-header h1 {
    margin: 0;
  }
  .back-btn {
    padding: 8px 16px;
    background: #fff;
    border: 1px solid #e2e8f0;
    border-radius: 6px;
    cursor: pointer;
  }
  .settings-form {
    display: flex;
    flex-direction: column;
    gap: 20px;
    background: #fff;
    padding: 24px;
    border-radius: 12px;
    border: 1px solid #e2e8f0;
  }
  .form-group label {
    display: block;
    font-weight: 600;
    margin-bottom: 8px;
  }
  .form-group input,
  .form-group textarea {
    width: 100%;
    padding: 10px;
    border: 1px solid #e2e8f0;
    border-radius: 6px;
    font-size: 0.95rem;
    box-sizing: border-box;
  }
  .hint {
    display: block;
    font-size: 0.8rem;
    color: #64748b;
    margin-top: 4px;
  }
  .form-actions {
    margin-top: 8px;
  }
  .save-btn {
    padding: 12px 24px;
    background: var(--primary, #3b82f6);
    color: #fff;
    border: none;
    border-radius: 8px;
    font-weight: 600;
    cursor: pointer;
  }
  .save-btn:disabled {
    opacity: 0.6;
  }
  .loading {
    text-align: center;
    color: #64748b;
    padding: 40px 0;
  }
</style>