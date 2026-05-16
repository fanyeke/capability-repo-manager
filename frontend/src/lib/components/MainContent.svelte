<script lang="ts">
  let { children }: { children: import('svelte').Snippet } = $props();
</script>

<main class="main-content">
  <div class="content-inner">
    {@render children()}
  </div>
</main>

<style>
  .main-content {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    background: var(--bg-app);
  }
  .content-inner {
    max-width: var(--content-max-width);
    padding: 0 var(--content-padding) var(--content-padding);
    margin: 0 auto;
  }
  /* Scrollbar styling */
  .main-content::-webkit-scrollbar {
    width: var(--scrollbar-width);
  }
  .main-content::-webkit-scrollbar-track {
    background: var(--scrollbar-track);
  }
  .main-content::-webkit-scrollbar-thumb {
    background: var(--scrollbar-thumb);
    border-radius: 3px;
  }
</style>
