<script lang="ts">
  import { _ } from 'svelte-i18n';
  import type { CapabilityResource, ResourceContent } from '$lib/types';
  import { FileText, Code, Archive } from 'lucide-svelte';
  import Badge from './Badge.svelte';
  import StatusPill from './StatusPill.svelte';
  import ResourceIcon from './ResourceIcon.svelte';
  import Tabs from './Tabs.svelte';
  import CodeViewer from './CodeViewer.svelte';
  import Banner from './Banner.svelte';
  import Skeleton from './Skeleton.svelte';

  let {
    resource,
  }: {
    resource: CapabilityResource | null;
  } = $props();

  let activeTab = $state<'overview' | 'content' | 'metadata'>('overview');
  let content = $state<ResourceContent | null>(null);
  let contentLoading = $state(false);
  let contentError = $state<string | null>(null);

  $effect(() => {
    if (resource && activeTab === 'content') {
      loadContent();
    }
  });

  async function loadContent() {
    if (!resource || content) return;
    contentLoading = true;
    contentError = null;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const result = await invoke<ResourceContent>('get_resource_content', {
        resourceId: resource.id,
      });
      content = result;
    } catch (e: any) {
      contentError = e?.message ?? String(e);
    } finally {
      contentLoading = false;
    }
  }

  const tabs = [
    { id: 'overview', label: $_('resource.tab_overview') },
    { id: 'content', label: $_('resource.tab_content') },
    { id: 'metadata', label: $_('resource.tab_metadata') },
  ];

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB'];
    const i = Math.min(Math.floor(Math.log(bytes) / Math.log(k)), sizes.length - 1);
    return `${(bytes / Math.pow(k, i)).toFixed(i > 0 ? 1 : 0)} ${sizes[i]}`;
  }

  function formatMetadata(json: string): string {
    try {
      return JSON.stringify(JSON.parse(json), null, 2);
    } catch {
      return json;
    }
  }
</script>

{#if !resource}
  <div class="resource-empty">{$_('resource.select_hint')}</div>
{:else}
  <Tabs tabs={tabs} active={activeTab} onChange={(id) => activeTab = id as any} />

  {#if activeTab === 'overview'}
    <div class="overview-section">
      <div class="overview-header">
        <ResourceIcon type={resource.type} size={20} />
        <h3>{resource.name}</h3>
        <Badge variant="info">{resource.type}</Badge>
      </div>
      <dl class="overview-fields">
        <div class="field-row"><dt>{$_('resource.scope')}</dt><dd><StatusPill status={resource.scope === 'project' ? 'info' : resource.scope === 'local' ? 'warning' : 'info'} label={resource.scope} /></dd></div>
        <div class="field-row"><dt>{$_('resource.source_path')}</dt><dd class="mono">{resource.source_path ?? '—'}</dd></div>
        <div class="field-row"><dt>{$_('resource.git_tracked')}</dt><dd>{resource.tracked_by_git ? $_('resource.yes') : $_('resource.no')}</dd></div>
        <div class="field-row"><dt>Hash</dt><dd class="mono hash">{resource.content_hash ?? '—'}</dd></div>
        {#if content}
          <div class="field-row"><dt>{$_('resource.size')}</dt><dd>{formatBytes(content.size_bytes)}</dd></div>
        {/if}
      </dl>
      {#if resource.error_message}
        <Banner type="error" dismissible={false}>
          {resource.error_message}
        </Banner>
      {/if}
    </div>

  {:else if activeTab === 'content'}
    <div class="content-section">
      {#if contentLoading}
        <Skeleton variant="card" />
        <Skeleton variant="text" />
        <Skeleton variant="text" />
      {:else if contentError}
        <Banner type="error" dismissible={false}>{contentError}</Banner>
      {:else if content}
        {#if content.is_binary}
          <Banner type="info" dismissible={false}>
            <Archive size={14} /> {$_('resource.binary_file')} ({formatBytes(content.size_bytes)})
          </Banner>
        {:else if content.content}
          <CodeViewer code={content.content} language={content.language} maxHeight="500px" />
        {:else if content.error}
          <Banner type="warning" dismissible={false}>{content.error}</Banner>
        {/if}
      {:else}
        <div class="content-placeholder">
          <Code size={32} />
          <p>{$_('resource.load_content')}</p>
        </div>
      {/if}
    </div>

  {:else if activeTab === 'metadata'}
    <div class="metadata-section">
      {#if resource.metadata_json}
        <CodeViewer code={formatMetadata(resource.metadata_json)} language="json" maxHeight="400px" />
      {:else}
        <p class="empty-meta">{$_('resource.no_metadata')}</p>
      {/if}
    </div>
  {/if}
{/if}

<style>
  .resource-empty {
    text-align: center;
    color: var(--text-muted);
    padding: var(--space-8);
  }
  .overview-section {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    padding: var(--space-3) 0;
  }
  .overview-header {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }
  .overview-header h3 {
    margin: 0;
    font-size: var(--font-size-md);
    flex: 1;
  }
  .overview-fields {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    margin: 0;
  }
  .field-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: var(--font-size-sm);
    padding: var(--space-1) 0;
    border-bottom: 1px solid var(--border-default);
  }
  .field-row dt {
    color: var(--text-muted);
    font-size: var(--font-size-xs);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .field-row dd {
    margin: 0;
    color: var(--text-secondary);
  }
  .field-row .mono {
    font-family: var(--font-mono);
    font-size: var(--font-size-xs);
  }
  .field-row .hash {
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .content-section {
    padding: var(--space-2) 0;
  }
  .content-placeholder {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-8);
    color: var(--text-muted);
  }
  .metadata-section {
    padding: var(--space-2) 0;
  }
  .empty-meta {
    text-align: center;
    color: var(--text-muted);
    padding: var(--space-4);
  }
</style>
