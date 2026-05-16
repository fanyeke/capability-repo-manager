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
    padding: 16px 24px;
    border-bottom: 1px solid #e2e8f0;
    background: #fff;
  }
  .page-header h1 {
    margin: 0;
  }
  .back-btn {
    padding: 8px 16px;
    background: #fff;
    border: 1px solid #e2e8f0;
    border-radius: 6px;
    cursor: pointer;
  }
  .compare-content {
    flex: 1;
    padding: 24px;
    background: #f8fafc;
  }
  .mode-tabs {
    display: flex;
    gap: 8px;
    margin-bottom: 24px;
  }
  .mode-btn {
    padding: 10px 20px;
    background: #fff;
    border: 1px solid #e2e8f0;
    border-radius: 6px;
    cursor: pointer;
  }
  .mode-btn.active {
    background: var(--primary, #3b82f6);
    color: #fff;
    border-color: var(--primary, #3b82f6);
  }
  .select-section {
    background: #fff;
    padding: 20px;
    border-radius: 8px;
    border: 1px solid #e2e8f0;
    margin-bottom: 24px;
  }
  .select-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
    margin-bottom: 16px;
  }
  .select-panel label {
    display: block;
    font-weight: 600;
    margin-bottom: 6px;
    font-size: 0.85rem;
  }
  .select-panel select {
    width: 100%;
    padding: 10px;
    border: 1px solid #e2e8f0;
    border-radius: 6px;
    font-size: 0.9rem;
    box-sizing: border-box;
  }
  .compare-btn {
    padding: 12px 32px;
    background: var(--primary, #3b82f6);
    color: #fff;
    border: none;
    border-radius: 8px;
    font-weight: 600;
    cursor: pointer;
  }
  .compare-btn:disabled {
    opacity: 0.6;
  }
  .result-section {
    background: #fff;
    padding: 20px;
    border-radius: 8px;
    border: 1px solid #e2e8f0;
  }
  .compare-info {
    background: #fff;
    padding: 20px;
    border-radius: 8px;
    border: 1px solid #e2e8f0;
    color: #64748b;
  }
  .compare-info p {
    font-size: 0.85rem;
    margin: 8px 0;
  }
</style>
