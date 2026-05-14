<script lang="ts">
  import Dashboard from "$lib/pages/Dashboard.svelte";
  import RepoDetail from "$lib/pages/RepoDetail.svelte";
  import Settings from "$lib/pages/Settings.svelte";
  import GuidedSetup from "$lib/pages/GuidedSetup.svelte";
  import PackExport from "$lib/pages/PackExport.svelte";
  import PackApply from "$lib/pages/PackApply.svelte";
  import Doctor from "$lib/pages/Doctor.svelte";
  import Compare from "$lib/pages/Compare.svelte";
  import { currentPage } from "$lib/stores/uiStore";
  import { onMount } from "svelte";

  onMount(async () => {
    // Check if there are any repos already indexed
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const repos = await invoke<any[]>("list_repositories", { filter: {} });
      if (repos.length === 0) {
        currentPage.set("guidedsetup");
      }
    } catch {
      // First launch, show guided setup
      currentPage.set("guidedsetup");
    }
  });
</script>

<main>
  {#if $currentPage === "guidedsetup"}
    <GuidedSetup />
  {:else if $currentPage === "dashboard"}
    <Dashboard />
  {:else if $currentPage === "settings"}
    <Settings />
  {:else if $currentPage === "packexport"}
    <PackExport />
  {:else if $currentPage === "packapply"}
    <PackApply />
  {:else if $currentPage === "doctor"}
    <Doctor />
  {:else if $currentPage === "compare"}
    <Compare />
  {:else if $currentPage.startsWith("repo:")}
    <RepoDetail repoId={$currentPage.slice(5)} />
  {/if}
</main>

<style>
  main {
    min-height: 100vh;
    background: var(--bg-primary, #f8fafc);
    color: var(--text-primary, #0f172a);
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  }
</style>