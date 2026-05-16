<script lang="ts">
  import { Copy, Check } from 'lucide-svelte';

  let {
    code,
    language = 'text',
    maxHeight = '400px',
    showLineNumbers = true,
    class: className = '',
  }: {
    code: string;
    language?: string;
    maxHeight?: string;
    showLineNumbers?: boolean;
    class?: string;
  } = $props();

  let copied = $state(false);
  let lines = $derived(code.split('\n'));
  let lineCount = $derived(lines.length);

  async function handleCopy() {
    try {
      await navigator.clipboard.writeText(code);
      copied = true;
      setTimeout(() => copied = false, 2000);
    } catch { /* clipboard not available */ }
  }
</script>

<div class="code-viewer {className}" style="max-height: {maxHeight}">
  <div class="code-toolbar">
    <span class="code-lang">{language}</span>
    <button class="code-copy" onclick={handleCopy} aria-label="Copy code">
      {#if copied}
        <Check size={14} />
      {:else}
        <Copy size={14} />
      {/if}
    </button>
  </div>
  <pre class="code-content">
    <code class="code-lines">
      {#each lines as line, i (i)}
        <span class="code-line">
          {#if showLineNumbers}
            <span class="line-num">{i + 1}</span>
          {/if}
          <span class="line-text">{line}</span>
        </span>
      {/each}
    </code>
  </pre>
</div>

<style>
  .code-viewer {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    overflow: hidden;
    background: var(--bg-elevated);
  }
  .code-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 12px;
    background: var(--bg-card);
    border-bottom: 1px solid var(--border-default);
  }
  .code-lang {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  .code-copy {
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 2px;
    border-radius: var(--radius-sm);
  }
  .code-copy:hover {
    color: var(--text-secondary);
    background: var(--bg-hover);
  }
  .code-content {
    overflow: auto;
    padding: var(--space-3);
    margin: 0;
    font-family: var(--font-mono);
    font-size: var(--font-size-xs);
    line-height: 1.6;
  }
  .code-line {
    display: flex;
    gap: var(--space-3);
  }
  .line-num {
    color: var(--text-muted);
    text-align: right;
    min-width: 2ch;
    user-select: none;
    opacity: 0.5;
  }
  .line-text {
    color: var(--text-primary);
    white-space: pre;
  }
</style>
