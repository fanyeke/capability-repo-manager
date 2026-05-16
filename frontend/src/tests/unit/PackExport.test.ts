import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import { get } from 'svelte/store';
import PackExport from '$lib/pages/PackExport.svelte';
import { selectedRepoDetail } from '$lib/stores/repoStore';
import { resources, typeGroups, selectedType } from '$lib/stores/capabilityStore';
import type { RepoDetail, Repository, CapabilityInventory, CapabilityResource } from '$lib/types';

function makeDetail(name: string): RepoDetail {
  const repo: Repository = {
    id: 'repo-1',
    name,
    path: `/path/${name}`,
    canonical_path: `/path/${name}`,
    remote_url: null,
    current_branch: 'main',
    head_commit: 'abc123',
    dirty_state: 'clean',
    first_indexed_at: '2026-01-01T00:00:00Z',
    last_indexed_at: '2026-01-01T00:00:00Z',
    capability_index_status: 'never_indexed' as const,
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

describe('PackExport', () => {
  beforeEach(() => {
    selectedRepoDetail.set(null);
    resources.set(null);
  });

  it('shows empty state when no repository selected', () => {
    const { container } = render(PackExport);
    expect(container.textContent).toContain('请先选择一个仓库');
  });

  it('shows page header', () => {
    const { container } = render(PackExport);
    expect(container.textContent).toContain('导出能力包');
  });

  it('shows pack metadata form fields when repo detail exists', () => {
    selectedRepoDetail.set(makeDetail('test-repo'));
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
    expect(container.textContent).toContain('包元数据');
    expect(container.textContent).toContain('预览');
  });

  it('shows back to dashboard button', () => {
    const { container } = render(PackExport);
    expect(container.textContent).toContain('返回仪表盘');
  });

  it('shows type dropdown with options when detail exists', () => {
    selectedRepoDetail.set(makeDetail('test-repo'));
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
    expect(container.textContent).toContain('项目');
    expect(container.textContent).toContain('蓝图');
    expect(container.textContent).toContain('基线');
  });

  it('shows resource selection section when detail exists', () => {
    selectedRepoDetail.set(makeDetail('big-repo'));
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
    expect(container.textContent).toContain('big-repo');
    expect(container.textContent).toContain('选择资源');
  });

  it('shows empty state when resources store is null', () => {
    selectedRepoDetail.set(makeDetail('test-repo'));
    resources.set(null);
    const { container } = render(PackExport);
    expect(container.textContent).toContain('导出能力包');
  });
});
