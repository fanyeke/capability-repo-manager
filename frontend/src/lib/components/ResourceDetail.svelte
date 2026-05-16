<script lang="ts">
  import { _ } from 'svelte-i18n';
  import type { CapabilityResource } from '$lib/types';

  let {
    resource,
  }: {
    resource: CapabilityResource | null;
  } = $props();
</script>

{#if !resource}
  <div class="resource-empty">
    <p>{$_('resource.select_hint')}</p>
  </div>
{:else}
  <div class="resource-detail">
    <div class="detail-header">
      <h3>{resource.name}</h3>
      <span class="type-badge type-{resource.type}">{resource.type}</span>
    </div>

    <div class="detail-fields">
      <div class="field">
        <label>{$_('resource.scope')}</label>
        <span class="scope-badge scope-{resource.scope}">{resource.scope}</span>
      </div>

      {#if resource.source_path}
        <div class="field">
          <label>{$_('resource.source_path')}</label>
          <code>{resource.source_path}</code>
        </div>
      {/if}

      <div class="field">
        <label>{$_('resource.git_tracked')}</label>
        <span>{resource.tracked_by_git ? $_('resource.yes') : $_('resource.no')}</span>
      </div>

      {#if resource.content_hash}
        <div class="field">
          <label>{$_('resource.content_hash')}</label>
          <code class="hash">{resource.content_hash}</code>
        </div>
      {/if}

      {#if resource.metadata_json}
        <div class="field">
          <label>{$_('resource.metadata')}</label>
          <pre class="metadata">{resource.metadata_json}</pre>
        </div>
      {/if}
    </div>

    {#if resource.error_message}
      <div class="error-box">
        <strong>{$_('resource.parse_error')}</strong>
        <p>{resource.error_message}</p>
      </div>
    {/if}
  </div>
{/if}

<style>
  .resource-empty {
    text-align: center;
    color: var(--text-muted);
    padding: var(--space-8) 0;
  }
  .resource-detail {
    padding: var(--space-4);
  }
  .detail-header {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: var(--space-4);
  }
  .detail-header h3 {
    margin: 0;
    font-size: var(--font-size-md);
  }
  .type-badge {
    padding: 2px 8px;
    border-radius: var(--radius-full);
    font-size: var(--font-size-xs);
    text-transform: uppercase;
    background: var(--color-info-bg);
    color: var(--color-info);
  }
  .detail-fields {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }
  .field label {
    display: block;
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    margin-bottom: 2px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .field code {
    background: var(--bg-elevated);
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
  }
  .field code.hash {
    font-size: var(--font-size-xs);
    word-break: break-all;
  }
  .scope-badge {
    padding: 2px 8px;
    border-radius: var(--radius-sm);
    font-size: var(--font-size-sm);
  }
  .scope-project { background: var(--color-info-bg); color: var(--color-info); }
  .scope-local { background: var(--color-warning-bg); color: var(--color-warning); }
  .scope-user { background: var(--color-primary-bg); color: var(--color-primary-text); }
  .scope-inherited { background: var(--color-danger-bg); color: var(--color-danger); }
  .scope-unknown { background: var(--bg-elevated); color: var(--text-muted); }
  .metadata {
    background: var(--bg-elevated);
    padding: var(--space-2);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-xs);
    max-height: 200px;
    overflow: auto;
    white-space: pre-wrap;
    word-break: break-all;
    color: var(--text-secondary);
  }
  .error-box {
    margin-top: var(--space-4);
    padding: var(--space-3);
    background: var(--color-danger-bg);
    border: 1px solid var(--color-danger);
    border-radius: var(--radius-md);
  }
  .error-box strong {
    color: var(--color-danger);
    font-size: var(--font-size-sm);
  }
  .error-box p {
    color: var(--color-danger);
    font-size: var(--font-size-sm);
    margin: var(--space-1) 0 0 0;
  }
</style>
