<script lang="ts">
  import type { RepositorySummary, RepoFilter } from '$lib/types';
  import RepoCard from './RepoCard.svelte';
  import { _ } from 'svelte-i18n';

  let {
    repos,
    filter,
    isLoading = false,
    onSelectRepo,
    onFilterChange,
    onScan,
  }: {
    repos: RepositorySummary[];
    filter: RepoFilter;
    isLoading?: boolean;
    onSelectRepo: (id: string) => void;
    onFilterChange: (filter: RepoFilter) => void;
    onScan: () => void;
  } = $props();

  let searchText = $state('');
  let sortBy = $state<'name' | 'path' | 'last_indexed_at' | 'dirty_state'>('name');
  let sortOrder = $state<'asc' | 'desc'>('asc');
  let dirtyOnly = $state(false);

  function updateFilter() {
    onFilterChange({
      search: searchText || undefined,
      dirty_only: dirtyOnly || undefined,
      sort_by: sortBy,
      sort_order: sortOrder,
    });
  }

  function handleSearch() {
    updateFilter();
  }

  function toggleDirtyFilter() {
    dirtyOnly = !dirtyOnly;
    updateFilter();
  }

  function handleSortChange(field: 'name' | 'path' | 'last_indexed_at' | 'dirty_state') {
    if (sortBy === field) {
      sortOrder = sortOrder === 'asc' ? 'desc' : 'asc';
    } else {
      sortBy = field;
      sortOrder = 'asc';
    }
    updateFilter();
  }
</script>

<div class="repo-list-container">
  <div class="toolbar">
    <div class="search-bar">
      <input
        type="search"
        bind:value={searchText}
        placeholder={$_('common.search')}
        oninput={handleSearch}
      />
    </div>

    <div class="filter-controls">
      <button class="filter-btn" class:active={dirtyOnly} onclick={toggleDirtyFilter}>
        {$_('repo.dirty_state')}
      </button>

      <button class="scan-btn primary" onclick={onScan} disabled={isLoading}>
        {isLoading ? $_('dashboard.scanning') : $_('dashboard.scan')}
      </button>
    </div>

    <div class="sort-controls">
      <span class="sort-label">{$_('common.sort')}:</span>
      {#each [{ key: 'name', label: $_('common.sort') }, { key: 'last_indexed_at', label: $_('repo.last_indexed') }, { key: 'dirty_state', label: $_('repo.dirty_state') }] as option}
        <button
          class="sort-btn"
          class:active={sortBy === option.key}
          onclick={() =>
            handleSortChange(option.key as 'name' | 'path' | 'last_indexed_at' | 'dirty_state')}
        >
          {option.label}
          {#if sortBy === option.key}
            <span class="sort-arrow">{sortOrder === 'asc' ? ' ↑' : ' ↓'}</span>
          {/if}
        </button>
      {/each}
    </div>
  </div>

  {#if isLoading}
    <p class="loading-text">{$_('dashboard.loading')}</p>
  {:else if repos.length === 0}
    <p class="empty-text">{$_('dashboard.no_repos')}</p>
  {:else}
    <div class="repo-grid">
      {#each repos as repo (repo.id)}
        <RepoCard {repo} onSelect={onSelectRepo} />
      {/each}
    </div>
  {/if}
</div>

<style>
  .repo-list-container {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .toolbar {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .search-bar input {
    width: 100%;
    padding: 10px 14px;
    border: 1px solid var(--border-color, #e2e8f0);
    border-radius: 8px;
    font-size: 0.95rem;
    box-sizing: border-box;
  }
  .filter-controls {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  .filter-btn,
  .sort-btn {
    padding: 6px 14px;
    border: 1px solid var(--border-color, #e2e8f0);
    border-radius: 6px;
    background: #fff;
    font-size: 0.85rem;
    cursor: pointer;
  }
  .filter-btn.active,
  .sort-btn.active {
    background: var(--primary, #3b82f6);
    color: #fff;
    border-color: var(--primary, #3b82f6);
  }
  .scan-btn.primary {
    padding: 8px 20px;
    background: var(--primary, #3b82f6);
    color: #fff;
    border: none;
    border-radius: 8px;
    font-weight: 600;
    cursor: pointer;
  }
  .scan-btn.primary:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
  .sort-controls {
    display: flex;
    gap: 6px;
    align-items: center;
    font-size: 0.85rem;
  }
  .sort-label {
    color: #64748b;
  }
  .sort-arrow {
    font-size: 0.75rem;
  }
  .repo-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: 16px;
  }
  .loading-text,
  .empty-text {
    text-align: center;
    color: #64748b;
    padding: 40px 0;
  }
</style>
