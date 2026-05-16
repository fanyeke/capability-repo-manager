<script lang="ts">
  import { currentPage, navigateTo, theme, reducedMotion } from '$lib/stores/uiStore';
  import { _ } from 'svelte-i18n';
  import { showToast } from '$lib/stores/toastStore';
  import { ArrowLeft } from 'lucide-svelte';
  import Button from '$lib/components/Button.svelte';
  import Card from '$lib/components/Card.svelte';
  import Field from '$lib/components/Field.svelte';
  import DirectoryPicker from '$lib/components/DirectoryPicker.svelte';
  import { zoomLevel as zoomStore, zoomIn, zoomOut, resetZoom } from '$lib/stores/zoomStore';
  import { onMount } from 'svelte';
  import { get } from 'svelte/store';

  interface SettingsData {
    scan_roots: string[];
    scan_depth: number;
    ignore_patterns: string[];
    pack_storage_dir: string;
    log_level: string;
    theme: string;
    reduced_motion: boolean;
  }

  let data: SettingsData = $state({
    scan_roots: [],
    scan_depth: 5,
    ignore_patterns: ['node_modules', '.venv', 'vendor', '.cache', 'build', 'target'],
    pack_storage_dir: '~/.capability-repo-manager/packs',
    log_level: 'info',
    theme: 'dark',
    reduced_motion: false,
  });

  let activeGroup = $state<string>('scan');
  let isLoading = $state(false);
  let newRoot = $state('');
  let selectedZoom = $state('100');
  onMount(() => { selectedZoom = String(get(zoomStore)); });

  const groups = [
    { id: 'scan', label: $_('settings.group_scan') },
    { id: 'storage', label: $_('settings.group_storage') },
    { id: 'appearance', label: $_('settings.group_appearance') },
    { id: 'logging', label: $_('settings.group_logging') },
  ];

  const logLevels = [
    { value: 'error', label: 'Error' },
    { value: 'warn', label: 'Warn' },
    { value: 'info', label: 'Info' },
    { value: 'debug', label: 'Debug' },
    { value: 'trace', label: 'Trace' },
  ];

  onMount(loadSettings);

  async function loadSettings() {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const settings = await invoke<SettingsData>('get_settings');
      data = { ...data, ...settings };
    } catch (e) {
      console.error('Failed to load settings:', e);
    }
  }

  async function saveSettings() {
    isLoading = true;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('update_settings', {
        newSettings: {
          scan_roots: data.scan_roots,
          scan_depth: data.scan_depth,
          ignore_patterns: data.ignore_patterns,
          pack_storage_dir: data.pack_storage_dir.trim() || '~/.capability-repo-manager/packs',
          file_watch_enabled: false,
          log_level: data.log_level,
          theme: data.theme,
          reduced_motion: data.reduced_motion,
        },
      });
      // Apply theme/motion immediately
      theme.set(data.theme as 'dark' | 'light' | 'system');
      reducedMotion.set(data.reduced_motion);
      showToast($_('settings.saved'), 'success');
    } catch (e) {
      showToast($_('settings.save_error') + ': ' + String(e), 'error');
    } finally {
      isLoading = false;
    }
  }

  function addRoot() {
    const p = newRoot.trim();
    if (p && !data.scan_roots.includes(p)) {
      data.scan_roots = [...data.scan_roots, p];
      newRoot = '';
    }
  }

  function removeRoot(p: string) {
    data.scan_roots = data.scan_roots.filter((r) => r !== p);
  }

  async function exportDebugBundle() {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const { save } = await import('@tauri-apps/plugin-dialog');
      const dest = await save({
        defaultPath: 'debug-bundle.zip',
        filters: [{ name: 'ZIP Archive', extensions: ['zip'] }],
      });
      if (!dest) return; // user cancelled
      await invoke('export_debug_bundle', { destinationPath: dest, redactPaths: true });
      showToast($_('settings.debug_exported'), 'success');
    } catch (e) {
      showToast($_('settings.debug_export_failed') + ': ' + String(e), 'error');
    }
  }

</script>

