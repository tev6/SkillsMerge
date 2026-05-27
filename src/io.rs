use std::path::{Path, PathBuf};

use crate::error::{Result, SkillsMergeError};
use crate::ir::SkillIR;
use crate::parser;

/// Collect all SKILLS files from input paths (files or directories)
pub fn collect_input_files(paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut files = Vec::new();

    for path in paths {
        if path.is_file() {
            files.push(path.clone());
        } else if path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    let entry_path = entry.path();
                    if entry_path.is_file() && is_skills_file(&entry_path) {
                        files.push(entry_path);
                    }
                }
            }
        }
    }

    files.sort();
    files
}

/// Check if a file is a supported SKILLS file based on extension
fn is_skills_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()).map(|e| e.to_lowercase()).as_deref(),
        Some("md") | Some("json") | Some("yaml") | Some("yml") | Some("toml")
    )
}

/// Load and parse all SKILLS files from input paths
pub fn load_skills(paths: &[PathBuf]) -> (Vec<SkillIR>, Vec<SkillsMergeError>) {
    let files = collect_input_files(paths);
    let mut skills = Vec::new();
    let mut errors = Vec::new();

    for file_path in files {
        match parser::parse_file(&file_path) {
            Ok(skill) => skills.push(skill),
            Err(e) => errors.push(e),
        }
    }

    (skills, errors)
}

/// Ensure output directory exists
pub fn ensure_output_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(SkillsMergeError::IoError)?;
        }
    }
    Ok(())
}
