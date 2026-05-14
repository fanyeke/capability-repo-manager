import { writable } from "svelte/store";
import type { DoctorReport, DoctorIssue } from "$lib/types";

export const report = writable<DoctorReport | null>(null);
export const isLoading = writable(false);
export const error = writable<string | null>(null);

export async function runDoctor(repoId: string): Promise<void> {
  isLoading.set(true);
  error.set(null);
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    const result = await invoke<DoctorReport>("run_doctor", { repoId });
    report.set(result);
  } catch (e: any) {
    error.set(e?.message ?? String(e));
  } finally {
    isLoading.set(false);
  }
}

export function clearReport(): void {
  report.set(null);
}

export function clearError(): void {
  error.set(null);
}

export function groupIssuesBySeverity(
  issues: DoctorIssue[],
): Record<string, DoctorIssue[]> {
  const groups: Record<string, DoctorIssue[]> = {
    critical: [],
    warning: [],
    info: [],
  };
  for (const issue of issues) {
    groups[issue.severity]?.push(issue);
  }
  return groups;
}

export function getScoreColor(score: number): string {
  if (score >= 80) return "#22c55e"; // green
  if (score >= 50) return "#eab308"; // yellow
  return "#ef4444"; // red
}

export function getScoreLabel(score: number): string {
  if (score >= 80) return "Healthy";
  if (score >= 50) return "Needs Attention";
  return "Critical";
}