<div class="settings-page">
  <div class="page-top">
    <Button variant="ghost" size="sm" onclick={() => navigateTo('dashboard')}>
      <ArrowLeft size={14} /> {$_('nav.back')}
    </Button>
    <h1>{$_('settings.title')}</h1>
  </div>

  <div class="settings-layout">
    <aside class="settings-nav">
      {#each groups as g (g.id)}
        <button class="group-btn" class:active={activeGroup === g.id} onclick={() => activeGroup = g.id as typeof activeGroup}>
          {g.label}
        </button>
      {/each}
    </aside>

    <main class="settings-main">
      <form class="settings-form" onsubmit={(e) => { e.preventDefault(); saveSettings(); }}>
        {#if activeGroup === 'scan'}
          <h2>{$_('settings.group_scan')}</h2>
          <Field label={$_('settings.scan_roots')} hint={$_('settings.scan_roots_hint')}>
            <DirectoryPicker directories={data.scan_roots} onChange={(dirs) => data.scan_roots = dirs} />
          </Field>
          <Field label={$_('settings.scan_depth')} hint={$_('settings.scan_depth_hint')}>
            <div class="depth-row">
              <input type="range" bind:value={data.scan_depth} min="1" max="10" class="depth-slider" />
              <span class="depth-value">{data.scan_depth}</span>
            </div>
          </Field>
          <Field label={$_('settings.ignore_patterns')} hint={$_('settings.ignore_patterns_hint')}>
            <input type="text" value={data.ignore_patterns.join(', ')} class="form-input"
              oninput={(e) => data.ignore_patterns = (e.target as HTMLInputElement).value.split(/[,\s]+/).filter(Boolean)} />
          </Field>

        {:else if activeGroup === 'storage'}
          <h2>{$_('settings.group_storage')}</h2>
          <Field label={$_('settings.pack_storage')} hint={$_('settings.pack_storage_hint')}>
            <input type="text" bind:value={data.pack_storage_dir} class="form-input" />
          </Field>
          <div class="button-group">
            <Button variant="secondary" size="sm" onclick={exportDebugBundle}>{$_('settings.export_debug')}</Button>
          </div>

        {:else if activeGroup === 'appearance'}
          <h2>{$_('settings.group_appearance')}</h2>
          <Field label={$_('settings.theme')}>
            <select bind:value={data.theme} class="form-select">
              <option value="dark">Dark</option>
              <option value="light">Light</option>
              <option value="system">System</option>
            </select>
          </Field>
          <Field label={$_('settings.zoom_level')}>
            <select bind:value={selectedZoom} class="form-select log-select" onchange={() => zoomStore.set(Number(selectedZoom))}>
              <option value="60">60%</option>
              <option value="80">80%</option>
              <option value="90">90%</option>
              <option value="100">100%</option>
              <option value="110">110%</option>
              <option value="120">120%</option>
              <option value="150">150%</option>
            </select>
          </Field>
          <Field label={$_('settings.reduced_motion')}>
            <label class="checkbox-row">
              <input type="checkbox" bind:checked={data.reduced_motion} />
              {$_('settings.reduced_motion_hint')}
            </label>
          </Field>

        {:else if activeGroup === 'logging'}
          <h2>{$_('settings.group_logging')}</h2>
          <Field label={$_('settings.log_level')} hint={$_('settings.log_level_hint')}>
            <select bind:value={data.log_level} class="form-select log-select">
              {#each logLevels as lvl (lvl.value)}
                <option value={lvl.value}>{lvl.label}</option>
              {/each}
            </select>
          </Field>
          <div class="button-group">
            <Button variant="secondary" size="sm" onclick={async () => { try { const { invoke } = await import('@tauri-apps/api/core'); const { open } = await import('@tauri-apps/plugin-shell'); const logDir = await invoke<string>('get_log_directory'); await open(logDir); } catch {} } }>{$_('settings.open_log_dir')}</Button>
          </div>
        {/if}

        <div class="form-actions">
          <button class="submit-btn" disabled={isLoading}><span>{isLoading ? '...' : $_('common.save')}</span></button>
        </div>
      </form>
    </main>

    <aside class="settings-preview">
      <Card padding="md">
        {#snippet title()}{$_('settings.preview')}{/snippet}
        <div class="preview-stats">
          <div class="pstat"><span class="pstat-label">{$_('settings.scan_roots')}</span><span class="pstat-value">{data.scan_roots.length}</span></div>
          <div class="pstat"><span class="pstat-label">{$_('settings.ignore_patterns')}</span><span class="pstat-value">{data.ignore_patterns.length} patterns</span></div>
          <div class="pstat"><span class="pstat-label">{$_('settings.scan_depth')}</span><span class="pstat-value">{data.scan_depth}</span></div>
          <div class="pstat"><span class="pstat-label">{$_('settings.log_level')}</span><span class="pstat-value">{data.log_level}</span></div>
          <div class="pstat"><span class="pstat-label">{$_('settings.theme')}</span><span class="pstat-value">{data.theme}</span></div>
          <div class="pstat"><span class="pstat-label">{$_('settings.reduced_motion')}</span><span class="pstat-value">{data.reduced_motion ? '✓' : '—'}</span></div>
        </div>
      </Card>
    </aside>
  </div>
</div>

<style>
  .settings-page {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding-top: var(--space-4);
  }
  .page-top {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }
  .page-top h1 { margin: 0; font-size: var(--font-size-xl); }

  .settings-layout {
    display: grid;
    grid-template-columns: 180px 1fr 240px;
    gap: var(--space-4);
  }
  .settings-nav {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .group-btn {
    text-align: left;
    padding: var(--space-2) var(--space-3);
    background: none;
    border: none;
    border-radius: var(--radius-md);
    color: var(--text-secondary);
    font-size: var(--font-size-sm);
    cursor: pointer;
    transition: background var(--transition-fast), color var(--transition-fast);
  }
  .group-btn:hover { background: var(--bg-hover); color: var(--text-primary); }
  .group-btn.active { background: var(--color-primary-bg); color: var(--color-primary-text); }

  .settings-form h2 {
    margin: 0 0 var(--space-4);
    font-size: var(--font-size-md);
    font-weight: 600;
  }
  .settings-form {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
  }
  .chips { display: flex; flex-wrap: wrap; gap: var(--space-1); margin-bottom: var(--space-2); }
  .chip {
    display: inline-flex; align-items: center; gap: 4px;
    padding: 2px 8px; background: var(--color-primary-bg); color: var(--color-primary-text);
    border-radius: var(--radius-sm); font-size: var(--font-size-xs);
  }
  .chip-remove { background: none; border: none; color: inherit; cursor: pointer; padding: 0; font-size: 14px; line-height: 1; }
  .add-row { display: flex; gap: var(--space-2); }
  .form-input {
    flex: 1; padding: 8px 12px; background: var(--bg-card);
    border: 1px solid var(--border-default); border-radius: var(--radius-md);
    color: var(--text-primary); font-size: var(--font-size-sm); outline: none;
  }
  .form-input:focus { border-color: var(--color-primary); }
  .form-select {
    padding: 8px 12px; background: var(--bg-card);
    border: 1px solid var(--border-default); border-radius: var(--radius-md);
    color: var(--text-primary); font-size: var(--font-size-sm);
  }
  .log-select { max-width: 200px; }
  .depth-row { display: flex; align-items: center; gap: var(--space-3); }
  .depth-slider { flex: 1; accent-color: var(--color-primary); }
  .depth-value { font-size: var(--font-size-sm); min-width: 24px; text-align: center; }
  .checkbox-row { display: flex; align-items: center; gap: var(--space-2); cursor: pointer; font-size: var(--font-size-sm); color: var(--text-secondary); }
  .button-group { display: flex; gap: var(--space-2); }
  .form-actions { margin-top: var(--space-2); }
  .submit-btn { padding: var(--space-2) var(--space-5); background: var(--color-primary); border: none; border-radius: var(--radius-md); color: #fff; font-size: var(--font-size-sm); cursor: pointer; } .submit-btn:disabled { opacity: 0.5; }

  .settings-preview { display: flex; flex-direction: column; gap: var(--space-3); }
  .preview-stats { display: flex; flex-direction: column; gap: var(--space-2); }
  .pstat { display: flex; justify-content: space-between; font-size: var(--font-size-xs); }
  .pstat-label { color: var(--text-muted); }
  .pstat-value { color: var(--text-primary); font-weight: 500; }
</style>
