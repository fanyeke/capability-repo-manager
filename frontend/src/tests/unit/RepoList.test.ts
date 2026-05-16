import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import RepoList from '$lib/components/RepoList.svelte';
import type { RepositorySummary, RepoFilter } from '$lib/types';
import { get } from 'svelte/store';

function makeRepo(
  id: string,
  name: string,
  dirty: 'clean' | 'modified' | 'unknown',
): RepositorySummary {
  return {
    id,
    name,
    path: `/${name}`,
    branch: 'main',
    dirty_state: dirty,
    capability_counts: { skill: 1, mcp: 0, hook: 0, rule: 0, agent: 0 },
    capability_index_status: 'never_indexed' as const,
    last_capability_error: null,
    doctor_score: 100,
    last_indexed_at: '2026-01-01T00:00:00Z',
  };
}

describe('RepoList', () => {
  const defaultProps = {
    repos: [] as RepositorySummary[],
    filter: {} as RepoFilter,
    isLoading: false,
    onSelectRepo: vi.fn(),
    onFilterChange: vi.fn(),
    onScan: vi.fn(),
  };

  it('renders loading state', () => {
    const { container } = render(RepoList, { props: { ...defaultProps, isLoading: true } });
    expect(container.textContent).toContain('正在加载仓库列表');
  });

  it('renders empty state when no repos', () => {
    const { container } = render(RepoList, { props: { ...defaultProps, repos: [] } });
    expect(container.textContent).toContain('未发现仓库');
  });

  it('renders repo grid with repos', () => {
    const repos = [makeRepo('1', 'test-repo', 'clean')];
    const { container } = render(RepoList, { props: { ...defaultProps, repos } });
    expect(container.textContent).toContain('test-repo');
  });

  it('renders all repo names from list', () => {
    const repos = [
      makeRepo('1', 'project-alpha', 'clean'),
      makeRepo('2', 'project-beta', 'modified'),
    ];
    const { container } = render(RepoList, { props: { ...defaultProps, repos } });
    expect(container.textContent).toContain('project-alpha');
    expect(container.textContent).toContain('project-beta');
  });

  it('shows scan button with default text when not loading', () => {
    const { container } = render(RepoList, { props: defaultProps });
    expect(container.textContent).toContain('扫描仓库');
  });
});
