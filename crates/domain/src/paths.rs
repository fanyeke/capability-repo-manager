/// Centralized constants for Claude Code directory structure.
///
/// All crates should reference these constants instead of hardcoding
/// directory/file names, so that changes (e.g. `.claude/rules` → `.claude/instructions`)
/// require only one update.

/// Root Claude Code config directory within a repository.
pub const CLAUDE_DIR: &str = ".claude";

/// MCP server configuration file.
pub const MCP_JSON: &str = "mcp.json";

/// Claude Code settings file (hooks, model config, etc.).
pub const SETTINGS_JSON: &str = "settings.json";

/// Directory for custom skills.
pub const SKILLS_DIR: &str = "skills";

/// Directory for custom MCP servers.
pub const MCP_DIR: &str = "mcp";

/// Directory for hook configurations.
pub const HOOKS_DIR: &str = "hooks";

/// Directory for coding rules.
pub const RULES_DIR: &str = "rules";

/// Directory for agent definitions.
pub const AGENTS_DIR: &str = "agents";

/// Directory for custom commands.
pub const COMMANDS_DIR: &str = "commands";

/// Directory for plugin definitions.
pub const PLUGINS_DIR: &str = "plugins";

/// Skill markdown filename (placed inside each skill subdirectory).
pub const SKILL_MD: &str = "SKILL.md";

/// Agent definition markdown filename.
pub const AGENT_MD: &str = "agent.md";

/// Full path to `.claude/mcp.json` within a repo.
pub fn mcp_json_path(repo_root: &std::path::Path) -> std::path::PathBuf {
    repo_root.join(CLAUDE_DIR).join(MCP_JSON)
}

/// Full path to `.claude/settings.json` within a repo.
pub fn settings_json_path(repo_root: &std::path::Path) -> std::path::PathBuf {
    repo_root.join(CLAUDE_DIR).join(SETTINGS_JSON)
}

/// Full path to `.claude/rules/` within a repo.
pub fn rules_dir_path(repo_root: &std::path::Path) -> std::path::PathBuf {
    repo_root.join(CLAUDE_DIR).join(RULES_DIR)
}

/// Full path to `.claude/skills/` within a repo.
pub fn skills_dir_path(repo_root: &std::path::Path) -> std::path::PathBuf {
    repo_root.join(CLAUDE_DIR).join(SKILLS_DIR)
}

/// Full path to `.claude/agents/` within a repo.
pub fn agents_dir_path(repo_root: &std::path::Path) -> std::path::PathBuf {
    repo_root.join(CLAUDE_DIR).join(AGENTS_DIR)
}
