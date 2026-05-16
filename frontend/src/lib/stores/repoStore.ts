import { writable, derived } from 'svelte/store';
import type { RepositorySummary, RepoDetail, RepoFilter, ScanResult } from '$lib/types';

/// Wrapper around Tauri invoke that times out after 30 seconds.
async function invokeWithTimeout<T>(cmd: string, args: Record<string, unknown>): Promise<T> {
  const { invoke } = await import('@tauri-apps/api/core');
  return Promise.race([
    invoke<T>(cmd, args),
    new Promise<T>((_, reject) =>
      setTimeout(() => reject(new Error(`Command '${cmd}' timed out after 30s`)), 30000),
    ),
  ]);
}

export const repos = writable<RepositorySummary[]>([]);
export const selectedRepoId = writable<string | null>(null);
export const selectedRepoDetail = writable<RepoDetail | null>(null);
export const repoFilter = writable<RepoFilter>({});
export const isScanning = writable(false);
export const scanResult = writable<ScanResult | null>(null);
export const isLoading = writable(false);
export const error = writable<string | null>(null);

export const filteredRepos = derived([repos, repoFilter], ([$repos, $filter]) => {
  let result = [...$repos];

  if ($filter.search) {
    const s = $filter.search.toLowerCase();
    result = result.filter(
      (r) => r.name.toLowerCase().includes(s) || r.path.toLowerCase().includes(s),
    );
  }

  if ($filter.dirty_only) {
    result = result.filter((r) => r.dirty_state !== 'clean');
  }

  // Pinned repos always first
  result.sort((a, b) => {
    if (a.pinned && !b.pinned) return -1;
    if (!a.pinned && b.pinned) return 1;
    // Secondary sort by requested field
    if ($filter.sort_by) {
      const key = $filter.sort_by as keyof RepositorySummary;
      const desc = $filter.sort_order === 'desc';
      const va = a[key] ?? '';
      const vb = b[key] ?? '';
      if (va < vb) return desc ? 1 : -1;
      if (va > vb) return desc ? -1 : 1;
    }
    return a.name.localeCompare(b.name);
  });

  return result;
});

export async function scanRepositories(paths: string[]): Promise<void> {
  isScanning.set(true);
  isLoading.set(true);
  error.set(null);
  try {
    const result = await invokeWithTimeout<ScanResult>('scan_repositories', { paths });
    scanResult.set(result);
    await loadRepos();
  } catch (e: any) {
    error.set(e?.message ?? String(e));
    throw e;
  } finally {
    isScanning.set(false);
    isLoading.set(false);
  }
}

export async function loadRepos(): Promise<void> {
  isLoading.set(true);
  error.set(null);
  try {
    const filter = getFilter();
    const result = await invokeWithTimeout<RepositorySummary[]>('list_repositories', {
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
    const { invoke } = await import('@tauri-apps/api/core');
    const detail = await invoke<RepoDetail>('get_repository_detail', {
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
    const { invoke } = await import('@tauri-apps/api/core');
    const detail = await invoke<RepoDetail>('refresh_repository', { repoId });
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
    const { invoke } = await import('@tauri-apps/api/core');
    await invoke('remove_repository', { repoId });
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
