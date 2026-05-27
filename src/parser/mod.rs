pub mod markdown;
pub mod json;
pub mod yaml;
pub mod toml;

use crate::error::{Result, SkillsMergeError};
use crate::ir::{InputFormat, SkillIR};

/// Detect input format from file extension
pub fn detect_format(path: &std::path::Path) -> InputFormat {
    match path.extension().and_then(|e| e.to_str()).map(|e| e.to_lowercase()) {
        Some(e) if e == "json" => InputFormat::Json,
        Some(e) if e == "yaml" || e == "yml" => InputFormat::Yaml,
        Some(e) if e == "toml" => InputFormat::Toml,
        _ => InputFormat::Markdown, // default
    }
}

/// Parse a SKILLS file into SkillIR
pub fn parse_file(path: &std::path::Path) -> Result<SkillIR> {
    if !path.exists() {
        return Err(SkillsMergeError::FileNotFound {
            path: path.display().to_string(),
        });
    }

    let content = std::fs::read_to_string(path).map_err(|e| SkillsMergeError::IoError(e))?;
    let format = detect_format(path);
    parse_content(&content, path, format)
}

/// Parse content string into SkillIR
pub fn parse_content(content: &str, path: &std::path::Path, format: InputFormat) -> Result<SkillIR> {
    match format {
        InputFormat::Markdown => markdown::parse(content, path),
        InputFormat::Json => json::parse(content, path),
        InputFormat::Yaml => yaml::parse(content, path),
        InputFormat::Toml => toml::parse(content, path),
    }
}

/// Parse multiple files
pub fn parse_files(paths: &[std::path::PathBuf]) -> Vec<Result<SkillIR>> {
    paths.iter().map(|p| parse_file(p)).collect()
}
