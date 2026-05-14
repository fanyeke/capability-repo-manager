import { writable, derived } from "svelte/store";
import type { CapabilityResource, CapabilityInventory } from "$lib/types";

export const resources = writable<CapabilityInventory | null>(null);
export const selectedResourceId = writable<string | null>(null);
export const selectedType = writable<string | null>(null);
export const isLoading = writable(false);
export const error = writable<string | null>(null);

export const selectedResource = derived(
  [resources, selectedResourceId],
  ([$resources, $id]) => {
    if (!$resources || !$id) return null;
    for (const group of Object.values($resources)) {
      const found = (group as CapabilityResource[]).find((r) => r.id === $id);
      if (found) return found;
    }
    return null;
  },
);

export const typeGroups = derived(resources, ($r) => {
  if (!$r) return [];
  return [
    { type: "skill", label: "Skills", count: $r.skills.length },
    { type: "mcp", label: "MCP Servers", count: $r.mcp.length },
    { type: "hook", label: "Hooks", count: $r.hooks.length },
    { type: "rule", label: "Rules", count: $r.rules.length },
    { type: "agent", label: "Agents", count: $r.agents.length },
    { type: "command", label: "Commands", count: $r.commands.length },
    { type: "plugin", label: "Plugins", count: $r.plugins.length },
    { type: "settings", label: "Settings", count: $r.settings.length },
  ].filter((g) => g.count > 0 || !$r);
});

export const filteredResources = derived(
  [resources, selectedType],
  ([$resources, $type]) => {
    if (!$resources) return [];
    if (!$type) {
      const all: CapabilityResource[] = [];
      for (const group of Object.values($resources)) {
        all.push(...(group as CapabilityResource[]));
      }
      return all;
    }
    const key = $type as keyof CapabilityInventory;
    return ($resources[key] as CapabilityResource[]) ?? [];
  },
);

export async function loadCapabilityInventory(repoId: string): Promise<void> {
  isLoading.set(true);
  error.set(null);
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    const inv = await invoke<CapabilityInventory>("get_capability_inventory", {
      repoId,
    });
    resources.set(inv);
  } catch (e: any) {
    error.set(e?.message ?? String(e));
  } finally {
    isLoading.set(false);
  }
}

export function selectResource(resourceId: string): void {
  selectedResourceId.set(resourceId);
}

export function setTypeFilter(type: string | null): void {
  selectedType.set(type);
}

export function clearError(): void {
  error.set(null);
}
