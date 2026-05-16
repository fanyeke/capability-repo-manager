<script lang="ts">
  import { ArrowUp, ArrowDown, Minus, type Icon as IconType } from 'lucide-svelte';

  let {
    label,
    value,
    trend,
    icon,
    class: className = '',
  }: {
    label: string;
    value: number | string;
    trend?: 'up' | 'down' | 'neutral';
    icon?: typeof IconType;
    class?: string;
  } = $props();

  const trendIcon = trend === 'up' ? ArrowUp : trend === 'down' ? ArrowDown : Minus;
</script>

<div class="metric-card {className}">
  <div class="metric-header">
    {#if icon}
      <svelte:component this={icon} size={16} class="metric-icon" />
    {/if}
    <span class="metric-label">{label}</span>
  </div>
  <div class="metric-value-row">
    <span class="metric-value">{value}</span>
    {#if trend}
      <svelte:component this={trendIcon} size={14} class="metric-trend metric-trend-{trend}" />
    {/if}
  </div>
</div>

<style>
  .metric-card {
    background: var(--bg-card);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    padding: var(--space-4);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    min-width: 140px;
  }
  .metric-header {
    display: flex;
    align-items: center;
    gap: var(--space-1);
  }
  .metric-icon {
    color: var(--color-primary);
  }
  .metric-label {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  .metric-value-row {
    display: flex;
    align-items: baseline;
    gap: var(--space-1);
  }
  .metric-value {
    font-size: var(--font-size-2xl);
    font-weight: 700;
    color: var(--text-primary);
    font-variant-numeric: tabular-nums;
  }
  .metric-trend {
    flex-shrink: 0;
  }
  .metric-trend-up { color: var(--color-success); }
  .metric-trend-down { color: var(--color-danger); }
  .metric-trend-neutral { color: var(--text-muted); }
</style>
