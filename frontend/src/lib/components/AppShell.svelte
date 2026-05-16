<script lang="ts">
  import Sidebar from './Sidebar.svelte';
  import TopHeader from './TopHeader.svelte';
  import MainContent from './MainContent.svelte';

  let { children }: { children: import('svelte').Snippet } = $props();
</script>

<div class="app-shell">
  <Sidebar />
  <div class="app-main">
    <TopHeader />
    <MainContent>
      {@render children()}
    </MainContent>
  </div>
</div>

<style>
  .app-shell {
    display: flex;
    height: 100vh;
    overflow: hidden;
    background: var(--bg-app);
    color: var(--text-primary);
    font-family: var(--font-sans);
    font-size: var(--font-size-md);
  }
  .app-main {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
  }
</style>
