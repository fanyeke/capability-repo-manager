<script lang="ts">
  import { currentPage, navigateTo } from '$lib/stores/uiStore';
  import { _ } from 'svelte-i18n';
  import {
    LayoutDashboard, GitBranch, Package, ArrowRightLeft, Stethoscope,
    GitCompare, Settings, type Icon as IconType,
  } from 'lucide-svelte';

  interface NavItem {
    id: string;
    label: string;
    icon: typeof IconType;
  }

  const navItems: NavItem[] = [
    { id: 'dashboard', label: $_('nav.overview'), icon: LayoutDashboard },
    { id: 'repositories', label: $_('nav.repositories'), icon: GitBranch },
    { id: 'packs', label: $_('nav.packs'), icon: Package },
    { id: 'migration', label: $_('nav.migration'), icon: ArrowRightLeft },
    { id: 'doctor', label: $_('nav.doctor'), icon: Stethoscope },
    { id: 'compare', label: $_('nav.compare'), icon: GitCompare },
    { id: 'settings', label: $_('nav.settings'), icon: Settings },
  ];

  // Map sidebar nav IDs to their corresponding page route
  const navRouteMap: Record<string, string> = {
    dashboard: 'dashboard',
    repositories: 'repositories',
    packs: 'packexport',
    migration: 'migration',
    doctor: 'doctor',
    compare: 'compare',
    settings: 'settings',
  };

  function isActive(id: string): boolean {
    const page = $currentPage;
    // Exact match or route prefix match (e.g. repo:xxx starts with repo)
    if (id === 'repositories') return page.startsWith('repo:') || page === 'repositories';
    return page === navRouteMap[id] || page.startsWith(id + ':');
  }
</script>

<aside class="sidebar">
  <div class="sidebar-brand">
    <span class="brand-icon">◆</span>
    <span class="brand-name">CRM</span>
  </div>

  <nav class="sidebar-nav">
    {#each navItems as item (item.id)}
      <button
        class="nav-item"
        class:active={isActive(item.id)}
        onclick={() => navigateTo(navRouteMap[item.id] || item.id)}
      >
        <svelte:component this={item.icon} size={16} />
        <span class="nav-label">{item.label}</span>
      </button>
    {/each}
  </nav>

  <div class="sidebar-footer">
    <div class="engine-status">
      <span class="status-dot"></span>
      <span class="status-text">{$_('nav.engine_ready')}</span>
    </div>
  </div>
</aside>

<style>
  .sidebar {
    width: var(--sidebar-width);
    background: var(--bg-sidebar);
    border-right: 1px solid var(--border-default);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
    overflow-y: auto;
  }
  .sidebar-brand {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-4) var(--space-4);
    border-bottom: 1px solid var(--border-default);
    font-weight: 600;
    font-size: var(--font-size-lg);
  }
  .brand-icon {
    color: var(--color-primary);
    font-size: 1.2rem;
  }
  .brand-name {
    letter-spacing: 0.5px;
  }
  .sidebar-nav {
    flex: 1;
    padding: var(--space-2);
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .nav-item {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    border: none;
    background: none;
    color: var(--text-secondary);
    font-size: var(--font-size-sm);
    cursor: pointer;
    border-radius: var(--radius-md);
    transition: background var(--transition-fast), color var(--transition-fast);
    text-align: left;
    width: 100%;
  }
  .nav-item:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }
  .nav-item.active {
    background: var(--color-primary-bg);
    color: var(--color-primary-text);
  }
  .nav-label {
    flex: 1;
  }
  .sidebar-footer {
    padding: var(--space-3) var(--space-4);
    border-top: 1px solid var(--border-default);
  }
  .engine-status {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--font-size-xs);
    color: var(--text-muted);
  }
  .status-dot {
    width: 6px;
    height: 6px;
    border-radius: var(--radius-full);
    background: var(--color-success);
  }
  .status-text {
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
</style>
