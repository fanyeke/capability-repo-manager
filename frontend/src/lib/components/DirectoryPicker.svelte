<script lang="ts">
  import { _ } from 'svelte-i18n';
  import { FolderOpen, X, Plus } from 'lucide-svelte';
  import Button from './Button.svelte';
  import Chip from './Chip.svelte';

  let {
    directories = [],
    onChange,
    placeholder = $_('directory_picker.placeholder'),
    class: className = '',
  }: {
    directories: string[];
    onChange: (dirs: string[]) => void;
    placeholder?: string;
    class?: string;
  } = $props();

  let manualInput = $state('');

  async function pickDirectory() {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const selected = await open({
        directory: true,
        multiple: true,
        title: $_('directory_picker.title'),
      });
      if (selected) {
        const paths = Array.isArray(selected) ? selected : [selected];
        const unique = paths.filter((p) => !directories.includes(p));
        onChange([...directories, ...unique]);
      }
    } catch (e) {
      // Fallback: dialog not available (e.g. in dev mode without Tauri)
      console.warn('Directory picker not available:', e);
    }
  }

  function addManual() {
    const path = manualInput.trim();
    if (path && !directories.includes(path)) {
      onChange([...directories, path]);
      manualInput = '';
    }
  }

  function removeDir(dir: string) {
    onChange(directories.filter((d) => d !== dir));
  }

  function handleManualKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      addManual();
    }
  }
</script>

<div class="directory-picker {className}">
  <div class="picker-actions">
    <Button variant="secondary" size="sm" onclick={pickDirectory}>
      <FolderOpen size={14} />
      {$_('directory_picker.browse')}
    </Button>
    <span class="picker-divider">{$_('directory_picker.or')}</span>
    <div class="manual-row">
      <input
        type="text"
        bind:value={manualInput}
        {placeholder}
        onkeydown={handleManualKeydown}
        class="manual-input"
      />
      <Button variant="ghost" size="sm" onclick={addManual} disabled={!manualInput.trim()}>
        <Plus size={14} />
      </Button>
    </div>
  </div>
  <Chip items={directories} onRemove={removeDir} />
</div>

<style>
  .directory-picker {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }
  .picker-actions {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    flex-wrap: wrap;
  }
  .picker-divider {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
  }
  .manual-row {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    flex: 1;
    min-width: 200px;
  }
  .manual-input {
    flex: 1;
    padding: 6px 10px;
    background: var(--bg-card);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    color: var(--text-primary);
    font-size: var(--font-size-sm);
    outline: none;
  }
  .manual-input:focus {
    border-color: var(--color-primary);
  }
</style>
