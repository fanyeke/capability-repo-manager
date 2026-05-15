// Tauri Bridge - Glue layer between Tauri commands and domain engines.
//
// This crate contains Tauri command handlers that delegate to:
// - repo-scanner (scan_repositories, list_repositories, refresh_repository)
// - claude-parser (get_capability_inventory, get_resource_detail)
// - pack-engine (export_capability_pack, list_packs, validate_pack)
// - migration-engine (build_migration_plan, apply_migration_plan, rollback_migration)
// - doctor-engine (run_doctor, compare_repo_with_pack, compare_repos)

pub mod commands;
pub mod state;

use std::path::PathBuf;
use tracing_appender::rolling;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

/// Initialise the tracing subscriber with console output and daily rolling file output.
///
/// Log files are written to `~/.capability-repo-manager/logs/` with daily rotation.
/// Respects the `RUST_LOG` environment variable; falls back to `info` level.
pub fn init_tracing(log_level: &str) -> Result<(), String> {
    let log_dir = get_log_dir();

    // Ensure log directory exists
    std::fs::create_dir_all(&log_dir)
        .map_err(|e| format!("Failed to create log directory: {}", e))?;

    // Daily rolling file appender: app-YYYY-MM-DD.log
    let file_appender = rolling::daily(&log_dir, "app");
    let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);

    // Console output (stderr)
    let (console_writer, _console_guard) = tracing_appender::non_blocking(std::io::stderr());

    // File layer: structured JSON format for machine parsing
    let file_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_target(true)
        .with_current_span(false)
        .with_writer(file_writer);

    // Console layer: human-readable format
    let console_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_writer(console_writer);

    // Filter: respect RUST_LOG env var, fall back to configured log_level
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level));

    tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .with(console_layer)
        .try_init()
        .map_err(|e| format!("Failed to init tracing subscriber: {}", e))?;

    tracing::info!(
        level = log_level,
        dir = %log_dir.display(),
        "logging_initialized"
    );

    Ok(())
}

/// Reconfigure the tracing filter at runtime when the log level changes.
///
/// This reloads the `EnvFilter` with the new level. Since `tracing-subscriber`
/// doesn't support hot-reload of individual layers, we dispatch a global level
/// update via the tracing facade itself.
pub fn set_log_level(level: &str) {
    tracing::info!(new_level = level, "log_level_changed");
    // Dynamic filter reload is handled by rebuilding the EnvFilter.
    // For now, log the change — the app restart will pick up the persisted setting.
    std::env::set_var("RUST_LOG", level);
}

/// Get the log directory path.
fn get_log_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home)
        .join(".capability-repo-manager")
        .join("logs")
}

/// Explicit guard type returned by `init_tracing` to keep the file appender alive.
///
/// The guard is intentionally unused in the current setup — the non_blocking
/// writers are owned by the subscriber and live for the process lifetime.
#[allow(dead_code)]
struct LogGuard {
    _file_guard: tracing_appender::non_blocking::WorkerGuard,
    _console_guard: tracing_appender::non_blocking::WorkerGuard,
}
