<script lang="ts">
  import { _ } from 'svelte-i18n';
  let {
    title = '出现了问题',
    message = '',
    detail = '',
  }: {
    title?: string;
    message?: string;
    detail?: string;
  } = $props();

  let copied = $state(false);

  function handleRetry() {
    location.reload();
  }

  function copyError() {
    navigator.clipboard.writeText(`Title: ${title}\nMessage: ${message}\nDetail: ${detail}`);
    copied = true;
    setTimeout(() => (copied = false), 2000);
  }
</script>

<div class="boot-error">
  <div class="error-card">
    <h1>{title}</h1>
    {#if message}
      <p class="error-message">{message}</p>
    {/if}
    {#if detail}
      <pre class="error-detail">{detail}</pre>
    {/if}
    <div class="error-actions">
      <button class="btn-primary" onclick={handleRetry}>{$_('app.retry')}</button>
      {#if detail}
        <button class="btn-secondary" onclick={copyError}>
          {copied ? $_('app.copied') : $_('app.copy_error')}
        </button>
      {/if}
    </div>
  </div>
</div>

<style>
  .boot-error {
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: 100vh;
    background: var(--bg-elevated);
    padding: 24px;
  }
  .error-card {
    max-width: 560px;
    width: 100%;
    background: var(--bg-card);
    border: 1px solid var(--border-default);
    border-radius: 12px;
    padding: 32px;
    box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1);
  }
  h1 {
    margin: 0 0 12px 0;
    font-size: 1.5rem;
    color: #dc2626;
  }
  .error-message {
    color: var(--text-muted);
    margin: 0 0 16px 0;
    line-height: 1.5;
  }
  .error-detail {
    background: #f1f5f9;
    padding: 12px;
    border-radius: 8px;
    font-size: 0.8rem;
    overflow-x: auto;
    white-space: pre-wrap;
    word-break: break-all;
    margin: 0 0 20px 0;
  }
  .error-actions {
    display: flex;
    gap: 8px;
  }
  .btn-primary {
    padding: 10px 20px;
    background: #3b82f6;
    color: #fff;
    border: none;
    border-radius: 8px;
    font-weight: 600;
    cursor: pointer;
  }
  .btn-secondary {
    padding: 10px 20px;
    background: var(--bg-card);
    color: #0f172a;
    border: 1px solid var(--border-default);
    border-radius: 8px;
    cursor: pointer;
  }
</style>
