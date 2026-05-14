use domain::{
    CapabilityResource, MigrationPlan, MigrationReport, MigrationReportItem,
    MigrationReportSummary,
};
use std::fs;
use std::path::Path;

pub fn execute_plan(
    plan: &MigrationPlan,
    pack_resources: &[CapabilityResource],
    pack_dir: &Path,
    target_dir: &Path,
) -> Result<MigrationReport, domain::AppError> {
    let mut report_items = Vec::new();
    let mut summary = MigrationReportSummary {
        added: 0,
        overwritten: 0,
        skipped: 0,
        failed: 0,
    };

    for item in &plan.items {
        let pack_res = pack_resources.iter().find(|r| r.id == item.resource_id);

        let result = match item.action.as_str() {
            "add" | "overwrite" | "rename" => {
                let src_path = item.source_path.as_deref().unwrap_or("");
                let dst_path = item.target_path.as_deref().unwrap_or(src_path);

                let full_src = pack_dir.join(src_path);
                let full_dst = target_dir.join(dst_path);

                copy_resource(&full_src, &full_dst)
            }
            "skip" => Ok(ExecResult::Skipped),
            _ => Ok(ExecResult::Skipped),
        };

        match result {
            Ok(ExecResult::Copied) => match item.action.as_str() {
                "add" | "rename" => {
                    summary.added += 1;
                    report_items.push(MigrationReportItem {
                        resource_name: pack_res.map(|r| r.name.clone()).unwrap_or_default(),
                        action: item.action.clone(),
                        status: "success".to_string(),
                        error: None,
                    });
                }
                "overwrite" => {
                    summary.overwritten += 1;
                    report_items.push(MigrationReportItem {
                        resource_name: pack_res.map(|r| r.name.clone()).unwrap_or_default(),
                        action: item.action.clone(),
                        status: "success".to_string(),
                        error: None,
                    });
                }
                _ => {
                    summary.added += 1;
                    report_items.push(MigrationReportItem {
                        resource_name: pack_res.map(|r| r.name.clone()).unwrap_or_default(),
                        action: item.action.clone(),
                        status: "success".to_string(),
                        error: None,
                    });
                }
            },
            Ok(ExecResult::Skipped) => {
                summary.skipped += 1;
                report_items.push(MigrationReportItem {
                    resource_name: pack_res.map(|r| r.name.clone()).unwrap_or_default(),
                    action: item.action.clone(),
                    status: "skipped".to_string(),
                    error: None,
                });
            }
            Err(e) => {
                summary.failed += 1;
                report_items.push(MigrationReportItem {
                    resource_name: pack_res.map(|r| r.name.clone()).unwrap_or_default(),
                    action: item.action.clone(),
                    status: "failed".to_string(),
                    error: Some(e),
                });
            }
        }
    }

    let status = if summary.failed == 0 {
        "success"
    } else if summary.added + summary.overwritten > 0 {
        "partial"
    } else {
        "failed"
    };

    Ok(MigrationReport {
        status: status.to_string(),
        items: report_items,
        summary,
    })
}

enum ExecResult {
    Copied,
    Skipped,
}

fn copy_resource(src: &Path, dst: &Path) -> Result<ExecResult, String> {
    if !src.exists() {
        return Err(format!("Source not found: {}", src.display()));
    }

    if src.is_dir() {
        copy_dir_recursive(src, dst)
            .map_err(|e| format!("Failed to copy directory {}: {}", src.display(), e))?;
    } else {
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create parent dir: {}", e))?;
        }
        fs::copy(src, dst)
            .map_err(|e| format!("Failed to copy file {}: {}", src.display(), e))?;
    }

    Ok(ExecResult::Copied)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exec_result_values() {
        // Verify the ExecResult enum compiles correctly
        let _ = ExecResult::Copied;
        let _ = ExecResult::Skipped;
    }
}
