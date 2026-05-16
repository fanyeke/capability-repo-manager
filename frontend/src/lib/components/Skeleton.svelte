<script lang="ts">
  let {
    variant = 'text',
    width,
    height,
    count = 1,
    class: className = '',
  }: {
    variant?: 'text' | 'card' | 'circle' | 'rect';
    width?: string;
    height?: string;
    count?: number;
    class?: string;
  } = $props();
</script>

{#each Array(count) as _, i (i)}
  <div
    class="skeleton skeleton-{variant} {className}"
    style={[
      width ? `width:${width}` : '',
      height ? `height:${height}` : '',
    ].filter(Boolean).join(';')}
  ></div>
{/each}

<style>
  .skeleton {
    background: linear-gradient(
      90deg,
      var(--bg-elevated) 25%,
      var(--bg-hover) 50%,
      var(--bg-elevated) 75%
    );
    background-size: 200% 100%;
    animation: shimmer 1.5s ease-in-out infinite;
    border-radius: var(--radius-sm);
  }
  @keyframes shimmer {
    0% { background-position: 200% 0; }
    100% { background-position: -200% 0; }
  }

  .skeleton-text {
    height: 12px;
    width: 100%;
    margin-bottom: 8px;
  }
  .skeleton-card {
    height: 80px;
    width: 100%;
    border-radius: var(--radius-md);
  }
  .skeleton-circle {
    width: 36px;
    height: 36px;
    border-radius: var(--radius-full);
  }
  .skeleton-rect {
    height: 100px;
    width: 100%;
    border-radius: var(--radius-md);
  }
</style>
