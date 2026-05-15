import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/svelte";
import { get } from "svelte/store";
import PackExport from "$lib/pages/PackExport.svelte";
import { selectedRepoDetail } from "$lib/stores/repoStore";
import { resources, typeGroups, selectedType } from "$lib/stores/capabilityStore";
import type { RepoDetail, Repository, CapabilityInventory, CapabilityResource } from "$lib/types";

function makeDetail(name: string): RepoDetail {
  const repo: Repository = {
    id: "repo-1",
    name,
    path: `/path/${name}`,
    canonical_path: `/path/${name}`,
    remote_url: null,
    current_branch: "main",
    head_commit: "abc123",
    dirty_state: "clean",
    first_indexed_at: "2026-01-01T00:00:00Z",
    last_indexed_at: "2026-01-01T00:00:00Z",
    capability_index_status: "never_indexed" as const,
    last_capability_indexed_at: null,
    last_capability_error: null,
  };
  return {
    repo,
    capabilities: {
      skills: [],
      mcp: [],
      hooks: [],
      rules: [],
      agents: [],
      commands: [],
      plugins: [],
      settings: [],
    } as CapabilityInventory,
    doctor_latest: null,
  };
}

describe("PackExport", () => {
  beforeEach(() => {
    selectedRepoDetail.set(null);
    resources.set(null);
  });

  it("shows empty state when no repository selected", () => {
    const { container } = render(PackExport);
    expect(container.textContent).toContain("Select a repository first");
  });

  it("shows page header", () => {
    const { container } = render(PackExport);
    expect(container.textContent).toContain("Export Capability Pack");
  });

  it("shows pack metadata form fields when repo detail exists", () => {
    selectedRepoDetail.set(makeDetail("test-repo"));
    resources.set({
      skills: [],
      mcp: [],
      hooks: [],
      rules: [],
      agents: [],
      commands: [],
      plugins: [],
      settings: [],
    });
    const { container } = render(PackExport);
    expect(container.textContent).toContain("Pack Metadata");
    expect(container.textContent).toContain("Preview");
  });

  it("shows back to dashboard button", () => {
    const { container } = render(PackExport);
    expect(container.textContent).toContain("Back to Dashboard");
  });

  it("shows type dropdown with options when detail exists", () => {
    selectedRepoDetail.set(makeDetail("test-repo"));
    resources.set({
      skills: [],
      mcp: [],
      hooks: [],
      rules: [],
      agents: [],
      commands: [],
      plugins: [],
      settings: [],
    });
    const { container } = render(PackExport);
    expect(container.textContent).toContain("Project");
    expect(container.textContent).toContain("Blueprint");
    expect(container.textContent).toContain("Baseline");
  });

  it("shows resource selection section when detail exists", () => {
    selectedRepoDetail.set(makeDetail("big-repo"));
    resources.set({
      skills: [],
      mcp: [],
      hooks: [],
      rules: [],
      agents: [],
      commands: [],
      plugins: [],
      settings: [],
    });
    const { container } = render(PackExport);
    expect(container.textContent).toContain("big-repo");
    expect(container.textContent).toContain("Select Resources");
  });

  it("shows empty state when resources store is null", () => {
    selectedRepoDetail.set(makeDetail("test-repo"));
    resources.set(null);
    const { container } = render(PackExport);
    expect(container.textContent).toContain("Export Capability Pack");
  });
});
