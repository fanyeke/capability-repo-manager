import { writable, derived } from "svelte/store";
import type {
  PackSummary,
  PackDetail,
  PackSelection,
  PackMetadata,
  PackFilter,
  ValidationResult,
  ManifestData,
} from "$lib/types";

export const packs = writable<PackSummary[]>([]);
export const selectedPackId = writable<string | null>(null);
export const selectedPackDetail = writable<PackDetail | null>(null);
export const packFilter = writable<PackFilter>({});
export const isLoading = writable(false);
export const error = writable<string | null>(null);
export const validationResult = writable<ValidationResult | null>(null);

export const filteredPacks = derived(
  [packs, packFilter],
  ([$packs, $filter]) => {
    let result = [...$packs];

    if ($filter.search) {
      const s = $filter.search.toLowerCase();
      result = result.filter(
        (p) => p.name.toLowerCase().includes(s) || (p.description ?? "").toLowerCase().includes(s),
      );
    }

    if ($filter.pack_type) {
      result = result.filter((p) => p.pack_type === $filter.pack_type);
    }

    return result;
  },
);

export async function exportPack(
  repoId: string,
  selection: PackSelection,
  metadata: PackMetadata,
): Promise<PackSummary | null> {
  isLoading.set(true);
  error.set(null);
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    const result = await invoke<PackSummary>("export_capability_pack", {
      repoId,
      selection,
      metadata,
    });
    await loadPacks();
    return result;
  } catch (e: any) {
    error.set(e?.message ?? String(e));
    return null;
  } finally {
    isLoading.set(false);
  }
}

export async function loadPacks(): Promise<void> {
  isLoading.set(true);
  error.set(null);
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    const f = getFilter();
    const result = await invoke<PackSummary[]>("list_packs", { filter: f });
    packs.set(result);
  } catch (e: any) {
    error.set(e?.message ?? String(e));
  } finally {
    isLoading.set(false);
  }
}

export async function loadPackDetail(packId: string): Promise<void> {
  isLoading.set(true);
  error.set(null);
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    const detail = await invoke<PackDetail>("get_pack_detail", { packId });
    selectedPackDetail.set(detail);
  } catch (e: any) {
    error.set(e?.message ?? String(e));
  } finally {
    isLoading.set(false);
  }
}

export async function deletePack(packId: string): Promise<void> {
  error.set(null);
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    await invoke("delete_pack", { packId });
    selectedPackId.set(null);
    selectedPackDetail.set(null);
    await loadPacks();
  } catch (e: any) {
    error.set(e?.message ?? String(e));
  }
}

export async function validatePack(packId: string): Promise<void> {
  error.set(null);
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    const result = await invoke<ValidationResult>("validate_pack", { packId });
    validationResult.set(result);
  } catch (e: any) {
    error.set(e?.message ?? String(e));
  }
}

export function selectPack(packId: string): void {
  selectedPackId.set(packId);
  loadPackDetail(packId);
}

export function setPackFilter(filter: PackFilter): void {
  packFilter.set(filter);
}

export function clearError(): void {
  error.set(null);
}

function getFilter(): PackFilter {
  let value: PackFilter = {};
  packFilter.subscribe((v) => (value = v))();
  return value;
}
