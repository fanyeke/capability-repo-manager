<script lang="ts">
  let { title, class: className = '', children }: { title?: string; class?: string; children?: import('svelte').Snippet } = $props();
</script>
<section class="section {className}">
  {#if title}<h2 class="section-title">{title}</h2>{/if}
  {#if children}{@render children()}{/if}
</section>
<style>
  .section { display: flex; flex-direction: column; gap: var(--space-3); }
  .section-title { font-size: var(--font-size-sm); font-weight: 600; color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.05em; margin: 0; }
</style>
