<script lang="ts">
  let { class: className = '', children }: { class?: string; children?: import('svelte').Snippet } = $props();
</script>
<div class="filter-bar {className}">{#if children}{@render children()}{/if}</div>
<style>
  .filter-bar { display: flex; align-items: center; gap: var(--space-2); flex-wrap: wrap; }
</style>
