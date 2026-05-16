<script lang="ts">
  import AppShell from '$lib/components/AppShell.svelte';
  import Dashboard from '$lib/pages/Dashboard.svelte';
  import RepoDetail from '$lib/pages/RepoDetail.svelte';
  import Settings from '$lib/pages/Settings.svelte';
  import GuidedSetup from '$lib/pages/GuidedSetup.svelte';
  import BootError from '$lib/components/BootError.svelte';
  import { currentPage } from '$lib/stores/uiStore';
  import { setupI18n, waitForI18n } from '$lib/i18n';
  import { onMount } from 'svelte';
  import { locale, _ } from 'svelte-i18n';

  // Lazy-loaded pages (loaded on demand)
  let PackExport: any = $state(null);
  let PackApply: any = $state(null);
  let Doctor: any = $state(null);
  let Compare: any = $state(null);
  let Activity: any = $state(null);

  let bootError: { title: string; message: string; detail: string } | null = $state(null);

  onMount(async () => {
    setupI18n();
    await waitForI18n();
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const repos = await invoke<any[]>('list_repositories', { filter: {} });
      if (repos.length === 0) {
        currentPage.set('guidedsetup');
      }
    } catch (e: any) {
      const msg = String(e?.message ?? e);
      // Categorize error
      if (
        msg.includes('not found') ||
        msg.includes('No function') ||
        msg.includes('not registered')
      ) {
        bootError = {
          title: $_('app.backend_unavailable_title'),
          message: $_('app.backend_unavailable_message'),
          detail: msg,
        };
      } else if (msg.includes('no such table') || msg.includes('database')) {
        bootError = {
          title: $_('app.database_error_title'),
          message: $_('app.database_error_message'),
          detail: msg,
        };
      } else {
        // First launch or empty state
        currentPage.set('guidedsetup');
      }
    }
  });

  async function loadPage(name: string) {
    if (name === 'packexport' && !PackExport) {
      PackExport = (await import('$lib/pages/PackExport.svelte')).default;
    } else if (name === 'packapply' && !PackApply) {
      PackApply = (await import('$lib/pages/PackApply.svelte')).default;
    } else if (name === 'doctor' && !Doctor) {
      Doctor = (await import('$lib/pages/Doctor.svelte')).default;
    } else if (name === 'compare' && !Compare) {
      Compare = (await import('$lib/pages/Compare.svelte')).default;
    } else if (name === 'activity' && !Activity) {
      Activity = (await import('$lib/pages/Activity.svelte')).default;
    }
  }

  // Pre-load page when currentPage changes
  $effect(() => {
    const page = $currentPage;
    loadPage(page);
  });

  function renderPage() {
    if (bootError) {
      return BootError;
    }
    if (!$locale) return null;

    const page = $currentPage;
    if (page === 'guidedsetup') return GuidedSetup;
    if (page === 'dashboard') return Dashboard;
    if (page === 'settings') return Settings;
    if (page === 'packexport') return PackExport;
    if (page === 'packapply') return PackApply;
    if (page === 'doctor') return Doctor;
    if (page === 'compare') return Compare;
    if (page === 'activity') return Activity;
    if (page.startsWith('repo:')) return null; // Handled inline
    return null;
  }
</script>

{#if bootError}
  <BootError title={bootError.title} message={bootError.message} detail={bootError.detail} />
{:else if !$locale}
  <div class="page-loading">Loading...</div>
{:else}
  <AppShell>
    {#if $currentPage === 'guidedsetup'}
      <GuidedSetup />
    {:else if $currentPage === 'dashboard'}
      <Dashboard />
    {:else if $currentPage === 'settings'}
      <Settings />
    {:else if $currentPage === 'packexport'}
      {#if PackExport}
        <PackExport />
      {:else}
        <div class="page-loading">{$_('app.loading')}</div>
      {/if}
    {:else if $currentPage === 'packapply'}
      {#if PackApply}
        <PackApply />
      {:else}
        <div class="page-loading">{$_('app.loading')}</div>
      {/if}
    {:else if $currentPage === 'doctor'}
      {#if Doctor}
        <Doctor />
      {:else}
        <div class="page-loading">{$_('app.loading')}</div>
      {/if}
    {:else if $currentPage === 'compare'}
      {#if Compare}
        <Compare />
      {:else}
        <div class="page-loading">{$_('app.loading')}</div>
      {/if}
    {:else if $currentPage === 'activity'}
      {#if Activity}
        <Activity />
      {:else}
        <div class="page-loading">{$_('app.loading')}</div>
      {/if}
    {:else if $currentPage.startsWith('repo:')}
      <RepoDetail repoId={$currentPage.slice(5)} />
    {:else}
      <BootError
        title={$_('app.unknown_page_title')}
        message={$_('app.unknown_page_message')}
        detail={$currentPage}
      />
    {/if}
  </AppShell>
{/if}

<style>
  .page-loading {
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: 60vh;
    color: var(--text-muted);
    font-size: var(--font-size-sm);
  }
</style>
