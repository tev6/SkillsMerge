use std::collections::{HashMap, HashSet};

use crate::ir::{
    Conflict, ConflictSource, ConflictType, Instruction, InstructionRef, Resolution,
    ResolutionChoice, Severity, SkillIR,
};

/// Minimum Jaccard similarity to flag two instructions as semantically related
const SEMANTIC_SIMILARITY_THRESHOLD: f64 = 0.30;
/// Minimum word count for an instruction to be considered in semantic analysis
const MIN_WORD_COUNT: usize = 3;
/// Domain keywords — if both instructions mention any of these the likelihood of
/// a real semantic conflict is higher.
const DOMAIN_KEYWORDS: &[&str] = &[
    "indent",
    "spaces",
    "tabs",
    "formatting",
    "lint",
    "style",
    "naming",
    "comment",
    "doc",
    "error",
    "handle",
    "log",
    "test",
    "commit",
    "review",
    "deploy",
    "build",
    "deps",
    "import",
    "export",
    "async",
    "sync",
    "api",
    "security",
    "auth",
    "token",
    "secret",
    "config",
    "env",
];

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
                    conflict.suggested_resolution =
                        suggest_resolution(instr_a, instr_b, skill_a, skill_b);

                    conflicts.push(conflict);
                }
            }
        }
    }

    // Step 4: Detect semantic overlaps — instructions with different commands
    // but similar content that likely address the same concern.
    let semantic = detect_semantic_overlaps(skills, &command_index);
    conflicts.extend(semantic);

    // Step 5: Check for circular dependencies (if skills reference each other)
    let circular = detect_circular_dependencies(skills);
    conflicts.extend(circular);

    // Step 6: Sort by severity (descending)
    conflicts.sort_by_key(|b| std::cmp::Reverse(b.severity));
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
            custom_content: None,
        }),
        std::cmp::Ordering::Less => Some(Resolution {
            strategy: "auto-select".to_string(),
            selected: ResolutionChoice::UseB,
            rationale: format!(
                "Instruction in '{}' has higher priority ({}) than '{}' ({})",
                skill_b.name, instr_b.priority, skill_a.name, instr_a.priority
            ),
            custom_content: None,
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
            dfs_cycle(
                i,
                &graph,
                &name_map,
                &mut visited,
                &mut in_stack,
                &mut path,
                &mut conflicts,
                skills,
            );
        }
    }

    conflicts
}

#[allow(clippy::too_many_arguments)]
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
                        make_instruction_ref(
                            &skills[node],
                            &skills[node]
                                .instructions
                                .first()
                                .cloned()
                                .unwrap_or_else(|| {
                                    crate::ir::Instruction::new(
                                        "N/A".to_string(),
                                        "N/A".to_string(),
                                    )
                                }),
                        ),
                        make_instruction_ref(
                            &skills[dep_idx],
                            &skills[dep_idx]
                                .instructions
                                .first()
                                .cloned()
                                .unwrap_or_else(|| {
                                    crate::ir::Instruction::new(
                                        "N/A".to_string(),
                                        "N/A".to_string(),
                                    )
                                }),
                        ),
                        format!("Circular dependency detected: {}", cycle_path.join(" -> ")),
                    );
                    conflicts.push(conflict);
                } else if !visited[dep_idx] {
                    dfs_cycle(
                        dep_idx, graph, name_map, visited, in_stack, path, conflicts, skills,
                    );
                }
            }
        }
    }

    path.pop();
    in_stack[node] = false;
}

// ---------------------------------------------------------------------------
// Semantic overlap detection
// ---------------------------------------------------------------------------

/// Detect instructions from different skills that have *different* command names
/// but semantically similar content (e.g. "Always use spaces" vs "Use tabs for
/// indentation").  These are soft conflicts that the exact-command matcher
/// misses because the command keywords differ.
fn detect_semantic_overlaps(
    skills: &[SkillIR],
    command_index: &HashMap<String, Vec<(usize, usize)>>,
) -> Vec<Conflict> {
    let mut conflicts = Vec::new();

    // Build a quick set of (skill_idx, instr_idx) pairs already covered by
    // exact-command conflicts so we don't double-report.
    let mut covered: HashSet<(usize, usize, usize, usize)> = HashSet::new();
    for entries in command_index.values() {
        if entries.len() < 2 {
            continue;
        }
        for i in 0..entries.len() {
            for j in (i + 1)..entries.len() {
                let a = entries[i];
                let b = entries[j];
                covered.insert((a.0, a.1, b.0, b.1));
                covered.insert((b.0, b.1, a.0, a.1));
            }
        }
    }

    // Cross-compare all instruction pairs across different skills
    for si_a in 0..skills.len() {
        for si_b in (si_a + 1)..skills.len() {
            for (ii_a, instr_a) in skills[si_a].instructions.iter().enumerate() {
                for (ii_b, instr_b) in skills[si_b].instructions.iter().enumerate() {
                    // Already caught by exact-command matching?
                    if covered.contains(&(si_a, ii_a, si_b, ii_b)) {
                        continue;
                    }

                    let tokens_a = tokenize(&instr_a.content);
                    let tokens_b = tokenize(&instr_b.content);

                    // Skip very short instructions — not enough signal
                    if tokens_a.len() < MIN_WORD_COUNT || tokens_b.len() < MIN_WORD_COUNT {
                        continue;
                    }

                    let sim = jaccard(&tokens_a, &tokens_b);
                    if sim >= SEMANTIC_SIMILARITY_THRESHOLD && share_domain(&tokens_a, &tokens_b) {
                        let severity = if sim > 0.6 {
                            Severity::High
                        } else if sim > 0.45 {
                            Severity::Medium
                        } else {
                            Severity::Low
                        };

                        let conflict = Conflict::new(
                            ConflictType::SemanticConflict,
                            severity,
                            make_instruction_ref(&skills[si_a], instr_a),
                            make_instruction_ref(&skills[si_b], instr_b),
                            format!(
                                "Semantically similar instructions (similarity: {:.0}%) from \
                                 '{}' and '{}': \"{}\" vs \"{}\"",
                                sim * 100.0,
                                skills[si_a].name,
                                skills[si_b].name,
                                truncate_str(&instr_a.content, 70),
                                truncate_str(&instr_b.content, 70),
                            ),
                        );
                        conflicts.push(conflict);
                    }
                }
            }
        }
    }

    conflicts
}

/// Tokenize a string into a set of meaningful lowercase words.
fn tokenize(text: &str) -> HashSet<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .map(|w| w.trim())
        .filter(|w| w.len() >= 3)
        .map(|w| w.to_string())
        .collect()
}

/// Jaccard similarity coefficient between two sets.
fn jaccard(a: &HashSet<String>, b: &HashSet<String>) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let intersection = a.intersection(b).count();
    let union = a.union(b).count();
    intersection as f64 / union as f64
}

/// Check whether two token sets share at least one domain keyword,
/// which increases confidence that the similarity is meaningful.
fn share_domain(tokens_a: &HashSet<String>, tokens_b: &HashSet<String>) -> bool {
    for kw in DOMAIN_KEYWORDS {
        let kw = *kw;
        if tokens_a.contains(kw) && tokens_b.contains(kw) {
            return true;
        }
    }
    // Fallback: if there's a substantial intersection anyway, it's likely real
    let common = tokens_a.intersection(tokens_b).count();
    common >= 2
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len])
    }
}
