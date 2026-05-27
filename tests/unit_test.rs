use skillsmerge::ir::*;
use skillsmerge::reporter;
use skillsmerge::config;
use skillsmerge::io;

#[test]
fn test_skill_ir_creation() {
    let skill = SkillIR::new(
        "test-skill".to_string(),
        std::path::PathBuf::from("test.md"),
        InputFormat::Markdown,
    );
    assert_eq!(skill.name, "test-skill");
    assert!(skill.instructions.is_empty());
    assert!(skill.description.is_none());
}

#[test]
fn test_instruction_builder() {
    let instr = Instruction::new("Always".to_string(), "write tests".to_string())
        .with_priority(10)
        .with_category("Testing".to_string());
    assert_eq!(instr.command, "Always");
    assert_eq!(instr.content, "write tests");
    assert_eq!(instr.priority, 10);
    assert_eq!(instr.category.as_deref(), Some("Testing"));
}

#[test]
fn test_severity_ordering() {
    assert!(Severity::Critical > Severity::High);
    assert!(Severity::High > Severity::Medium);
    assert!(Severity::Medium > Severity::Low);
    assert!(Severity::Low > Severity::Info);
}

#[test]
fn test_merge_strategy_from_str() {
    assert_eq!("auto".parse::<MergeStrategy>().unwrap(), MergeStrategy::AutoSelect);
    assert_eq!("preserve-all".parse::<MergeStrategy>().unwrap(), MergeStrategy::PreserveAll);
    assert_eq!("interactive".parse::<MergeStrategy>().unwrap(), MergeStrategy::Interactive);
    assert_eq!("semantic".parse::<MergeStrategy>().unwrap(), MergeStrategy::SemanticMerge);
}

#[test]
fn test_output_format_from_str() {
    assert_eq!("markdown".parse::<OutputFormat>().unwrap(), OutputFormat::Markdown);
    assert_eq!("json".parse::<OutputFormat>().unwrap(), OutputFormat::Json);
    assert_eq!("yaml".parse::<OutputFormat>().unwrap(), OutputFormat::Yaml);
    assert_eq!("toml".parse::<OutputFormat>().unwrap(), OutputFormat::Toml);
}

#[test]
fn test_conflict_creation() {
    let instr_ref_a = InstructionRef {
        skill_name: "skill-a".to_string(),
        instruction_id: uuid::Uuid::new_v4(),
        command: "Always".to_string(),
        content: "use tabs".to_string(),
    };
    let instr_ref_b = InstructionRef {
        skill_name: "skill-b".to_string(),
        instruction_id: uuid::Uuid::new_v4(),
        command: "Always".to_string(),
        content: "use spaces".to_string(),
    };

    let conflict = Conflict::new(
        ConflictType::InstructionOverride,
        Severity::High,
        instr_ref_a,
        instr_ref_b,
        "Conflicting indentation rules".to_string(),
    );

    assert_eq!(conflict.conflict_type, ConflictType::InstructionOverride);
    assert_eq!(conflict.severity, Severity::High);
    assert!(conflict.suggested_resolution.is_none());
}

#[test]
fn test_config_default() {
    let config = config::Config::default();
    assert_eq!(config.default_strategy, "ai");
    assert_eq!(config.default_output_format, "markdown");
}

#[test]
fn test_reporter_conflict_report() {
    let instr_ref_a = InstructionRef {
        skill_name: "skill-a".to_string(),
        instruction_id: uuid::Uuid::new_v4(),
        command: "Always".to_string(),
        content: "use tabs".to_string(),
    };
    let instr_ref_b = InstructionRef {
        skill_name: "skill-b".to_string(),
        instruction_id: uuid::Uuid::new_v4(),
        command: "Always".to_string(),
        content: "use spaces".to_string(),
    };

    let conflicts = vec![Conflict::new(
        ConflictType::InstructionOverride,
        Severity::High,
        instr_ref_a,
        instr_ref_b,
        "Conflicting rules".to_string(),
    )];

    let report = reporter::generate_conflict_report(&conflicts);
    assert!(report.contains("# Conflict Report"));
    assert!(report.contains("instruction_override"));
}

#[test]
fn test_io_collect_files() {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/skills");
    let files = io::collect_input_files(&[dir]);
    assert!(!files.is_empty(), "Should find fixture files");
}
