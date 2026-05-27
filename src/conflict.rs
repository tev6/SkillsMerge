use std::collections::HashMap;

use crate::ir::{
    Conflict, ConflictSource, ConflictType, Instruction, InstructionRef, Resolution,
    ResolutionChoice, Severity, SkillIR,
};

/// Detect conflicts among a list of skills
pub fn detect_conflicts(skills: &[SkillIR]) -> Vec<Conflict> {
    let mut conflicts = Vec::new();

    // Step 1: Build command index (command -> [(skill_idx, instruction_idx)])
    let mut command_index: HashMap<String, Vec<(usize, usize)>> = HashMap::new();
    for (skill_idx, skill) in skills.iter().enumerate() {
        for (instr_idx, instr) in skill.instructions.iter().enumerate() {
            command_index
                .entry(instr.command.clone())
                .or_default()
                .push((skill_idx, instr_idx));
        }
    }

    // Step 2: Find duplicate commands (quick filter)
    for (command, entries) in &command_index {
        if entries.len() < 2 {
            continue;
        }

        // Step 3: Deep analysis for each pair
        for i in 0..entries.len() {
            for j in (i + 1)..entries.len() {
                let (si_a, ii_a) = entries[i];
                let (si_b, ii_b) = entries[j];
                let skill_a = &skills[si_a];
                let skill_b = &skills[si_b];
                let instr_a = &skill_a.instructions[ii_a];
                let instr_b = &skill_b.instructions[ii_b];

                // Check for instruction override (same command, different content)
                if instr_a.content != instr_b.content {
                    let conflict_type = classify_conflict(instr_a, instr_b);
                    let severity = score_conflict(instr_a, instr_b, &conflict_type);

                    let mut conflict = Conflict::new(
                        conflict_type,
                        severity,
                        make_instruction_ref(skill_a, instr_a),
                        make_instruction_ref(skill_b, instr_b),
                        format!(
                            "Command '{}' defined differently in '{}' and '{}'",
                            command, skill_a.name, skill_b.name
                        ),
                    );

                    conflict.sources = vec![
                        ConflictSource {
                            skill_id: skill_a.id,
                            skill_name: skill_a.name.clone(),
                            instruction_id: instr_a.id,
                        },
                        ConflictSource {
                            skill_id: skill_b.id,
                            skill_name: skill_b.name.clone(),
                            instruction_id: instr_b.id,
                        },
                    ];

                    // Suggest resolution based on priority
                    conflict.suggested_resolution = suggest_resolution(instr_a, instr_b, skill_a, skill_b);

                    conflicts.push(conflict);
                }
            }
        }
    }

    // Step 4: Check for circular dependencies (if skills reference each other)
    let circular = detect_circular_dependencies(skills);
    conflicts.extend(circular);

    // Step 5: Sort by severity (descending)
    conflicts.sort_by(|a, b| b.severity.cmp(&a.severity));
    conflicts
}

/// Classify the type of conflict between two instructions
fn classify_conflict(a: &Instruction, b: &Instruction) -> ConflictType {
    // Check parameter incompatibility
    if !a.parameters.is_empty() && !b.parameters.is_empty() {
        for pa in &a.parameters {
            for pb in &b.parameters {
                if pa.name == pb.name && pa.value != pb.value {
                    return ConflictType::ParameterIncompatible;
                }
            }
        }
    }

    // Check priority conflict
    if a.priority != b.priority && a.priority != 0 && b.priority != 0 {
        return ConflictType::PriorityConflict;
    }

    // Default to instruction override
    ConflictType::InstructionOverride
}

/// Score conflict severity
fn score_conflict(a: &Instruction, b: &Instruction, conflict_type: &ConflictType) -> Severity {
    match conflict_type {
        ConflictType::InstructionOverride => {
            // Same command with completely different content is critical
            if a.command == b.command && !a.content.is_empty() && !b.content.is_empty() {
                Severity::High
            } else {
                Severity::Medium
            }
        }
        ConflictType::PriorityConflict => Severity::Medium,
        ConflictType::ParameterIncompatible => Severity::High,
        ConflictType::SemanticConflict => Severity::Critical,
        ConflictType::CircularDependency => Severity::Critical,
    }
}

