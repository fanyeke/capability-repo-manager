<script lang="ts">
  let { class: className = '', children }: { class?: string; children?: import('svelte').Snippet } = $props();
</script>
<div class="page {className}">{#if children}{@render children()}{/if}</div>
<style>
  .page { display: flex; flex-direction: column; gap: var(--space-4); padding-top: var(--space-4); }
</style>
