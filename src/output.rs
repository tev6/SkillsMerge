use crate::ir::{MergeResult, OutputFormat};

/// Generate output in the specified format
pub fn generate(result: &MergeResult, format: OutputFormat) -> String {
    match format {
        OutputFormat::Markdown => generate_markdown(result),
        OutputFormat::Json => generate_json(result),
        OutputFormat::Yaml => generate_yaml(result),
        OutputFormat::Toml => generate_toml(result),
    }
}

/// Generate enhanced Markdown output
fn generate_markdown(result: &MergeResult) -> String {
    let mut output = String::new();

    // Front matter
    output.push_str("---\n");
    output.push_str("generated_by: SkillsMerge\n");
    output.push_str("version: 1.0.0\n");
    output.push_str(&format!(
        "source_files: {}\n",
        result.statistics.total_skills
    ));
    output.push_str(&format!(
        "merge_date: \"{}\"\n",
        chrono::Utc::now().format("%Y-%m-%d")
    ));
    output.push_str(&format!(
        "conflict_count: {}\n",
        result.conflicts_resolved.len() + result.conflicts_unresolved.len()
    ));
    output.push_str("---\n\n");

    // Title
    output.push_str("# Merged Skill Collection\n\n");

    // Metadata section
    output.push_str("## Metadata\n");
    output.push_str(&format!(
        "- Original Skills: {}\n",
        result.merged_skill.description.as_deref().unwrap_or("N/A")
    ));
    output.push_str(&format!(
        "- Total Instructions: {}\n",
        result.statistics.total_instructions
    ));
    output.push_str(&format!(
        "- Merge Duration: {}ms\n",
        result.statistics.merge_duration_ms
    ));
    output.push_str(&format!(
        "- Resolution: {}\n\n",
        if result.conflicts_unresolved.is_empty() {
            "all conflicts resolved"
        } else {
            "unresolved conflicts remain"
        }
    ));

    // Conflicts section
    if !result.conflicts_resolved.is_empty() || !result.conflicts_unresolved.is_empty() {
        output.push_str("## Conflicts\n\n");

        for (i, conflict) in result.conflicts_resolved.iter().enumerate() {
            output.push_str(&format!(
                "### Conflict #{}: {}\n",
                i + 1,
                conflict.conflict_type
            ));
            output.push_str(&format!(
                "> **Source**: {} vs {}\n",
                conflict.instruction_a.skill_name, conflict.instruction_b.skill_name
            ));
            if let Some(resolution) = &conflict.suggested_resolution {
                output.push_str(&format!(
                    "> **Resolution**: {:?} ({})\n",
                    resolution.selected, resolution.strategy
                ));
                output.push_str(&format!("> **Rationale**: {}\n", resolution.rationale));
            }
            output.push('\n');
        }

        for (i, conflict) in result.conflicts_unresolved.iter().enumerate() {
            output.push_str(&format!(
                "### Unresolved Conflict #{}: {}\n",
                i + 1,
                conflict.conflict_type
            ));
            output.push_str(&format!(
                "> **Source**: {} vs {}\n",
                conflict.instruction_a.skill_name, conflict.instruction_b.skill_name
            ));
            output.push_str(&format!("> **Severity**: {}\n", conflict.severity));
            output.push_str(&format!("> **Description**: {}\n", conflict.description));
            output.push('\n');
        }
    }

    // Instructions section
    output.push_str("## Instructions\n\n");
    let mut current_category: Option<&str> = None;
    for instr in &result.merged_skill.instructions {
        if instr.category.as_deref() != current_category {
            current_category = instr.category.as_deref();
            if let Some(cat) = current_category {
                output.push_str(&format!("### Category: {}\n\n", cat));
            }
        }
        output.push_str(&format!(
            "- **{}** (priority: {}): {}\n",
            instr.command, instr.priority, instr.content
        ));
    }

    // Warnings
    if !result.warnings.is_empty() {
        output.push_str("\n## Warnings\n\n");
        for warning in &result.warnings {
            output.push_str(&format!("- {}\n", warning));
        }
    }

    output
}

fn generate_json(result: &MergeResult) -> String {
    serde_json::to_string_pretty(result).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e))
}

fn generate_yaml(result: &MergeResult) -> String {
    serde_yaml::to_string(result).unwrap_or_else(|e| format!("error: {}", e))
}

fn generate_toml(result: &MergeResult) -> String {
    // TOML doesn't support all nested structures well, use simplified output
    let mut output = String::new();
    output.push_str("[metadata]\n");
    output.push_str(&format!(
        "total_skills = {}\n",
        result.statistics.total_skills
    ));
    output.push_str(&format!(
        "total_instructions = {}\n",
        result.statistics.total_instructions
    ));
    output.push_str(&format!(
        "conflicts_found = {}\n",
        result.statistics.conflicts_found
    ));
    output.push_str(&format!(
        "conflicts_resolved = {}\n",
        result.conflicts_resolved.len()
    ));
    output.push('\n');

    output.push_str("[[instructions]]\n");
    for instr in &result.merged_skill.instructions {
        output.push_str(&format!("command = \"{}\"\n", instr.command));
        output.push_str(&format!(
            "content = \"{}\"\n",
            instr.content.replace('"', "\\\"")
        ));
        output.push_str(&format!("priority = {}\n", instr.priority));
        if let Some(cat) = &instr.category {
            output.push_str(&format!("category = \"{}\"\n", cat));
        }
        output.push_str("\n[[instructions]]\n");
    }

    output
}

/// Write output to file
pub fn write_to_file(content: &str, path: &std::path::Path) -> crate::error::Result<()> {
    std::fs::write(path, content).map_err(crate::error::SkillsMergeError::IoError)
}
