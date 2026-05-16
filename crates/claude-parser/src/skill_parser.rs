use std::fs;
use std::path::Path;

use domain::paths::{CLAUDE_DIR, SKILLS_DIR};
use domain::CapabilityResource;
use sha2::{Digest, Sha256};

pub fn parse_skills(repo_path: &str) -> Vec<CapabilityResource> {
    let skills_dir = Path::new(repo_path).join(CLAUDE_DIR).join(SKILLS_DIR);
    if !skills_dir.exists() || !skills_dir.is_dir() {
        return Vec::new();
    }

    let mut resources = Vec::new();
    let entries = match fs::read_dir(&skills_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let skill_name = path.file_name().unwrap().to_string_lossy().to_string();

        let skill_md = path.join("SKILL.md");
        let skill_md_lower = path.join("skill.md");

        let md_path = if skill_md.exists() {
            skill_md
        } else if skill_md_lower.exists() {
            skill_md_lower
        } else {
            continue;
        };

        let content = match fs::read_to_string(&md_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let hash = format!("{:x}", Sha256::digest(content.as_bytes()));
        let summary = extract_summary(&content);

        let metadata = serde_json::json!({"summary": summary});

        let relative_source =
            format!("{}/{}/{}/{}", CLAUDE_DIR, SKILLS_DIR, skill_name, md_path.file_name().unwrap().to_string_lossy());

        resources.push(CapabilityResource {
            id: uuid::Uuid::new_v4().to_string(),
            repo_id: None,
            pack_id: None,
            r#type: "skill".to_string(),
            name: skill_name,
            source_path: Some(relative_source),
            scope: "project".to_string(),
            tracked_by_git: true,
            content_hash: Some(hash),
            metadata_json: Some(metadata.to_string()),
            error_message: None,
        });
    }

    resources
}

fn extract_summary(content: &str) -> String {
    let mut past_heading = false;
    let mut blank_after_heading = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with('#') {
            past_heading = true;
            blank_after_heading = false;
            continue;
        }

        if trimmed.is_empty() {
            if past_heading {
                blank_after_heading = true;
            }
            continue;
        }

        // First non-empty, non-heading line after the heading block is the summary
        if past_heading && blank_after_heading {
            return trimmed.to_string();
        }

        // No heading at all: first text line is the summary
        if !past_heading {
            return trimmed.to_string();
        }

        // Non-empty line immediately after heading (no blank line separator):
        // this is the summary
        if past_heading && !blank_after_heading {
            return trimmed.to_string();
        }
    }

    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_summary_from_skill_md() {
        let content = "# My Skill\n\nThis is the summary.\n\nMore details.";
        assert_eq!(extract_summary(content), "This is the summary.");
    }

    #[test]
    fn extract_summary_no_description() {
        let content = "# Just a Title";
        assert_eq!(extract_summary(content), "");
    }

    #[test]
    fn extract_summary_no_heading_first_line() {
        let content = "Direct summary line.\n\nMore stuff.";
        assert_eq!(extract_summary(content), "Direct summary line.");
    }
}
