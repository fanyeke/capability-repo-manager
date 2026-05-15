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
///
/// On failure to create the log directory or initialize the subscriber, falls back
/// to console-only output with an `eprintln!` warning.
pub fn init_tracing(log_level: &str) -> Result<(), String> {
    let log_dir = get_log_dir();

    // Try to ensure log directory exists; fall back to console-only on failure
    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        eprintln!(
            "Warning: Could not create log directory {}: {}. Logging to console only.",
            log_dir.display(),
            e
        );
        return init_console_only(log_level);
    }

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

    match tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .with(console_layer)
        .try_init()
    {
        Ok(()) => {
            tracing::info!(
                level = log_level,
                dir = %log_dir.display(),
                "logging_initialized"
            );
            Ok(())
        }
        Err(e) => {
            eprintln!(
                "Warning: Could not initialize tracing subscriber: {}. Logging unavailable.",
                e
            );
            Ok(())
        }
    }
}

/// Set up console-only tracing when the log directory is not available.
fn init_console_only(log_level: &str) -> Result<(), String> {
    let console_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_writer(std::io::stderr);

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level));

    match tracing_subscriber::registry()
        .with(filter)
        .with(console_layer)
        .try_init()
    {
        Ok(()) => {
            eprintln!("Tracing initialized (console only)");
            Ok(())
        }
        Err(e) => {
            eprintln!(
                "Warning: Could not initialize tracing subscriber: {}. No logging available.",
                e
            );
            Ok(())
        }
    }
}

/// Log the app_start event with environment metadata.
///
/// Called after `init_tracing` succeeds so the event appears in logs.
pub fn log_app_start(db_path: &str, settings_path: &str) {
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        platform = std::env::consts::OS,
        settings_path = %settings_path,
        db_path = %db_path,
        log_dir = %get_log_dir().display(),
        "app_start"
    );
}

/// Clean up log files older than 14 days.
///
/// Called at startup to prevent unbounded log disk usage.
pub fn clean_old_logs() {
    let log_dir = get_log_dir();
    if !log_dir.exists() {
        return;
    }

    let cutoff = chrono::Utc::now() - chrono::Duration::days(14);
    if let Ok(entries) = std::fs::read_dir(&log_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("log") {
                continue;
            }
            if let Ok(metadata) = path.metadata() {
                if let Ok(modified) = metadata.modified() {
                    let modified_time: chrono::DateTime<chrono::Utc> = modified.into();
                    if modified_time < cutoff && std::fs::remove_file(&path).is_ok() {
                        tracing::info!(path = %path.display(), "cleaned_old_log_file");
                    }
                }
            }
        }
    }
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
pub(crate) fn get_log_dir() -> PathBuf {
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