/// Suggest a resolution for a conflict
fn suggest_resolution(
    instr_a: &Instruction,
    instr_b: &Instruction,
    skill_a: &SkillIR,
    skill_b: &SkillIR,
) -> Option<Resolution> {
    // Prefer higher priority
    match instr_a.priority.cmp(&instr_b.priority) {
        std::cmp::Ordering::Greater => Some(Resolution {
            strategy: "auto-select".to_string(),
            selected: ResolutionChoice::UseA,
            rationale: format!(
                "Instruction in '{}' has higher priority ({}) than '{}' ({})",
                skill_a.name, instr_a.priority, skill_b.name, instr_b.priority
            ),
        }),
        std::cmp::Ordering::Less => Some(Resolution {
            strategy: "auto-select".to_string(),
            selected: ResolutionChoice::UseB,
            rationale: format!(
                "Instruction in '{}' has higher priority ({}) than '{}' ({})",
                skill_b.name, instr_b.priority, skill_a.name, instr_a.priority
            ),
        }),
        std::cmp::Ordering::Equal => None, // No automatic resolution possible
    }
}

fn make_instruction_ref(skill: &SkillIR, instr: &Instruction) -> InstructionRef {
    InstructionRef {
        skill_name: skill.name.clone(),
        instruction_id: instr.id,
        command: instr.command.clone(),
        content: instr.content.clone(),
    }
}

/// Detect circular dependencies among skills
fn detect_circular_dependencies(skills: &[SkillIR]) -> Vec<Conflict> {
    // Build dependency graph from config "depends_on" field
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();
    let mut name_map: HashMap<String, usize> = HashMap::new();

    for (idx, skill) in skills.iter().enumerate() {
        name_map.insert(skill.name.clone(), idx);
        if let Some(serde_json::Value::Array(deps)) = skill.config.get("depends_on") {
            let dep_names: Vec<String> = deps
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            graph.insert(skill.name.clone(), dep_names);
        } else {
            graph.insert(skill.name.clone(), Vec::new());
        }
    }

    // DFS cycle detection
    let mut visited = vec![false; skills.len()];
    let mut in_stack = vec![false; skills.len()];
    let mut conflicts = Vec::new();

    for i in 0..skills.len() {
        if !visited[i] {
            let mut path = Vec::new();
            dfs_cycle(i, &graph, &name_map, &mut visited, &mut in_stack, &mut path, &mut conflicts, skills);
        }
    }

    conflicts
}

fn dfs_cycle(
    node: usize,
    graph: &HashMap<String, Vec<String>>,
    name_map: &HashMap<String, usize>,
    visited: &mut Vec<bool>,
    in_stack: &mut Vec<bool>,
    path: &mut Vec<usize>,
    conflicts: &mut Vec<Conflict>,
    skills: &[SkillIR],
) {
    visited[node] = true;
    in_stack[node] = true;
    path.push(node);

    let name = &skills[node].name;
    if let Some(deps) = graph.get(name) {
        for dep in deps {
            if let Some(&dep_idx) = name_map.get(dep) {
                if in_stack[dep_idx] {
                    // Found cycle
                    let cycle_start = path.iter().position(|&x| x == dep_idx).unwrap_or(0);
                    let cycle_path: Vec<String> = path[cycle_start..]
                        .iter()
                        .map(|&idx| skills[idx].name.clone())
                        .collect();

                    let conflict = Conflict::new(
                        ConflictType::CircularDependency,
                        Severity::Critical,
                        make_instruction_ref(&skills[node], &skills[node].instructions.first().cloned().unwrap_or_else(|| crate::ir::Instruction::new("N/A".to_string(), "N/A".to_string()))),
                        make_instruction_ref(&skills[dep_idx], &skills[dep_idx].instructions.first().cloned().unwrap_or_else(|| crate::ir::Instruction::new("N/A".to_string(), "N/A".to_string()))),
                        format!("Circular dependency detected: {}", cycle_path.join(" -> ")),
                    );
                    conflicts.push(conflict);
                } else if !visited[dep_idx] {
                    dfs_cycle(dep_idx, graph, name_map, visited, in_stack, path, conflicts, skills);
                }
            }
        }
    }

    path.pop();
    in_stack[node] = false;
}
