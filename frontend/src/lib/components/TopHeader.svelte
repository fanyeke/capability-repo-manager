<script lang="ts">
  import { _ } from 'svelte-i18n';
  import { Scan, CircleHelp, Sun, Moon, Monitor } from 'lucide-svelte';
  import { scanRepositories, isScanning } from '$lib/stores/repoStore';
  import { currentPage, theme } from '$lib/stores/uiStore';
  import { zoomLevel, resetZoom } from '$lib/stores/zoomStore';

  const themeIcons = { dark: Moon, light: Sun, system: Monitor };

  function cycleTheme() {
    const current = $theme;
    const next = current === 'dark' ? 'light' : current === 'light' ? 'system' : 'dark';
    theme.set(next);
  }

  async function handleScan() {
    const { invoke } = await import('@tauri-apps/api/core');
    const settings = await invoke<{ scan_roots: string[] }>('get_settings');
    if (settings.scan_roots.length > 0) {
      scanRepositories(settings.scan_roots).catch(() => {});
    } else {
      currentPage.set('guidedsetup');
    }
  }
</script>

<header class="top-header">
  <div class="header-left"></div>
  <div class="header-right">
    <button class="header-btn" onclick={handleScan} disabled={$isScanning}>
      <Scan size={14} />
      <span>{$isScanning ? $_('dashboard.scanning') : $_('dashboard.scan')}</span>
    </button>
    <button class="header-btn" onclick={cycleTheme} title={$_('settings.theme')}>
      <svelte:component this={themeIcons[$theme]} size={14} />
    </button>
    <button class="header-btn icon-only" onclick={resetZoom} title="Reset zoom">
      <span class="zoom-text">{$zoomLevel}%</span>
    </button>
    <button class="header-btn icon-only" title={$_('common.help')}>
      <CircleHelp size={14} />
    </button>
  </div>
</header>

<style>
  .top-header {
    height: var(--header-height);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 var(--space-4);
    border-bottom: 1px solid var(--border-default);
    background: var(--bg-panel);
    flex-shrink: 0;
  }
  .header-left, .header-right { display: flex; align-items: center; gap: var(--space-2); }
  .header-btn {
    display: flex; align-items: center; gap: 6px;
    padding: 6px 12px; border: 1px solid var(--border-default);
    background: var(--bg-card); color: var(--text-secondary);
    font-size: var(--font-size-sm); border-radius: var(--radius-md);
    cursor: pointer;
    transition: background var(--transition-fast), color var(--transition-fast);
  }
  .header-btn:hover { background: var(--bg-hover); color: var(--text-primary); }
  .header-btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .header-btn.icon-only { padding: 6px; width: 28px; height: 28px; justify-content: center; }
  .zoom-text { font-size: var(--font-size-xs); font-family: var(--font-mono); }
</style>
