import { writable } from 'svelte/store';
import type {
  MigrationPlan,
  MigrationPlanItem,
  MigrationReport,
  MigrationRunSummary,
  ConflictStrategy,
} from '$lib/types';

export const plan = writable<MigrationPlan | null>(null);
export const report = writable<MigrationReport | null>(null);
export const history = writable<MigrationRunSummary[]>([]);
export const strategies = writable<ConflictStrategy[]>([]);
export const isLoading = writable(false);
export const error = writable<string | null>(null);

type StrategyAction = 'skip' | 'overwrite';

export function setStrategy(resourceId: string, action: string): void {
  strategies.update((s) => {
    const existing = s.findIndex((st) => st.resource_id === resourceId);
    if (existing >= 0) {
      s[existing] = { resource_id: resourceId, action: action as ConflictStrategy['action'] };
      return [...s];
    }
    return [...s, { resource_id: resourceId, action: action as ConflictStrategy['action'] }];
  });
}

export function getDefaultStrategy(item: MigrationPlanItem): StrategyAction {
  if (item.action === 'add' || item.action === 'unresolved') return 'skip';
  return item.action as StrategyAction;
}

export async function buildMigrationPlan(packId: string, targetRepoId: string): Promise<void> {
  isLoading.set(true);
  error.set(null);
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    const result = await invoke<MigrationPlan>('build_migration_plan', {
      packId,
      targetRepoId,
    });
    plan.set(result);

    // Set default strategies for conflicts
    const defaultStrategies: ConflictStrategy[] = result.items
      .filter((item) => item.action === 'unresolved' || item.action === 'overwrite')
      .map((item) => ({
        resource_id: item.resource_id,
        action: (item.action === 'unresolved' ? 'skip' : item.action) as StrategyAction,
      }));
    strategies.set(defaultStrategies);
  } catch (e: any) {
    error.set(e?.message ?? String(e));
  } finally {
    isLoading.set(false);
  }
}

export async function applyMigrationPlan(): Promise<void> {
  isLoading.set(true);
  error.set(null);
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    const p = getPlan();
    if (!p) throw new Error('No plan to apply');

    const s = getStrategies();
    const result = await invoke<MigrationReport>('apply_migration_plan', {
      planId: p.plan_id,
      strategies: s,
    });
    report.set(result);
  } catch (e: any) {
    error.set(e?.message ?? String(e));
  } finally {
    isLoading.set(false);
  }
}

export async function rollbackMigration(runId: string): Promise<void> {
  isLoading.set(true);
  error.set(null);
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    await invoke('rollback_migration', { runId });
  } catch (e: any) {
    error.set(e?.message ?? String(e));
  } finally {
    isLoading.set(false);
  }
}

export async function loadMigrationHistory(repoId: string): Promise<void> {
  isLoading.set(true);
  error.set(null);
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    const result = await invoke<MigrationRunSummary[]>('get_migration_history', {
      repoId,
    });
    history.set(result);
  } catch (e: any) {
    error.set(e?.message ?? String(e));
  } finally {
    isLoading.set(false);
  }
}

export function clearPlan(): void {
  plan.set(null);
  report.set(null);
  strategies.set([]);
}

export function clearError(): void {
  error.set(null);
}

function getPlan(): MigrationPlan | null {
  let value: MigrationPlan | null = null;
  plan.subscribe((v) => (value = v))();
  return value;
}

function getStrategies(): ConflictStrategy[] {
  let value: ConflictStrategy[] = [];
  strategies.subscribe((v) => (value = v))();
  return value;
}
