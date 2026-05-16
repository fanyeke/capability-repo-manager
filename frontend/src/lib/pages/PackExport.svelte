<script lang="ts">
  import { _ } from 'svelte-i18n';
  import { currentPage, navigateTo } from '$lib/stores/uiStore';
  import { selectedRepoDetail } from '$lib/stores/repoStore';
  import { resources, loadCapabilityInventory, typeGroups, filteredResources } from '$lib/stores/capabilityStore';
  import { get } from 'svelte/store';
import { exportPack, isLoading, error } from '$lib/stores/packStore';
  import { ArrowLeft, Package, Upload, Check } from 'lucide-svelte';
  import Button from '$lib/components/Button.svelte';
  import Card from '$lib/components/Card.svelte';
  import Banner from '$lib/components/Banner.svelte';
  import Badge from '$lib/components/Badge.svelte';
  import ResourceIcon from '$lib/components/ResourceIcon.svelte';
  import Skeleton from '$lib/components/Skeleton.svelte';
  import type { CapabilityResource } from '$lib/types';

  let step = $state<'select' | 'metadata' | 'export'>('select');
  let packName = $state('');
  let packVersion = $state('1.0.0');
  let packDescription = $state('');
  let packType = $state('project');
  let detail = $derived($selectedRepoDetail);
  let selectedIds = $state<Set<string>>(new Set());
  let exportError = $state<string | null>(null);

  function getResourcesByType(inv: any, resourceType: string): any[] {
    if (!inv) return [];
    for (const group of Object.values(inv)) {
      const arr = group as any[];
      if (arr.length > 0 && arr[0]?.type === resourceType) {
        return arr;
      }
    }
    return [];
  }

  function toggleResource(id: string) {
    if (selectedIds.has(id)) {
      selectedIds.delete(id);
    } else {
      selectedIds.add(id);
    }
  }

  function selectAllForType(type: string) {
    const inv = get(resources);
    const typeResources = getResourcesByType(inv, type) as CapabilityResource[];
    const allSelected = typeResources.every((r) => selectedIds.has(r.id));
    for (const r of typeResources) {
      if (allSelected) selectedIds.delete(r.id);
      else selectedIds.add(r.id);
    }
  }

  async function handleExport() {
    if (!detail?.repo.id || selectedIds.size === 0) return;
    exportError = null;
    step = 'export';

    const result = await exportPack(
      detail.repo.id,
      { resource_ids: Array.from(selectedIds) },
      {
        name: packName,
        version: packVersion,
        description: packDescription || undefined,
        pack_type: packType,
      },
    );

    if (result) {
      currentPage.set('packapply');
    } else {
      exportError = get(error);
      step = 'metadata';
    }
  }

  const packTypeOptions = [
    { value: 'project', label: $_('pack_export.type_project') },
    { value: 'blueprint', label: $_('pack_export.type_blueprint') },
    { value: 'baseline', label: $_('pack_export.type_baseline') },
  ];

  const steps = [
    { id: 'select', label: $_('pack_export.step_select') },
    { id: 'metadata', label: $_('pack_export.step_metadata') },
    { id: 'export', label: $_('pack_export.step_export') },
  ];
</script>

