<script lang="ts">
  let { left, right, ratio = '1fr 1fr', class: className = '' }: { left: import('svelte').Snippet; right: import('svelte').Snippet; ratio?: string; class?: string } = $props();
</script>
<div class="split-pane" style="grid-template-columns: {ratio} {className}"><div class="split-left">{@render left()}</div><div class="split-right">{@render right()}</div></div>
<style>
  .split-pane { display: grid; gap: var(--space-4); min-height: 0; }
  .split-left, .split-right { min-width: 0; }
</style>
