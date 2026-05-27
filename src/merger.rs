use std::collections::HashMap;

use crate::conflict;
use crate::ir::{
    Conflict, Instruction, InstructionRef, MergeResult, MergeStatistics, MergeStrategy, Resolution,
    ResolutionChoice, SkillIR,
};

/// Merge multiple skills using the specified strategy
pub fn merge(skills: Vec<SkillIR>, strategy: &MergeStrategy) -> MergeResult {
    let all_conflicts = conflict::detect_conflicts(&skills);
    merge_with_conflicts(skills, all_conflicts, strategy)
}

/// Merge skills using pre-resolved conflicts (from interactive mode)
pub fn merge_with_conflicts(
    skills: Vec<SkillIR>,
    all_conflicts: Vec<Conflict>,
    strategy: &MergeStrategy,
) -> MergeResult {
    let start = std::time::Instant::now();
    let total_skills = skills.len();
    let total_instructions: usize = skills.iter().map(|s| s.instructions.len()).sum();
    let conflicts_found = all_conflicts.len();

    // Resolve conflicts based on strategy
    let (resolved, unresolved) = resolve_conflicts(all_conflicts, strategy, &skills);

    // Build merged skill
    let merged_instructions = build_merged_instructions(&skills, &resolved, strategy);
    let merged_config = merge_configs(&skills);
    let merged_metadata = merge_metadata(&skills);

    let mut merged_skill = SkillIR::new(
        format!(
            "Merged-{}",
            skills.first().map(|s| s.name.clone()).unwrap_or_default()
        ),
        std::path::PathBuf::from("merged"),
        skills
            .first()
            .map(|s| s.source.format)
            .unwrap_or(crate::ir::InputFormat::Markdown),
    );
    merged_skill.description = Some(format!(
        "Merged skill from: {}",
        skills
            .iter()
            .map(|s| s.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    ));
    merged_skill.instructions = merged_instructions;
    merged_skill.config = merged_config;
    merged_skill.metadata = merged_metadata;

    let warnings = generate_warnings(&unresolved);
    let conflicts_resolved = resolved.len();

    MergeResult {
        merged_skill,
        conflicts_resolved: resolved,
        conflicts_unresolved: unresolved,
        warnings,
        statistics: MergeStatistics {
            total_skills,
            total_instructions,
            conflicts_found,
            conflicts_resolved,
            merge_duration_ms: start.elapsed().as_millis() as u64,
        },
    }
}

fn resolve_conflicts(
    conflicts: Vec<Conflict>,
    strategy: &MergeStrategy,
    skills: &[SkillIR],
) -> (Vec<Conflict>, Vec<Conflict>) {
    let mut resolved = Vec::new();
    let mut unresolved = Vec::new();

    for mut conflict in conflicts {
        match strategy {
            MergeStrategy::PreserveAll => {
                // Keep all, mark as unresolved
                unresolved.push(conflict);
            }
            MergeStrategy::AutoSelect => {
                // Try auto-resolution
                if let Some(suggestion) = &conflict.suggested_resolution {
                    conflict.suggested_resolution = Some(suggestion.clone());
                    resolved.push(conflict);
                } else {
                    // No automatic resolution, prefer higher priority skill
                    let pri_a = get_instruction_priority(skills, &conflict.instruction_a);
                    let pri_b = get_instruction_priority(skills, &conflict.instruction_b);
                    if pri_a != pri_b {
                        conflict.suggested_resolution = Some(Resolution {
                            strategy: "auto-select".to_string(),
                            selected: if pri_a > pri_b {
                                ResolutionChoice::UseA
                            } else {
                                ResolutionChoice::UseB
                            },
                            rationale: format!(
                                "Selected by skill priority ({} vs {})",
                                pri_a, pri_b
                            ),
                            custom_content: None,
                        });
                        resolved.push(conflict);
                    } else {
                        unresolved.push(conflict);
                    }
                }
            }
            MergeStrategy::Interactive => {
                // Interactive mode: all conflicts are unresolved initially
                unresolved.push(conflict);
            }
            MergeStrategy::SemanticMerge => {
                // Semantic merge: try to combine
                conflict.suggested_resolution = Some(Resolution {
                    strategy: "semantic-merge".to_string(),
                    selected: ResolutionChoice::Merge,
                    rationale: "Combined both instructions".to_string(),
                    custom_content: None,
                });
                resolved.push(conflict);
            }
            MergeStrategy::Custom(rule) => {
                // Custom strategy based on rule name
                match rule.as_str() {
                    "latest" => {
                        conflict.suggested_resolution = Some(Resolution {
                            strategy: "custom-latest".to_string(),
                            selected: ResolutionChoice::UseB,
                            rationale: "Using latest definition".to_string(),
                            custom_content: None,
                        });
                        resolved.push(conflict);
                    }
                    _ => {
                        unresolved.push(conflict);
                    }
                }
            }
        }
    }

    (resolved, unresolved)
}

fn get_instruction_priority(skills: &[SkillIR], instr_ref: &InstructionRef) -> i32 {
    skills
        .iter()
        .find(|s| s.name == instr_ref.skill_name)
        .and_then(|s| s.metadata.priority)
        .unwrap_or(0)
}

fn build_merged_instructions(
    skills: &[SkillIR],
    resolved: &[Conflict],
    _strategy: &MergeStrategy,
) -> Vec<Instruction> {
    let mut instructions: Vec<Instruction> = Vec::new();
    let mut excluded_ids: Vec<uuid::Uuid> = Vec::new();

    // Determine which instructions to exclude based on resolved conflicts
    for conflict in resolved {
        if let Some(resolution) = &conflict.suggested_resolution {
            match resolution.selected {
                ResolutionChoice::UseA => {
                    excluded_ids.push(conflict.instruction_b.instruction_id);
                }
                ResolutionChoice::UseB => {
                    excluded_ids.push(conflict.instruction_a.instruction_id);
                }
                ResolutionChoice::Skip => {
                    excluded_ids.push(conflict.instruction_a.instruction_id);
                    excluded_ids.push(conflict.instruction_b.instruction_id);
                }
                ResolutionChoice::Merge => {
                    // If there's custom content, exclude both originals
                    // (custom text replaces both). Otherwise keep both
                    // (they will be added alongside the combined version).
                    if resolution.custom_content.is_some() {
                        excluded_ids.push(conflict.instruction_a.instruction_id);
                        excluded_ids.push(conflict.instruction_b.instruction_id);
                    }
                }
            }
        }
    }

    // Collect all instructions, excluding resolved-against ones
    for skill in skills {
        for instr in &skill.instructions {
            if !excluded_ids.contains(&instr.id) {
                instructions.push(instr.clone());
            }
        }
    }

    // For merged conflicts, add combined instructions
    for conflict in resolved {
        if let Some(resolution) = &conflict.suggested_resolution {
            if resolution.selected == ResolutionChoice::Merge {
                let content = if let Some(ref custom) = resolution.custom_content {
                    custom.clone()
                } else {
                    format!(
                        "{}\n{}",
                        conflict.instruction_a.content, conflict.instruction_b.content
                    )
                };
                let combined = Instruction::new(conflict.instruction_a.command.clone(), content);
                instructions.push(combined);
            }
        }
    }

    // Sort by priority (descending), then by category
    instructions.sort_by(|a, b| {
        b.priority
            .cmp(&a.priority)
            .then_with(|| a.category.cmp(&b.category))
    });

    instructions
}

fn merge_configs(skills: &[SkillIR]) -> HashMap<String, serde_json::Value> {
    let mut config = HashMap::new();

    // Merge configs from all skills, later skills override earlier ones
    for skill in skills {
        for (k, v) in &skill.config {
            config.insert(k.clone(), v.clone());
        }
    }

    config
}

fn merge_metadata(skills: &[SkillIR]) -> crate::ir::SkillMetadata {
    let mut metadata = crate::ir::SkillMetadata::default();
    metadata.tags = skills
        .iter()
        .flat_map(|s| s.metadata.tags.clone())
        .collect();
    metadata.tags.sort();
    metadata.tags.dedup();
    metadata
}

fn generate_warnings(unresolved: &[Conflict]) -> Vec<String> {
    unresolved
        .iter()
        .map(|c| format!("[{}] {}", c.severity, c.description))
        .collect()
}
