<script lang="ts">
  import { currentPage, navigateTo } from "$lib/stores/uiStore";
  import { _ } from "svelte-i18n";

  let scanRoots = $state("");
  let scanDepth = $state("5");
  let ignorePatterns = $state("node_modules, .venv, vendor, .cache, build");
  let packStorageDir = $state("");
  let isLoading = $state(false);
  let logLevel = $state("info");
  let redactPaths = $state(false);
  let isExporting = $state(false);

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
        log_level: string;
      }>("get_settings");
      scanRoots = settings.scan_roots.join(", ");
      scanDepth = String(settings.scan_depth);
      ignorePatterns = settings.ignore_patterns.join(", ");
      packStorageDir = settings.pack_storage_dir;
      logLevel = settings.log_level || "info";
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
          log_level: logLevel,
        },
      });
    } catch (e) {
      console.error("Failed to save settings:", e);
    } finally {
      isLoading = false;
    }
  }

  async function setLogLevel(level: string) {
    logLevel = level;
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("set_log_level", { level });
    } catch (e) {
      console.error("Failed to set log level:", e);
    }
  }

  async function exportDebugBundle() {
    const { save } = await import("@tauri-apps/plugin-dialog");
    const path = await save({
      defaultPath: "debug-bundle.zip",
      filters: [{ name: "ZIP Archive", extensions: ["zip"] }],
    });
    if (!path) return;

    isExporting = true;
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("export_debug_bundle", {
        destinationPath: path,
        redactPaths,
      });
    } catch (e) {
      console.error("Failed to export debug bundle:", e);
    } finally {
      isExporting = false;
    }
  }

  async function openLogDir() {
    try {
      const { open } = await import("@tauri-apps/plugin-shell");
      await open("~/.capability-repo-manager/logs/");
    } catch (e) {
      console.error("Failed to open log directory:", e);
    }
  }

  loadSettings();
</script>

<div class="settings-page">
  <header class="settings-header">
    <h1>{$_('settings.title')}</h1>
    <button class="back-btn" onclick={() => navigateTo("dashboard")}>{$_('nav.back')}</button>
  </header>

  {#if isLoading}
    <p class="loading">{$_('settings.loading')}</p>
  {:else}
    <form class="settings-form" onsubmit={(e) => { e.preventDefault(); saveSettings(); }}>
      <div class="form-group">
        <label>{$_('settings.scan_roots')}</label>
        <textarea
          bind:value={scanRoots}
          placeholder={$_('settings.scan_roots_placeholder')}
          rows={2}
        ></textarea>
        <span class="hint">{$_('settings.scan_roots_hint')}</span>
      </div>

      <div class="form-group">
        <label>{$_('settings.scan_depth')}</label>
        <input type="number" bind:value={scanDepth} min="1" max="20" />
        <span class="hint">{$_('settings.scan_depth_hint')}</span>
      </div>

      <div class="form-group">
        <label>{$_('settings.ignore_patterns')}</label>
        <textarea bind:value={ignorePatterns} rows={2}></textarea>
        <span class="hint">{$_('settings.ignore_patterns_hint')}</span>
      </div>

      <div class="form-group">
        <label>{$_('settings.pack_storage')}</label>
        <input type="text" bind:value={packStorageDir} placeholder={$_('settings.pack_storage_placeholder')} />
        <span class="hint">{$_('settings.pack_storage_hint')}</span>
      </div>

      <hr class="section-divider" />

      <div class="form-group">
        <label>{$_('settings.log_level')}</label>
        <select class="log-level-select" value={logLevel} onchange={(e) => setLogLevel((e.target as HTMLSelectElement).value)}>
          <option value="info">INFO</option>
          <option value="debug">DEBUG</option>
          <option value="trace">TRACE</option>
        </select>
        <span class="hint">{$_('settings.log_level_hint')}</span>
      </div>

      <div class="form-group">
        <label class="checkbox-label">
          <input type="checkbox" bind:checked={redactPaths} />
          {$_('settings.redact_paths')}
        </label>
        <span class="hint">{$_('settings.redact_paths_hint')}</span>
      </div>

      <div class="button-group">
        <button type="button" class="action-btn" onclick={exportDebugBundle} disabled={isExporting}>
          {isExporting ? $_('settings.exporting_bundle') : $_('settings.export_debug_bundle')}
        </button>
        <button type="button" class="action-btn" onclick={openLogDir}>
          {$_('settings.open_log_dir')}
        </button>
      </div>

      <div class="form-actions">
        <button type="submit" class="save-btn" disabled={isLoading}>{$_('settings.save')}</button>
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
  .form-group textarea,
  .form-group select {
    width: 100%;
    padding: 10px;
    border: 1px solid #e2e8f0;
    border-radius: 6px;
    font-size: 0.95rem;
    box-sizing: border-box;
  }
  .log-level-select {
    max-width: 200px;
  }
  .checkbox-label {
    display: flex !important;
    align-items: center;
    gap: 8px;
    cursor: pointer;
  }
  .checkbox-label input[type="checkbox"] {
    width: auto;
  }
  .hint {
    display: block;
    font-size: 0.8rem;
    color: #64748b;
    margin-top: 4px;
  }
  .section-divider {
    border: none;
    border-top: 1px solid #e2e8f0;
    margin: 4px 0;
  }
  .button-group {
    display: flex;
    gap: 12px;
    flex-wrap: wrap;
  }
  .action-btn {
    padding: 10px 20px;
    background: #f8fafc;
    border: 1px solid #e2e8f0;
    border-radius: 8px;
    font-size: 0.9rem;
    cursor: pointer;
    transition: background 0.15s;
  }
  .action-btn:hover {
    background: #f1f5f9;
  }
  .action-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
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
