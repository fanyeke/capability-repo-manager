<script lang="ts">
  import { currentPage, navigateTo } from '$lib/stores/uiStore';
  import { _ } from 'svelte-i18n';
  import { showToast } from '$lib/stores/toastStore';
  import Button from '$lib/components/Button.svelte';
  import Card from '$lib/components/Card.svelte';

  let scanRoots = $state('');
  let scanDepth = $state('5');
  let ignorePatterns = $state('node_modules, .venv, vendor, .cache, build');
  let packStorageDir = $state('');
  let isLoading = $state(false);
  let logLevel = $state('info');

  async function loadSettings() {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const settings = await invoke<{
        scan_roots: string[];
        scan_depth: number;
        ignore_patterns: string[];
        pack_storage_dir: string;
        log_level: string;
      }>('get_settings');
      scanRoots = settings.scan_roots.join(', ');
      scanDepth = String(settings.scan_depth);
      ignorePatterns = settings.ignore_patterns.join(', ');
      packStorageDir = settings.pack_storage_dir;
      logLevel = settings.log_level;
    } catch (e) {
      console.error('Failed to load settings:', e);
    }
  }

  async function exportDebugBundle() {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('export_debug_bundle', { destinationPath: '', redactPaths: true });
      showToast($_('settings.debug_exported'), 'success');
    } catch (e) {
      showToast($_('settings.debug_export_failed') + ': ' + String(e), 'error');
    }
  }

  async function openLogDir() {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const { open } = await import('@tauri-apps/plugin-shell');
      const logDir = await invoke<string>('get_log_directory');
      await open(logDir);
    } catch (e) {
      console.error('Failed to open log dir:', e);
    }
  }

  async function saveSettings() {
    isLoading = true;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('update_settings', {
        newSettings: {
          scan_roots: scanRoots.split(/[,\s]+/).filter((p) => p.trim()),
          scan_depth: parseInt(scanDepth, 10) || 5,
          ignore_patterns: ignorePatterns.split(/[,\s]+/).filter((p) => p.trim()),
          pack_storage_dir: packStorageDir.trim() || '~/.capability-repo-manager/packs',
          file_watch_enabled: false,
          log_level: logLevel,
        },
      });
      showToast($_('settings.saved'), 'success');
    } catch (e) {
      showToast($_('settings.save_error') + ': ' + String(e), 'error');
    } finally {
      isLoading = false;
    }
  }

  const logLevels = [
    { value: 'error', label: 'Error' },
    { value: 'warn', label: 'Warn' },
    { value: 'info', label: 'Info' },
    { value: 'debug', label: 'Debug' },
    { value: 'trace', label: 'Trace' },
  ];

  loadSettings();
</script>

<div class="settings-page">
  <header class="settings-header">
    <h1>{$_('settings.title')}</h1>
    <button class="back-btn" onclick={() => navigateTo('dashboard')}>{$_('nav.back')}</button>
  </header>

  <form class="settings-form" onsubmit={(e) => { e.preventDefault(); saveSettings(); }}>
    <div class="form-group">
      <label>{$_('settings.scan_roots')}</label>
      <input type="text" bind:value={scanRoots} placeholder={$_('settings.scan_roots_placeholder')} />
      <span class="hint">{$_('settings.scan_roots_hint')}</span>
    </div>

    <div class="form-group">
      <label>{$_('settings.scan_depth')}</label>
      <input type="number" bind:value={scanDepth} min="1" max="20" />
      <span class="hint">{$_('settings.scan_depth_hint')}</span>
    </div>

    <div class="form-group">
      <label>{$_('settings.ignore_patterns')}</label>
      <input type="text" bind:value={ignorePatterns} placeholder="node_modules, .venv, target" />
      <span class="hint">{$_('settings.ignore_patterns_hint')}</span>
    </div>

    <div class="form-group">
      <label>{$_('settings.pack_storage')}</label>
      <input type="text" bind:value={packStorageDir} placeholder="~/.capability-repo-manager/packs" />
      <span class="hint">{$_('settings.pack_storage_hint')}</span>
    </div>

    <div class="form-group">
      <label>{$_('settings.log_level')}</label>
      <select bind:value={logLevel} class="log-level-select">
        {#each logLevels as lvl (lvl.value)}
          <option value={lvl.value}>{lvl.label}</option>
        {/each}
      </select>
      <span class="hint">{$_('settings.log_level_hint')}</span>
    </div>

    <hr class="section-divider" />

    <div class="form-group">
      <label>{$_('settings.debug_title')}</label>
      <div class="button-group">
        <button type="button" class="action-btn" onclick={exportDebugBundle}>
          {$_('settings.export_debug')}
        </button>
        <button type="button" class="action-btn" onclick={openLogDir}>
          {$_('settings.open_log_dir')}
        </button>
      </div>
    </div>

    <div class="form-actions">
      <button type="submit" class="save-btn" disabled={isLoading}>
        {isLoading ? $_('common.saving') : $_('common.save')}
      </button>
    </div>
  </form>
</div>

<style>
  .settings-page {
    max-width: 600px;
    margin: 0 auto;
    padding: var(--space-6);
  }
  .settings-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--space-6);
  }
  .settings-header h1 { margin: 0; }
  .back-btn {
    padding: var(--space-2) var(--space-4);
    background: var(--bg-card);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    cursor: pointer;
    color: var(--text-secondary);
    font-size: var(--font-size-sm);
  }
  .settings-form {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
    background: var(--bg-card);
    padding: var(--space-6);
    border-radius: var(--radius-lg);
    border: 1px solid var(--border-default);
  }
  .form-group label {
    display: block;
    font-weight: 500;
    margin-bottom: var(--space-2);
    color: var(--text-secondary);
    font-size: var(--font-size-sm);
  }
  .form-group input,
  .form-group textarea,
  .form-group select {
    width: 100%;
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    font-size: var(--font-size-sm);
    background: var(--bg-panel);
    color: var(--text-primary);
    outline: none;
    box-sizing: border-box;
  }
  .form-group input:focus,
  .form-group select:focus {
    border-color: var(--color-primary);
  }
  .log-level-select { max-width: 200px; }
  .hint {
    display: block;
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    margin-top: var(--space-1);
  }
  .section-divider {
    border: none;
    border-top: 1px solid var(--border-default);
    margin: var(--space-1) 0;
  }
  .button-group {
    display: flex;
    gap: var(--space-3);
    flex-wrap: wrap;
  }
  .action-btn {
    padding: var(--space-2) var(--space-4);
    background: var(--bg-elevated);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    color: var(--text-secondary);
    font-size: var(--font-size-sm);
    cursor: pointer;
    transition: background var(--transition-fast);
  }
  .action-btn:hover {
    background: var(--bg-hover);
  }
  .action-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
  .form-actions { margin-top: var(--space-2); }
  .save-btn {
    padding: var(--space-3) var(--space-6);
    background: var(--color-primary);
    color: #fff;
    border: none;
    border-radius: var(--radius-md);
    font-weight: 600;
    cursor: pointer;
    font-size: var(--font-size-sm);
  }
  .save-btn:hover { background: var(--color-primary-hover); }
  .save-btn:disabled { opacity: 0.6; cursor: not-allowed; }
</style>
