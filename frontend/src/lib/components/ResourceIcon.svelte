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
  .resource-icon-skill { color: #3b82f6; }
  .resource-icon-mcp { color: #8b5cf6; }
  .resource-icon-hook { color: #ef4444; }
  .resource-icon-rule { color: #f59e0b; }
  .resource-icon-agent { color: #10b981; }
  .resource-icon-command { color: #ec4899; }
  .resource-icon-plugin { color: #6366f1; }
  .resource-icon-settings { color: #6b7280; }
  .resource-icon-contextDoc { color: #6b7280; }
</style>
