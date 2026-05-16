<script lang="ts">
  import { scanRepositories } from '$lib/stores/repoStore';
  import { currentPage } from '$lib/stores/uiStore';
  import { _ } from 'svelte-i18n';
  import { X, Scan, Settings as SettingsIcon, Check } from 'lucide-svelte';
  import Button from '$lib/components/Button.svelte';
  import Card from '$lib/components/Card.svelte';
  import Banner from '$lib/components/Banner.svelte';
  import Skeleton from '$lib/components/Skeleton.svelte';

  let step = $state<'paths' | 'config' | 'scanning'>('paths');
  let scanPaths = $state('');
  let paths: string[] = $state([]);
  let ignorePatterns = $state('node_modules,.venv,target');
  let scanDepth = $state(5);
  let isScanning = $state(false);
  let scanError = $state<string | null>(null);

  function addPaths() {
    const items = scanPaths.split(/[,\s\n]+/).map((p) => p.trim()).filter((p) => p);
    const unique = items.filter((p) => !paths.includes(p));
    paths = [...paths, ...unique];
    scanPaths = '';
  }

  function removePath(p: string) {
    paths = paths.filter((x) => x !== p);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      addPaths();
    }
  }

  async function handleScan() {
    if (paths.length === 0) return;
    isScanning = true;
    scanError = null;
    step = 'scanning';
    try {
      await scanRepositories(paths);
      // Persist scan roots
      const { invoke } = await import('@tauri-apps/api/core');
      const currentSettings = await invoke<{ scan_roots: string[]; scan_depth: number; ignore_patterns: string[] }>('get_settings');
      await invoke('update_settings', {
        newSettings: {
          ...currentSettings,
          scan_roots: paths,
          scan_depth: scanDepth,
          ignore_patterns: ignorePatterns.split(/[,\s]+/).filter((p) => p),
        },
      });
      currentPage.set('dashboard');
    } catch (e: any) {
      scanError = e?.message ?? String(e);
      isScanning = false;
      step = 'config';
    }
  }
</script>

