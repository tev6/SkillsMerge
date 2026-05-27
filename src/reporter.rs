use crate::ir::{Conflict, MergeResult, Severity};

/// Generate a conflict report
pub fn generate_conflict_report(conflicts: &[Conflict]) -> String {
    let mut report = String::new();

    report.push_str("# Conflict Report\n\n");
    report.push_str(&format!("Total conflicts found: {}\n\n", conflicts.len()));

    // Summary by severity
    report.push_str("## Summary by Severity\n\n");
    for severity in &[Severity::Critical, Severity::High, Severity::Medium, Severity::Low, Severity::Info] {
        let count = conflicts.iter().filter(|c| &c.severity == severity).count();
        if count > 0 {
            report.push_str(&format!("- **{}**: {}\n", severity, count));
        }
    }
    report.push_str("\n");

    // Summary by type
    report.push_str("## Summary by Type\n\n");
    use crate::ir::ConflictType;
    for ct in &[
        ConflictType::InstructionOverride,
        ConflictType::PriorityConflict,
        ConflictType::ParameterIncompatible,
        ConflictType::SemanticConflict,
        ConflictType::CircularDependency,
    ] {
        let count = conflicts.iter().filter(|c| &c.conflict_type == ct).count();
        if count > 0 {
            report.push_str(&format!("- **{}**: {}\n", ct, count));
        }
    }
    report.push_str("\n");

    // Detailed conflicts
    report.push_str("## Detailed Conflicts\n\n");
    for (i, conflict) in conflicts.iter().enumerate() {
        report.push_str(&format!("### Conflict #{}\n", i + 1));
        report.push_str(&format!("- **Type**: {}\n", conflict.conflict_type));
        report.push_str(&format!("- **Severity**: {}\n", conflict.severity));
        report.push_str(&format!("- **Source A**: {} - `{}`\n", conflict.instruction_a.skill_name, conflict.instruction_a.command));
        report.push_str(&format!("- **Source B**: {} - `{}`\n", conflict.instruction_b.skill_name, conflict.instruction_b.command));
        report.push_str(&format!("- **Description**: {}\n", conflict.description));
        if let Some(resolution) = &conflict.suggested_resolution {
            report.push_str(&format!("- **Suggested Resolution**: {:?} - {}\n", resolution.selected, resolution.rationale));
        }
        report.push_str("\n");
    }

    report
}

/// Generate a merge summary report
pub fn generate_merge_summary(result: &MergeResult) -> String {
    let mut report = String::new();

    report.push_str("# Merge Summary\n\n");
    report.push_str(&format!("Skills merged: {}\n", result.statistics.total_skills));
    report.push_str(&format!("Total instructions: {}\n", result.statistics.total_instructions));
    report.push_str(&format!("Conflicts found: {}\n", result.statistics.conflicts_found));
    report.push_str(&format!("Conflicts resolved: {}\n", result.conflicts_resolved.len()));
    report.push_str(&format!("Conflicts unresolved: {}\n", result.conflicts_unresolved.len()));
    report.push_str(&format!("Merge duration: {}ms\n", result.statistics.merge_duration_ms));

    if !result.warnings.is_empty() {
        report.push_str("\n## Warnings\n\n");
        for warning in &result.warnings {
            report.push_str(&format!("- {}\n", warning));
        }
    }

    report
}
