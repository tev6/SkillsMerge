use std::path::PathBuf;

use skillsmerge::parser;
use skillsmerge::conflict;
use skillsmerge::merger;
use skillsmerge::output;
use skillsmerge::ir::{MergeStrategy, OutputFormat, InputFormat};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/skills")
}

#[test]
fn test_parse_markdown() {
    let path = fixtures_dir().join("skill-a.md");
    let skill = parser::parse_file(&path).expect("Failed to parse markdown");
    assert_eq!(skill.name, "Skill A");
    assert!(skill.description.is_some());
    assert!(!skill.instructions.is_empty());
    assert_eq!(skill.source.format, InputFormat::Markdown);
}

#[test]
fn test_parse_json() {
    let path = fixtures_dir().join("skill-c.json");
    let skill = parser::parse_file(&path).expect("Failed to parse JSON");
    assert_eq!(skill.name, "skill-c");
    assert!(skill.description.is_some());
    assert!(!skill.instructions.is_empty());
    assert_eq!(skill.source.format, InputFormat::Json);
}

#[test]
fn test_parse_yaml() {
    let path = fixtures_dir().join("skill-d.yaml");
    let skill = parser::parse_file(&path).expect("Failed to parse YAML");
    assert_eq!(skill.name, "skill-d");
    assert!(skill.description.is_some());
    assert!(!skill.instructions.is_empty());
    assert_eq!(skill.source.format, InputFormat::Yaml);
}

#[test]
fn test_parse_toml() {
    let path = fixtures_dir().join("skill-e.toml");
    let skill = parser::parse_file(&path).expect("Failed to parse TOML");
    assert_eq!(skill.name, "skill-e");
    assert!(skill.description.is_some());
    assert!(!skill.instructions.is_empty());
    assert_eq!(skill.source.format, InputFormat::Toml);
}

#[test]
fn test_detect_format() {
    assert_eq!(parser::detect_format(PathBuf::from("test.md").as_path()), InputFormat::Markdown);
    assert_eq!(parser::detect_format(PathBuf::from("test.json").as_path()), InputFormat::Json);
    assert_eq!(parser::detect_format(PathBuf::from("test.yaml").as_path()), InputFormat::Yaml);
    assert_eq!(parser::detect_format(PathBuf::from("test.yml").as_path()), InputFormat::Yaml);
    assert_eq!(parser::detect_format(PathBuf::from("test.toml").as_path()), InputFormat::Toml);
    assert_eq!(parser::detect_format(PathBuf::from("test.txt").as_path()), InputFormat::Markdown);
}

#[test]
fn test_conflict_detection() {
    let skill_a = parser::parse_file(&fixtures_dir().join("skill-a.md")).unwrap();
    let skill_b = parser::parse_file(&fixtures_dir().join("skill-b.md")).unwrap();

    let conflicts = conflict::detect_conflicts(&[skill_a, skill_b]);
    // skill-a and skill-b have conflicting instructions (same commands, different content)
    assert!(!conflicts.is_empty(), "Should detect conflicts between skill-a and skill-b");
}

#[test]
fn test_no_conflicts_compatible_skills() {
    let skill_c = parser::parse_file(&fixtures_dir().join("skill-c.json")).unwrap();
    let skill_d = parser::parse_file(&fixtures_dir().join("skill-d.yaml")).unwrap();

    let conflicts = conflict::detect_conflicts(&[skill_c, skill_d]);
    assert!(conflicts.is_empty(), "Compatible skills should have no conflicts");
}

#[test]
fn test_merge_auto_select() {
    let skill_a = parser::parse_file(&fixtures_dir().join("skill-a.md")).unwrap();
    let skill_b = parser::parse_file(&fixtures_dir().join("skill-b.md")).unwrap();

    let result = merger::merge(vec![skill_a, skill_b], &MergeStrategy::AutoSelect);
    assert!(!result.merged_skill.instructions.is_empty());
    assert!(result.statistics.conflicts_found > 0);
}

#[test]
fn test_merge_preserve_all() {
    let skill_a = parser::parse_file(&fixtures_dir().join("skill-a.md")).unwrap();
    let skill_b = parser::parse_file(&fixtures_dir().join("skill-b.md")).unwrap();

    let result = merger::merge(vec![skill_a, skill_b], &MergeStrategy::PreserveAll);
    assert!(!result.merged_skill.instructions.is_empty());
    assert!(!result.conflicts_unresolved.is_empty());
}

#[test]
fn test_merge_semantic() {
    let skill_a = parser::parse_file(&fixtures_dir().join("skill-a.md")).unwrap();
    let skill_b = parser::parse_file(&fixtures_dir().join("skill-b.md")).unwrap();

    let result = merger::merge(vec![skill_a, skill_b], &MergeStrategy::SemanticMerge);
    assert!(!result.merged_skill.instructions.is_empty());
    assert!(result.conflicts_resolved.len() > 0);
}

#[test]
fn test_output_markdown() {
    let skill_c = parser::parse_file(&fixtures_dir().join("skill-c.json")).unwrap();
    let skill_d = parser::parse_file(&fixtures_dir().join("skill-d.yaml")).unwrap();

    let result = merger::merge(vec![skill_c, skill_d], &MergeStrategy::AutoSelect);
    let content = output::generate(&result, OutputFormat::Markdown);
    assert!(content.contains("# Merged Skill Collection"));
    assert!(content.contains("## Metadata"));
    assert!(content.contains("## Instructions"));
}

#[test]
fn test_output_json() {
    let skill_c = parser::parse_file(&fixtures_dir().join("skill-c.json")).unwrap();
    let skill_d = parser::parse_file(&fixtures_dir().join("skill-d.yaml")).unwrap();

    let result = merger::merge(vec![skill_c, skill_d], &MergeStrategy::AutoSelect);
    let content = output::generate(&result, OutputFormat::Json);
    assert!(content.contains("merged_skill"));
}

#[test]
fn test_merge_all_formats() {
    let skill_a = parser::parse_file(&fixtures_dir().join("skill-a.md")).unwrap();
    let skill_b = parser::parse_file(&fixtures_dir().join("skill-b.md")).unwrap();
    let skill_c = parser::parse_file(&fixtures_dir().join("skill-c.json")).unwrap();
    let skill_d = parser::parse_file(&fixtures_dir().join("skill-d.yaml")).unwrap();
    let skill_e = parser::parse_file(&fixtures_dir().join("skill-e.toml")).unwrap();

    let result = merger::merge(
        vec![skill_a, skill_b, skill_c, skill_d, skill_e],
        &MergeStrategy::AutoSelect,
    );

    assert_eq!(result.statistics.total_skills, 5);
    assert!(!result.merged_skill.instructions.is_empty());
}
