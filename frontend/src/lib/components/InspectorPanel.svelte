<script lang="ts">
  let { class: className = '', children }: { class?: string; children?: import('svelte').Snippet } = $props();
</script>
<div class="inspector {className}">{#if children}{@render children()}{/if}</div>
<style>
  .inspector { background: var(--bg-card); border: 1px solid var(--border-default); border-radius: var(--radius-md); padding: var(--space-4); min-height: 200px; }
</style>