<div class="guided-setup">
  <div class="setup-layout">
    <aside class="setup-brand">
      <div class="brand-header">
        <span class="brand-icon">◆</span>
        <h1 class="brand-title">{$_('app.title')}</h1>
      </div>
      <p class="brand-desc">{$_('guided_setup.intro')}</p>
      <div class="brand-steps">
        {#each [
          { id: 'paths', label: $_('guided_setup.step_paths') },
          { id: 'config', label: $_('guided_setup.step_config') },
          { id: 'scanning', label: $_('guided_setup.step_scan') },
        ] as s}
          <div class="step-indicator" class:active={step === s.id} class:done={['config', 'scanning'].includes(step) && ['paths'].includes(s.id) || step === 'scanning' && s.id === 'config'}>
            <span class="step-number">
              {#if step === 'scanning' && s.id === 'scanning'}
                <span class="spinner"></span>
              {:else if ['config', 'scanning'].includes(step) && s.id === 'paths' || step === 'scanning' && s.id === 'config'}
                <Check size={12} />
              {:else}
                {['paths', 'config', 'scanning'].indexOf(s.id) + 1}
              {/if}
            </span>
            <span class="step-label">{s.label}</span>
          </div>
        {/each}
      </div>
    </aside>

    <main class="setup-main">
      {#if scanError}
        <Banner type="error" dismissible={false}>
          {$_('guided_setup.scan_error')}: {scanError}
        </Banner>
      {/if}

      {#if step === 'paths'}
        <Card padding="lg">
          {#snippet title()}<h2>{$_('guided_setup.add_scan_dirs')}</h2>{/snippet}
          <div class="input-group">
            <div class="chips">
              {#each paths as p (p)}
                <span class="chip">
                  {p}
                  <button class="chip-remove" onclick={() => removePath(p)} aria-label="Remove">
                    <X size={12} />
                  </button>
                </span>
              {/each}
            </div>
            <div class="path-input-row">
              <input
                type="text"
                bind:value={scanPaths}
                placeholder={$_('guided_setup.scan_placeholder')}
                onkeydown={handleKeydown}
                class="path-input"
              />
              <Button variant="secondary" size="sm" onclick={addPaths} disabled={!scanPaths.trim()}>
                {$_('common.add')}
              </Button>
            </div>
            <p class="hint">{$_('guided_setup.paths_hint')}</p>
          </div>
          <div class="step-actions">
            <Button variant="ghost" onclick={() => currentPage.set('dashboard')}>
              {$_('guided_setup.skip')}
            </Button>
            <Button variant="primary" onclick={() => step = 'config'} disabled={paths.length === 0}>
              {$_('common.next')}
            </Button>
          </div>
        </Card>

      {:else if step === 'config'}
        <Card padding="lg">
          {#snippet title()}<h2>{$_('guided_setup.config_title')}</h2>{/snippet}
          <div class="config-group">
            <label class="config-label">{$_('guided_setup.ignore_patterns')}</label>
            <input
              type="text"
              bind:value={ignorePatterns}
              class="config-input"
            />
            <p class="hint">{$_('guided_setup.ignore_hint')}</p>
          </div>
          <div class="config-group">
            <label class="config-label">{$_('guided_setup.scan_depth')}</label>
            <div class="depth-row">
              <input
                type="range"
                bind:value={scanDepth}
                min="1"
                max="10"
                class="depth-slider"
              />
              <span class="depth-value">{scanDepth}</span>
            </div>
          </div>
          <div class="step-actions">
            <Button variant="ghost" onclick={() => step = 'paths'}>
              {$_('common.back')}
            </Button>
            <Button variant="primary" onclick={handleScan} disabled={paths.length === 0}>
              <Scan size={14} />
              {$_('guided_setup.scan')}
            </Button>
          </div>
        </Card>

      {:else if step === 'scanning'}
        <Card padding="lg">
          {#snippet title()}<h2>{$_('guided_setup.scanning_title')}</h2>{/snippet}
          <div class="scanning-status">
            <Skeleton variant="card" />
            <Skeleton variant="text" />
            <Skeleton variant="text" />
          </div>
          <p class="scanning-hint">{$_('guided_setup.scanning_hint', { values: { count: paths.length } })}</p>
        </Card>
      {/if}

      <div class="setup-info">
        <h3>{$_('guided_setup.what_discovered')}</h3>
        <ul>
          <li><strong>Skills</strong> — {$_('guided_setup.skills')}</li>
          <li><strong>MCP Servers</strong> — {$_('guided_setup.mcp')}</li>
          <li><strong>Hooks</strong> — {$_('guided_setup.hooks')}</li>
          <li><strong>Rules</strong> — {$_('guided_setup.rules')}</li>
          <li><strong>Agents</strong> — {$_('guided_setup.agents')}</li>
          <li><strong>Commands</strong> — {$_('guided_setup.commands')}</li>
        </ul>
      </div>
    </main>
  </div>
</div>

<style>
  .guided-setup {
    min-height: calc(100vh - var(--header-height));
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-6);
  }
  .setup-layout {
    display: flex;
    gap: var(--space-8);
    max-width: 800px;
    width: 100%;
  }
  .setup-brand {
    flex: 0 0 240px;
    padding-top: var(--space-4);
  }
  .brand-header {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: var(--space-2);
  }
  .brand-icon {
    font-size: 1.5rem;
    color: var(--color-primary);
  }
  .brand-title {
    font-size: var(--font-size-lg);
    font-weight: 700;
    margin: 0;
  }
  .brand-desc {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    line-height: 1.5;
    margin-bottom: var(--space-6);
  }
  .brand-steps {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }
  .step-indicator {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--font-size-sm);
    color: var(--text-muted);
  }
  .step-indicator.active {
    color: var(--text-primary);
  }
  .step-indicator.done {
    color: var(--color-success);
  }
  .step-number {
    width: 24px;
    height: 24px;
    border-radius: var(--radius-full);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 11px;
    font-weight: 600;
    border: 1px solid var(--border-default);
    flex-shrink: 0;
    background: var(--bg-card);
  }
  .step-indicator.active .step-number {
    border-color: var(--color-primary);
    color: var(--color-primary);
  }
  .step-indicator.done .step-number {
    border-color: var(--color-success);
    background: var(--color-success);
    color: #fff;
  }
  .spinner {
    width: 10px;
    height: 10px;
    border: 2px solid var(--color-primary);
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
    display: block;
  }
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
  .setup-main {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }
  .input-group {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1);
  }
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    background: var(--color-primary-bg);
    color: var(--color-primary-text);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-xs);
  }
  .chip-remove {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    color: inherit;
    cursor: pointer;
    padding: 0;
    opacity: 0.7;
  }
  .chip-remove:hover { opacity: 1; }
  .path-input-row {
    display: flex;
    gap: var(--space-2);
  }
  .path-input {
    flex: 1;
    padding: 8px 12px;
    background: var(--bg-card);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    color: var(--text-primary);
    font-size: var(--font-size-sm);
    font-family: var(--font-sans);
  }
  .path-input:focus {
    outline: none;
    border-color: var(--color-primary);
  }
  .hint {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    margin: 0;
  }
  .step-actions {
    display: flex;
    justify-content: space-between;
    margin-top: var(--space-4);
  }
  .config-group {
    margin-bottom: var(--space-4);
  }
  .config-label {
    display: block;
    font-size: var(--font-size-sm);
    font-weight: 500;
    color: var(--text-secondary);
    margin-bottom: var(--space-1);
  }
  .config-input {
    width: 100%;
    padding: 8px 12px;
    background: var(--bg-card);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    color: var(--text-primary);
    font-size: var(--font-size-sm);
  }
  .config-input:focus {
    outline: none;
    border-color: var(--color-primary);
  }
  .depth-row {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }
  .depth-slider {
    flex: 1;
    accent-color: var(--color-primary);
  }
  .depth-value {
    font-size: var(--font-size-sm);
    color: var(--text-primary);
    min-width: 24px;
    text-align: center;
  }
  .scanning-status {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }
  .scanning-hint {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    text-align: center;
    margin: var(--space-3) 0 0;
  }
  .setup-info {
    background: var(--bg-card);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    padding: var(--space-4);
    font-size: var(--font-size-sm);
    color: var(--text-muted);
  }
  .setup-info h3 {
    margin: 0 0 var(--space-2);
    font-size: var(--font-size-sm);
    font-weight: 600;
    color: var(--text-secondary);
  }
  .setup-info ul {
    margin: 0;
    padding-left: var(--space-4);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .setup-info li strong {
    color: var(--text-secondary);
  }
</style>
