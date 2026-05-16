<script lang="ts">
  import {
    Code,
    Server,
    Zap,
    FileText,
    Bot,
    Terminal,
    Puzzle,
    Settings,
    type Icon as IconType,
  } from 'lucide-svelte';
  import type { ResourceType } from '$lib/types';

  let {
    type,
    size = 16,
    class: className = '',
  }: {
    type: ResourceType;
    size?: number;
    class?: string;
  } = $props();

  const iconMap: Record<string, typeof IconType> = {
    skill: Code,
    mcp: Server,
    hook: Zap,
    rule: FileText,
    agent: Bot,
    command: Terminal,
    plugin: Puzzle,
    settings: Settings,
    contextDoc: FileText,
  };

  const Icon = iconMap[type] ?? FileText;
</script>

<span class="resource-icon resource-icon-{type} {className}">
  <svelte:component this={Icon} {size} />
</span>

<style>
  .resource-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }
  .resource-icon-skill { color: var(--capability-skill); }
  .resource-icon-mcp { color: var(--capability-mcp); }
  .resource-icon-hook { color: var(--capability-hook); }
  .resource-icon-rule { color: var(--capability-rule); }
  .resource-icon-agent { color: var(--capability-agent); }
  .resource-icon-command { color: var(--capability-command); }
  .resource-icon-plugin { color: var(--capability-plugin); }
  .resource-icon-settings { color: var(--capability-settings); }
  .resource-icon-contextDoc { color: var(--capability-settings); }
</style>
