import { describe, it, expect, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { repos, repoFilter, filteredRepos } from '$lib/stores/repoStore';

describe('repoStore', () => {
  beforeEach(() => {
    repos.set([]);
    repoFilter.set({});
  });

  describe('filteredRepos', () => {
    const makeRepo = (id: string, name: string, dirtyState: 'clean' | 'modified' | 'unknown') => ({
      id,
      name,
      path: `/${name}`,
      branch: 'main',
      dirty_state: dirtyState,
      capability_counts: { skill: 0, mcp: 0, hook: 0, rule: 0, agent: 0 },
      capability_index_status: 'never_indexed' as const,
      last_capability_error: null as string | null,
      doctor_score: null as number | null,
      last_indexed_at: '2026-01-01T00:00:00Z',
    });

    it('returns all repos when no filter is active', () => {
      repos.set([makeRepo('1', 'alpha', 'clean'), makeRepo('2', 'beta', 'modified')]);
      repoFilter.set({});
      expect(get(filteredRepos)).toHaveLength(2);
    });

    it('filters by search query', () => {
      repos.set([makeRepo('1', 'my-project', 'clean'), makeRepo('2', 'other-repo', 'clean')]);
      repoFilter.set({ search: 'my' });
      expect(get(filteredRepos)).toHaveLength(1);
      expect(get(filteredRepos)[0].name).toBe('my-project');
    });

    it('filters by dirty_only', () => {
      repos.set([makeRepo('1', 'clean-repo', 'clean'), makeRepo('2', 'modified-repo', 'modified')]);
      repoFilter.set({ dirty_only: true });
      expect(get(filteredRepos)).toHaveLength(1);
      expect(get(filteredRepos)[0].name).toBe('modified-repo');
    });

    it('sorts by name ascending by default', () => {
      repos.set([makeRepo('2', 'zebra', 'clean'), makeRepo('1', 'alpha', 'clean')]);
      repoFilter.set({});
      expect(get(filteredRepos)[0].name).toBe('alpha');
    });

    it('sorts descending when sort_order is desc', () => {
      repos.set([makeRepo('1', 'alpha', 'clean'), makeRepo('2', 'zebra', 'clean')]);
      repoFilter.set({ sort_by: 'name', sort_order: 'desc' });
      expect(get(filteredRepos)[0].name).toBe('zebra');
    });
  });
});
