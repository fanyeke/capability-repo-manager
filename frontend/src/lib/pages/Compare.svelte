<script lang="ts">
  import { _ } from 'svelte-i18n';
  import { currentPage, navigateTo } from '$lib/stores/uiStore';
  import { repos, loadRepos } from '$lib/stores/repoStore';
  import { packs, loadPacks } from '$lib/stores/packStore';
  import {
    compareResult,
    compareRepos,
    compareRepoWithPack,
    selectedCategory,
    setCategory,
  } from '$lib/stores/compareStore';
  import CompareViewComp from '$lib/components/CompareView.svelte';

  let mode = $state<'repo-repo' | 'repo-pack'>('repo-repo');
  let sourceRepo = $state<string | null>(null);
  let targetRepo = $state<string | null>(null);
  let sourcePack = $state<string | null>(null);
  let targetRepoForPack = $state<string | null>(null);

  loadRepos();
  loadPacks();

  async function handleCompare() {
    if (mode === 'repo-repo' && sourceRepo && targetRepo) {
      await compareRepos(sourceRepo, targetRepo);
    } else if (mode === 'repo-pack' && sourcePack && targetRepoForPack) {
      await compareRepoWithPack(targetRepoForPack, sourcePack);
    }
  }
</script>

<div class="compare-page">
  <header class="page-header">
    <h1>{$_('compare.title')}</h1>
    <button class="back-btn" onclick={() => navigateTo('dashboard')}>{$_('nav.back')}</button>
  </header>

  <main class="compare-content">
    <div class="mode-tabs">
      <button
        class="mode-btn"
        class:active={mode === 'repo-repo'}
        onclick={() => (mode = 'repo-repo')}
      >
        {$_('compare.mode_repo_repo')}
      </button>
      <button
        class="mode-btn"
        class:active={mode === 'repo-pack'}
        onclick={() => (mode = 'repo-pack')}
      >
        {$_('compare.mode_repo_pack')}
      </button>
    </div>

    <div class="select-section">
      {#if mode === 'repo-repo'}
        <div class="select-row">
          <div class="select-panel">
            <label>{$_('compare.source_repo')}</label>
            <select bind:value={sourceRepo}>
              <option value="">{$_('compare.choose_source')}</option>
              {#each $repos as repo (repo.id)}
                <option value={repo.id}>{repo.name}</option>
              {/each}
            </select>
          </div>
          <div class="select-panel">
            <label>{$_('compare.target_repo')}</label>
            <select bind:value={targetRepo}>
              <option value="">{$_('compare.choose_target')}</option>
              {#each $repos as repo (repo.id)}
                <option value={repo.id}>{repo.name}</option>
              {/each}
            </select>
          </div>
        </div>
      {:else}
        <div class="select-row">
          <div class="select-panel">
            <label>{$_('compare.pack_source')}</label>
            <select bind:value={sourcePack}>
              <option value="">{$_('compare.choose_pack')}</option>
              {#each $packs as pack (pack.id)}
                <option value={pack.id}>{pack.name} v{pack.version}</option>
              {/each}
            </select>
          </div>
          <div class="select-panel">
            <label>{$_('compare.repo_target')}</label>
            <select bind:value={targetRepoForPack}>
              <option value="">{$_('compare.choose_repo')}</option>
              {#each $repos as repo (repo.id)}
                <option value={repo.id}>{repo.name}</option>
              {/each}
            </select>
          </div>
        </div>
      {/if}

      <button
        class="compare-btn"
        onclick={handleCompare}
        disabled={(mode === 'repo-repo' && (!sourceRepo || !targetRepo)) ||
          (mode === 'repo-pack' && (!sourcePack || !targetRepoForPack))}
      >
        {$_('compare.action')}
      </button>
    </div>

    {#if $compareResult}
      <div class="result-section">
        <CompareViewComp
          result={$compareResult}
          selectedCategory={$selectedCategory}
          onSelectCategory={setCategory}
        />
      </div>
    {:else}
      <div class="compare-info">
        <p>{$_('compare.empty_info')}</p>
        <p><strong>{$_('compare.missing')}</strong> — {$_('compare.missing_desc')}</p>
        <p><strong>{$_('compare.extra')}</strong> — {$_('compare.extra_desc')}</p>
        <p><strong>{$_('compare.modified')}</strong> — {$_('compare.modified_desc')}</p>
        <p><strong>{$_('compare.same')}</strong> — {$_('compare.same_desc')}</p>
      </div>
    {/if}
  </main>
</div>

<style>
  .compare-page {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
  }
  .page-header {
    display: flex;
    justify-content: space-between;
    padding: var(--space-4) var(--space-6);
    border-bottom: 1px solid var(--border-default);
    background: var(--bg-card);
  }
  .page-header h1 {
    margin: 0;
  }
  .back-btn {
    padding: var(--space-2) var(--space-4);
    background: var(--bg-card);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    cursor: pointer;
  }
  .compare-content {
    flex: 1;
    padding: var(--space-6);
    background: var(--bg-elevated);
  }
  .mode-tabs {
    display: flex;
    gap: var(--space-2);
    margin-bottom: var(--space-6);
  }
  .mode-btn {
    padding: 10px 20px;
    background: var(--bg-card);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    cursor: pointer;
  }
  .mode-btn.active {
    background: var(--color-primary);
    color: var(--text-primary);
    border-color: var(--color-primary);
  }
  .select-section {
    background: var(--bg-card);
    padding: var(--space-5);
    border-radius: var(--radius-md);
    border: 1px solid var(--border-default);
    margin-bottom: var(--space-6);
  }
  .select-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-4);
    margin-bottom: var(--space-4);
  }
  .select-panel label {
    display: block;
    font-weight: 600;
    margin-bottom: 6px;
    font-size: var(--font-size-sm);
  }
  .select-panel select {
    width: 100%;
    padding: 10px;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    font-size: var(--font-size-md);
    box-sizing: border-box;
  }
  .compare-btn {
    padding: var(--space-3) var(--space-8);
    background: var(--color-primary);
    color: var(--text-primary);
    border: none;
    border-radius: var(--radius-md);
    font-weight: 600;
    cursor: pointer;
  }
  .compare-btn:disabled {
    opacity: 0.6;
  }
  .result-section {
    background: var(--bg-card);
    padding: var(--space-5);
    border-radius: var(--radius-md);
    border: 1px solid var(--border-default);
  }
  .compare-info {
    background: var(--bg-card);
    padding: var(--space-5);
    border-radius: var(--radius-md);
    border: 1px solid var(--border-default);
    color: var(--text-muted);
  }
  .compare-info p {
    font-size: var(--font-size-sm);
    margin: var(--space-2) 0;
  }
</style>
