import { writable } from "svelte/store";

export interface OperationEvent {
  id: string;
  operation_id: string;
  operation_type: string;
  status: string;
  repo_id: string | null;
  pack_id: string | null;
  migration_run_id: string | null;
  summary: string | null;
  detail_json: string | null;
  created_at: string;
}

async function invokeWithTimeout<T>(cmd: string, args: Record<string, unknown>): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  return Promise.race([
    invoke<T>(cmd, args),
    new Promise<T>((_, reject) =>
      setTimeout(() => reject(new Error(`Command '${cmd}' timed out after 30s`)), 30000)
    ),
  ]);
}

export const events = writable<OperationEvent[]>([]);
export const isLoading = writable(false);
export const error = writable<string | null>(null);

export async function loadEvents(typeFilter?: string): Promise<void> {
  isLoading.set(true);
  error.set(null);
  try {
    const result = await invokeWithTimeout<OperationEvent[]>("list_operation_events", {
      limit: 100,
      offset: 0,
      operationType: typeFilter ?? null,
    });
    events.set(result);
  } catch (e: any) {
    error.set(e?.message ?? String(e));
  } finally {
    isLoading.set(false);
  }
}

export function clearError(): void {
  error.set(null);
}