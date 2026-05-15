<script lang="ts">
  import { scanRepositories, loadRepos, repos } from "$lib/stores/repoStore";
  import { currentPage } from "$lib/stores/uiStore";

  let scanPaths = $state("");
  let isScanning = $state(false);
  let showSkip = $state(false);

  async function handleScan() {
    if (!scanPaths.trim()) return;
    const paths = scanPaths.split(/[,\s]+/).filter((p) => p);
    isScanning = true;
    try {
      await scanRepositories(paths);
      currentPage.set("dashboard");
    } catch (e) {
      console.error("Scan failed:", e);
    } finally {
      isScanning = false;
    }
  }

  function handleSkip() {
    currentPage.set("dashboard");
  }
</script>

<div class="guided-setup">
  <h1>Welcome to Capability Repo Manager</h1>
  <p class="intro">
    Discover and manage Claude Code capabilities across your local Git repositories.
  </p>

  <div class="setup-step">
    <h2>Add Scan Directories</h2>
    <p class="step-desc">
      Enter the paths to directories containing your Git repositories. Separate multiple paths with commas or spaces.
    </p>

    <textarea
      bind:value={scanPaths}
      placeholder="e.g. /home/user/projects, /home/user/work"
      rows={3}
    ></textarea>

    <div class="setup-actions">
      <button class="primary-btn" onclick={handleScan} disabled={isScanning}>
        {isScanning ? "Scanning..." : "Scan Repositories"}
      </button>
      {#if isScanning}
        <button class="skip-btn" onclick={() => { isScanning = false; currentPage.set("dashboard"); }}>Cancel</button>
      {:else if showSkip || !scanPaths.trim()}
        <button class="skip-btn" onclick={handleSkip}>Skip for now</button>
      {/if}
    </div>
  </div>

  <div class="setup-info">
    <h3>What will be discovered?</h3>
    <ul>
      <li><strong>Skills</strong> — Custom slash commands and agent skills</li>
      <li><strong>MCP Servers</strong> — Model Context Protocol servers</li>
      <li><strong>Hooks</strong> — Pre/post execution hooks</li>
      <li><strong>Rules</strong> — Project-specific rules and guidelines</li>
      <li><strong>Agents</strong> — Custom agent definitions</li>
      <li><strong>Commands</strong> — Custom Claude Code commands</li>
    </ul>
  </div>
</div>

<style>
  .guided-setup {
    max-width: 600px;
    margin: 0 auto;
    padding: 40px 20px;
  }
  h1 {
    text-align: center;
    margin-bottom: 8px;
  }
  .intro {
    text-align: center;
    color: #64748b;
    margin-bottom: 32px;
  }
  .setup-step {
    background: #f8fafc;
    padding: 24px;
    border-radius: 12px;
    margin-bottom: 24px;
  }
  .setup-step h2 {
    margin: 0 0 8px 0;
    font-size: 1.1rem;
  }
  .step-desc {
    color: #475569;
    margin-bottom: 16px;
  }
  textarea {
    width: 100%;
    padding: 12px;
    border: 1px solid #e2e8f0;
    border-radius: 8px;
    font-size: 0.95rem;
    resize: vertical;
    box-sizing: border-box;
  }
  .setup-actions {
    display: flex;
    gap: 12px;
    margin-top: 16px;
  }
  .primary-btn {
    padding: 12px 24px;
    background: var(--primary, #3b82f6);
    color: #fff;
    border: none;
    border-radius: 8px;
    font-weight: 600;
    cursor: pointer;
  }
  .primary-btn:disabled {
    opacity: 0.6;
  }
  .skip-btn {
    padding: 12px 24px;
    background: #fff;
    color: #64748b;
    border: 1px solid #e2e8f0;
    border-radius: 8px;
    cursor: pointer;
  }
  .setup-info {
    background: #fff;
    padding: 20px;
    border: 1px solid #e2e8f0;
    border-radius: 8px;
  }
  .setup-info h3 {
    margin: 0 0 12px 0;
    font-size: 0.95rem;
    color: #475569;
  }
  ul {
    margin: 0;
    padding-left: 20px;
  }
  li {
    font-size: 0.85rem;
    color: #64748b;
    margin-bottom: 6px;
  }
  li strong {
    color: #334155;
  }
</style>