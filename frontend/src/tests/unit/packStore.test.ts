import { describe, it, expect, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { packs, packFilter, filteredPacks } from '$lib/stores/packStore';

describe('packStore', () => {
  beforeEach(() => {
    packs.set([]);
    packFilter.set({});
  });

  describe('filteredPacks', () => {
    const makePack = (
      id: string,
      name: string,
      packType: 'project' | 'blueprint' | 'baseline',
      desc?: string,
    ) => ({
      id,
      name,
      version: '1.0.0',
      description: desc || null,
      pack_type: packType,
      resource_count: 5,
      source_repo_name: 'test-repo',
      created_at: '2026-01-01T00:00:00Z',
    });

    it('returns all packs when no filter is active', () => {
      packs.set([makePack('1', 'pack-a', 'project'), makePack('2', 'pack-b', 'blueprint')]);
      expect(get(filteredPacks)).toHaveLength(2);
    });

    it('filters by search query', () => {
      packs.set([
        makePack('1', 'frontend-tools', 'project'),
        makePack('2', 'backend-config', 'blueprint'),
      ]);
      packFilter.set({ search: 'frontend' });
      expect(get(filteredPacks)).toHaveLength(1);
      expect(get(filteredPacks)[0].name).toBe('frontend-tools');
    });

    it('filters by pack_type', () => {
      packs.set([makePack('1', 'pack-a', 'project'), makePack('2', 'pack-b', 'blueprint')]);
      packFilter.set({ pack_type: 'project' });
      expect(get(filteredPacks)).toHaveLength(1);
      expect(get(filteredPacks)[0].name).toBe('pack-a');
    });

    it('searches in description', () => {
      packs.set([
        makePack('1', 'pack-a', 'project', 'This is for testing'),
        makePack('2', 'pack-b', 'blueprint', 'Production config'),
      ]);
      packFilter.set({ search: 'testing' });
      expect(get(filteredPacks)).toHaveLength(1);
    });
  });
});
