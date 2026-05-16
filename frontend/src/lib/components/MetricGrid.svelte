<script lang="ts">
  let { columns = 4, class: className = '', children }: { columns?: number; class?: string; children?: import('svelte').Snippet } = $props();
</script>
<div class="metric-grid cols-{columns} {className}">{#if children}{@render children()}{/if}</div>
<style>
  .metric-grid { display: grid; gap: var(--space-3); }
  .cols-2 { grid-template-columns: repeat(2, 1fr); }
  .cols-3 { grid-template-columns: repeat(3, 1fr); }
  .cols-4 { grid-template-columns: repeat(auto-fit, minmax(160px, 1fr)); }
</style>