<div class="pack-export">
  <div class="page-top">
    <Button variant="ghost" size="sm" onclick={() => navigateTo('dashboard')}>
      <ArrowLeft size={14} />
      {$_('nav.back')}
    </Button>
    <h1>{$_('pack_export.title')}</h1>
  </div>

  <div class="stepper">
    {#each steps as s, i (s.id)}
      <div class="step" class:active={step === s.id} class:done={['metadata', 'export'].includes(step) && s.id === 'select' || step === 'export' && s.id === 'metadata'}>
        <span class="step-num">
          {#if ['metadata', 'export'].includes(step) && s.id === 'select' || step === 'export' && s.id === 'metadata'}
            <Check size={12} />
          {:else}
            {i + 1}
          {/if}
        </span>
        <span class="step-label">{s.label}</span>
      </div>
      {#if i < steps.length - 1}
        <span class="step-line"></span>
      {/if}
    {/each}
  </div>

  {#if !detail}
    <Skeleton variant="card" />
  {:else}
    {#if exportError}
      <Banner type="error" dismissible={false}>{$_('pack_export.error')}: {exportError}</Banner>
    {/if}

    {#if step === 'select'}
      <div class="select-layout">
        <div class="resource-groups">
          {#each $typeGroups as group (group.type)}
            {#if group.count > 0}
              <Card padding="md">
                {#snippet title()}
                  <div class="group-header">
                    <ResourceIcon type={group.type as any} size={16} />
                    <span>{group.label}</span>
                    <Badge variant="default" size="sm">{group.count}</Badge>
                  </div>
                {/snippet}
                {#snippet actions()}
                  <Button variant="ghost" size="sm" onclick={() => selectAllForType(group.type)}>
                    {$_('pack_export.select_all')}
                  </Button>
                {/snippet}
                <div class="resource-check-list">
                  {#each getResourcesByType($resources, group.type) as resource (resource.id)}
                    <label class="resource-item">
                      <input
                        type="checkbox"
                        checked={selectedIds.has(resource.id)}
                        onchange={() => toggleResource(resource.id)}
                      />
                      <ResourceIcon type={resource.type} size={12} />
                      <span class="resource-name">{resource.name}</span>
                      {#if resource.error_message}
                        <Badge variant="danger" size="sm">{$_('pack_export.error_tag')}</Badge>
                      {/if}
                    </label>
                  {/each}
                </div>
              </Card>
            {/if}
          {/each}
        </div>
        <div class="select-sidebar">
          <Card padding="md">
            {#snippet title()}{$_('pack_export.summary')}{/snippet}
            <div class="summary-stats">
              <div class="stat">
                <span class="stat-label">{$_('pack_export.selected')}</span>
                <span class="stat-value">{selectedIds.size}</span>
              </div>
              <div class="stat">
                <span class="stat-label">{$_('pack_export.total')}</span>
                <span class="stat-value">{$typeGroups.reduce((s, g) => s + g.count, 0)}</span>
              </div>
            </div>
            <Button variant="primary" onclick={() => step = 'metadata'} disabled={selectedIds.size === 0} class="continue-btn">
              {$_('common.next')}
            </Button>
          </Card>
        </div>
      </div>

    {:else if step === 'metadata'}
      <div class="metadata-layout">
        <Card padding="lg">
          {#snippet title()}{$_('pack_export.metadata')}{/snippet}
          <div class="form-group">
            <label class="form-label">{$_('pack_export.name')} *</label>
            <input type="text" bind:value={packName} class="form-input" required />
          </div>
          <div class="form-group">
            <label class="form-label">{$_('pack_export.version')} *</label>
            <input type="text" bind:value={packVersion} class="form-input" />
          </div>
          <div class="form-group">
            <label class="form-label">{$_('pack_export.description')}</label>
            <textarea bind:value={packDescription} class="form-textarea" rows={3}></textarea>
          </div>
          <div class="form-group">
            <label class="form-label">{$_('pack_export.type')}</label>
            <select bind:value={packType} class="form-select">
              {#each packTypeOptions as opt (opt.value)}
                <option value={opt.value}>{opt.label}</option>
              {/each}
            </select>
          </div>
          <div class="step-actions">
            <Button variant="ghost" onclick={() => step = 'select'}>{$_('common.back')}</Button>
            <Button variant="primary" onclick={handleExport} disabled={!packName || selectedIds.size === 0}>
              <Upload size={14} />
              {$_('pack_export.export')}
            </Button>
          </div>
        </Card>

        <Card padding="md">
          {#snippet title()}{$_('pack_export.preview')}{/snippet}
          <div class="preview-info">
            <p><strong>{$_('pack_export.selected', { values: { n: selectedIds.size } })}</strong></p>
            <p><strong>{$_('pack_export.source', { values: { name: detail.repo.name } })}</strong></p>
            {#if packName}
              <p><strong>Name:</strong> {packName}</p>
            {/if}
          </div>
        </Card>
      </div>

    {:else if step === 'export'}
      <Card padding="lg">
        <div class="exporting-state">
          <Package size={40} class="export-icon" />
          <h2>{$_('pack_export.exporting')}</h2>
          <div class="export-progress">
            <Skeleton variant="text" />
            <Skeleton variant="text" />
          </div>
        </div>
      </Card>
    {/if}
  {/if}
</div>

<style>
  .pack-export {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding-top: var(--space-4);
  }
  .page-top {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }
  .page-top h1 {
    margin: 0;
    font-size: var(--font-size-xl);
  }
  .stepper {
    display: flex;
    align-items: center;
    gap: 0;
  }
  .step {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--font-size-sm);
    color: var(--text-muted);
  }
  .step.active { color: var(--text-primary); }
  .step.done { color: var(--color-success); }
  .step-num {
    width: 24px; height: 24px;
    border-radius: var(--radius-full);
    display: flex; align-items: center; justify-content: center;
    font-size: 11px; font-weight: 600;
    border: 1px solid var(--border-default);
    background: var(--bg-card);
  }
  .step.active .step-num { border-color: var(--color-primary); color: var(--color-primary); }
  .step.done .step-num { border-color: var(--color-success); background: var(--color-success); color: #fff; }
  .step-line {
    flex: 1;
    height: 1px;
    background: var(--border-default);
    margin: 0 var(--space-3);
    max-width: 60px;
  }

  .select-layout {
    display: grid;
    grid-template-columns: 1fr 260px;
    gap: var(--space-4);
  }
  .resource-groups {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }
  .group-header {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }
  .resource-check-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    max-height: 300px;
    overflow-y: auto;
  }
  .resource-item {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: 4px 0;
    font-size: var(--font-size-sm);
    cursor: pointer;
    color: var(--text-secondary);
  }
  .resource-item:hover { color: var(--text-primary); }
  .resource-name { flex: 1; }
  .select-sidebar { display: flex; flex-direction: column; gap: var(--space-3); }
  .summary-stats { display: flex; flex-direction: column; gap: var(--space-2); margin-bottom: var(--space-3); }
  .stat { display: flex; justify-content: space-between; font-size: var(--font-size-sm); }
  .stat-label { color: var(--text-muted); }
  .stat-value { color: var(--text-primary); font-weight: 600; }
  .continue-btn { width: 100%; }

  .metadata-layout {
    display: grid;
    grid-template-columns: 1fr 280px;
    gap: var(--space-4);
  }
  .form-group { margin-bottom: var(--space-4); }
  .form-label {
    display: block;
    font-size: var(--font-size-sm);
    font-weight: 500;
    color: var(--text-secondary);
    margin-bottom: var(--space-1);
  }
  .form-input, .form-textarea, .form-select {
    width: 100%;
    padding: 8px 12px;
    background: var(--bg-card);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    color: var(--text-primary);
    font-size: var(--font-size-sm);
    font-family: var(--font-sans);
  }
  .form-input:focus, .form-textarea:focus, .form-select:focus {
    outline: none;
    border-color: var(--color-primary);
  }
  .form-textarea { resize: vertical; }
  .step-actions {
    display: flex;
    justify-content: space-between;
    margin-top: var(--space-4);
  }

  .preview-info p { margin: 0 0 var(--space-1); font-size: var(--font-size-sm); color: var(--text-secondary); }
  .exporting-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: var(--space-8);
    text-align: center;
  }
  .export-icon { color: var(--color-primary); margin-bottom: var(--space-3); }
  .exporting-state h2 { margin: 0 0 var(--space-4); font-size: var(--font-size-lg); }
  .export-progress { width: 100%; max-width: 400px; display: flex; flex-direction: column; gap: var(--space-2); }
</style>
