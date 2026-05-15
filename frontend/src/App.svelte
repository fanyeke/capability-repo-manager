<script lang="ts">
  import Dashboard from "$lib/pages/Dashboard.svelte";
  import RepoDetail from "$lib/pages/RepoDetail.svelte";
  import Settings from "$lib/pages/Settings.svelte";
  import GuidedSetup from "$lib/pages/GuidedSetup.svelte";
  import BootError from "$lib/components/BootError.svelte";
  import { currentPage } from "$lib/stores/uiStore";
  import { setupI18n, waitForI18n } from "$lib/i18n";
  import { onMount } from "svelte";
  import { locale, _ } from "svelte-i18n";

  // Lazy-loaded pages (loaded on demand)
  let PackExport: any = $state(null);
  let PackApply: any = $state(null);
  let Doctor: any = $state(null);
  let Compare: any = $state(null);

  let bootError: { title: string; message: string; detail: string } | null = $state(null);

  onMount(async () => {
    setupI18n();
    await waitForI18n();
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const repos = await invoke<any[]>("list_repositories", { filter: {} });
      if (repos.length === 0) {
        currentPage.set("guidedsetup");
      }
    } catch (e: any) {
      const msg = String(e?.message ?? e);
      // Categorize error
      if (msg.includes("not found") || msg.includes("No function") || msg.includes("not registered")) {
        bootError = {
          title: "Backend Unavailable",
          message: "The Tauri backend commands are not responding. This may be a build or configuration issue.",
          detail: msg,
        };
      } else if (msg.includes("no such table") || msg.includes("database")) {
        bootError = {
          title: "Database Error",
          message: "The application database could not be accessed.",
          detail: msg,
        };
      } else {
        // First launch or empty state
        currentPage.set("guidedsetup");
      }
    }
  });

  async function loadPage(name: string) {
    if (name === "packexport" && !PackExport) {
      PackExport = (await import("$lib/pages/PackExport.svelte")).default;
    } else if (name === "packapply" && !PackApply) {
      PackApply = (await import("$lib/pages/PackApply.svelte")).default;
    } else if (name === "doctor" && !Doctor) {
      Doctor = (await import("$lib/pages/Doctor.svelte")).default;
    } else if (name === "compare" && !Compare) {
      Compare = (await import("$lib/pages/Compare.svelte")).default;
    }
  }

  // Pre-load page when currentPage changes
  $effect(() => {
    const page = $currentPage;
    loadPage(page);
  });
</script>

<main>
  {#if bootError}
    <BootError title={bootError.title} message={bootError.message} detail={bootError.detail} />
  {:else if !$locale}
    <div class="page-loading">Loading...</div>
  {:else if $currentPage === "guidedsetup"}
    <GuidedSetup />
  {:else if $currentPage === "dashboard"}
    <Dashboard />
  {:else if $currentPage === "settings"}
    <Settings />
  {:else if $currentPage === "packexport"}
    {#if PackExport}
      <PackExport />
    {:else}
      <div class="page-loading">Loading...</div>
    {/if}
  {:else if $currentPage === "packapply"}
    {#if PackApply}
      <PackApply />
    {:else}
      <div class="page-loading">Loading...</div>
    {/if}
  {:else if $currentPage === "doctor"}
    {#if Doctor}
      <Doctor />
    {:else}
      <div class="page-loading">Loading...</div>
    {/if}
  {:else if $currentPage === "compare"}
    {#if Compare}
      <Compare />
    {:else}
      <div class="page-loading">Loading...</div>
    {/if}
  {:else if $currentPage.startsWith("repo:")}
    <RepoDetail repoId={$currentPage.slice(5)} />
  {:else}
    <BootError
      title="Unknown Page"
      message="The application tried to navigate to an unrecognized page."
      detail={$currentPage}
    />
  {/if}
</main>

<style>
  main {
    min-height: 100vh;
    background: var(--bg-primary, #f8fafc);
    color: var(--text-primary, #0f172a);
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  }
  .page-loading {
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: 60vh;
    color: #64748b;
  }
</style>
