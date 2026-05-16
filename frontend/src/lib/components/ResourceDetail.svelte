<script lang="ts">
  import { _ } from "svelte-i18n";
  import type { CapabilityResource } from "$lib/types";

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
    color: #94a3b8;
    padding: 40px 0;
  }
  .resource-detail {
    padding: 16px;
  }
  .detail-header {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 20px;
  }
  .detail-header h3 {
    margin: 0;
    font-size: 1.15rem;
  }
  .type-badge {
    padding: 3px 10px;
    border-radius: 12px;
    font-size: 0.75rem;
    text-transform: uppercase;
    background: #e0e7ff;
    color: #3730a3;
  }
  .detail-fields {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .field label {
    display: block;
    font-size: 0.75rem;
    color: #64748b;
    margin-bottom: 2px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .field code {
    background: #f1f5f9;
    padding: 2px 6px;
    border-radius: 3px;
    font-size: 0.85rem;
  }
  .field code.hash {
    font-size: 0.75rem;
    word-break: break-all;
  }
  .scope-badge {
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 0.8rem;
  }
  .scope-project { background: #dbeafe; color: #1e40af; }
  .scope-local { background: #fef3c7; color: #92400e; }
  .scope-user { background: #ede9fe; color: #5b21b6; }
  .scope-inherited { background: #fce7f3; color: #831843; }
  .scope-unknown { background: #f1f5f9; color: #64748b; }
  .metadata {
    background: #f8fafc;
    padding: 8px;
    border-radius: 4px;
    font-size: 0.8rem;
    max-height: 200px;
    overflow: auto;
    white-space: pre-wrap;
    word-break: break-all;
  }
  .error-box {
    margin-top: 16px;
    padding: 12px;
    background: #fef2f2;
    border: 1px solid #fecaca;
    border-radius: 6px;
  }
  .error-box strong {
    color: #dc2626;
    font-size: 0.85rem;
  }
  .error-box p {
    color: #991b1b;
    font-size: 0.85rem;
    margin: 4px 0 0 0;
  }
</style>
