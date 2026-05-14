import { writable } from "svelte/store";
import type { CompareResult, CapabilityResource, DiffItem } from "$lib/types";

export const compareResult = writable<CompareResult | null>(null);
export const sourceType = writable<"repo" | "pack" | null>(null);
export const sourceId = writable<string | null>(null);
export const targetId = writable<string | null>(null);
export const selectedCategory = writable<"missing" | "extra" | "modified" | "same">("missing");
export const isLoading = writable(false);
export const error = writable<string | null>(null);

export async function compareRepoWithPack(
  repoId: string,
  packId: string,
): Promise<void> {
  isLoading.set(true);
  error.set(null);
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    const result = await invoke<CompareResult>("compare_repo_with_pack", {
      repoId,
      packId,
    });
    compareResult.set(result);
    sourceType.set("pack");
    sourceId.set(packId);
    targetId.set(repoId);
  } catch (e: any) {
    error.set(e?.message ?? String(e));
  } finally {
    isLoading.set(false);
  }
}

export async function compareRepos(
  repoIdA: string,
  repoIdB: string,
): Promise<void> {
  isLoading.set(true);
  error.set(null);
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    const result = await invoke<CompareResult>("compare_repos", {
      repoIdA,
      repoIdB,
    });
    compareResult.set(result);
    sourceType.set("repo");
    sourceId.set(repoIdA);
    targetId.set(repoIdB);
  } catch (e: any) {
    error.set(e?.message ?? String(e));
  } finally {
    isLoading.set(false);
  }
}

export function setCategory(category: "missing" | "extra" | "modified" | "same"): void {
  selectedCategory.set(category);
}

export function clearResult(): void {
  compareResult.set(null);
  sourceType.set(null);
  sourceId.set(null);
  targetId.set(null);
}

export function clearError(): void {
  error.set(null);
}
