import { writable, derived } from "svelte/store";
import type { RepositorySummary, RepoDetail, RepoFilter, ScanResult } from "$lib/types";

export const repos = writable<RepositorySummary[]>([]);
export const selectedRepoId = writable<string | null>(null);
export const selectedRepoDetail = writable<RepoDetail | null>(null);
export const repoFilter = writable<RepoFilter>({});
export const isScanning = writable(false);
export const scanResult = writable<ScanResult | null>(null);
export const isLoading = writable(false);
export const error = writable<string | null>(null);

export const filteredRepos = derived(
  [repos, repoFilter],
  ([$repos, $filter]) => {
    let result = [...$repos];

    if ($filter.search) {
      const s = $filter.search.toLowerCase();
      result = result.filter(
        (r) =>
          r.name.toLowerCase().includes(s) || r.path.toLowerCase().includes(s),
      );
    }

    if ($filter.dirty_only) {
      result = result.filter((r) => r.dirty_state !== "clean");
    }

    if ($filter.sort_by) {
      const key = $filter.sort_by as keyof RepositorySummary;
      const desc = $filter.sort_order === "desc";
      result.sort((a, b) => {
        const va = a[key] ?? "";
        const vb = b[key] ?? "";
        if (va < vb) return desc ? 1 : -1;
        if (va > vb) return desc ? -1 : 1;
        return 0;
      });
    }

    return result;
  },
);

export async function scanRepositories(paths: string[]): Promise<void> {
  isScanning.set(true);
  error.set(null);
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    const result = await invoke<ScanResult>("scan_repositories", { paths });
    scanResult.set(result);
    await loadRepos();
  } catch (e: any) {
    error.set(e?.message ?? String(e));
  } finally {
    isScanning.set(false);
  }
}

export async function loadRepos(): Promise<void> {
  isLoading.set(true);
  error.set(null);
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    const filter = getFilter();
    const result = await invoke<RepositorySummary[]>("list_repositories", {
      filter,
    });
    repos.set(result);
  } catch (e: any) {
    error.set(e?.message ?? String(e));
  } finally {
    isLoading.set(false);
  }
}

export async function loadRepoDetail(repoId: string): Promise<void> {
  isLoading.set(true);
  error.set(null);
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    const detail = await invoke<RepoDetail>("get_repository_detail", {
      repoId,
    });
    selectedRepoDetail.set(detail);
  } catch (e: any) {
    error.set(e?.message ?? String(e));
  } finally {
    isLoading.set(false);
  }
}

export async function refreshRepository(repoId: string): Promise<void> {
  isLoading.set(true);
  error.set(null);
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    const detail = await invoke<RepoDetail>("refresh_repository", { repoId });
    selectedRepoDetail.set(detail);
    await loadRepos();
  } catch (e: any) {
    error.set(e?.message ?? String(e));
  } finally {
    isLoading.set(false);
  }
}

export async function removeRepository(repoId: string): Promise<void> {
  error.set(null);
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    await invoke("remove_repository", { repoId });
    selectedRepoId.set(null);
    selectedRepoDetail.set(null);
    await loadRepos();
  } catch (e: any) {
    error.set(e?.message ?? String(e));
  }
}

export function selectRepo(repoId: string): void {
  selectedRepoId.set(repoId);
  loadRepoDetail(repoId);
}

export function setFilter(filter: RepoFilter): void {
  repoFilter.set(filter);
}

export function clearError(): void {
  error.set(null);
}

function getFilter() {
  let value: RepoFilter = {};
  repoFilter.subscribe((v) => (value = v))();
  return value;
}
