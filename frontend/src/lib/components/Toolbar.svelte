<script lang="ts">
  let { class: className = '', children }: { class?: string; children?: import('svelte').Snippet } = $props();
</script>
<div class="toolbar {className}">{#if children}{@render children()}{/if}</div>
<style>
  .toolbar { display: flex; align-items: center; justify-content: space-between; gap: var(--space-2); }
</style>
