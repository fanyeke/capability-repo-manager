<script lang="ts">
  import Dashboard from "$lib/pages/Dashboard.svelte";
  import RepoDetail from "$lib/pages/RepoDetail.svelte";
  import Settings from "$lib/pages/Settings.svelte";
  import { currentPage } from "$lib/stores/uiStore";
</script>

<main>
  {#if $currentPage === "dashboard"}
    <Dashboard />
  {:else if $currentPage === "settings"}
    <Settings />
  {:else if $currentPage.startsWith("repo:")}
    <RepoDetail repoId={$currentPage.slice(5)} />
  {/if}
</main>

<style>
  main {
    min-height: 100vh;
    background: var(--bg-primary);
    color: var(--text-primary);
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  }
</style>
