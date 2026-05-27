use crate::ai::client::LlmClient;
use crate::ai::prompts;
use crate::ai::semantic::detect_semantic_conflicts;
use crate::error::Result;
use crate::ir::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct AiMergeResult {
    merged_skill: AiMergedSkill,
    #[serde(default)]
    resolutions: Vec<AiResolution>,
}

#[derive(Debug, Deserialize)]
struct AiMergedSkill {
    name: String,
    description: String,
    instructions: Vec<AiInstruction>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct AiInstruction {
    command: String,
    content: String,
    category: String,
    priority: i32,
    #[serde(default)]
    rationale: String,
}

#[derive(Debug, Deserialize)]
struct AiResolution {
    conflict: String,
    resolution: String,
    chosen_instruction: String,
    rationale: String,
}

#[derive(Debug, Deserialize)]
pub struct ConflictQuestion {
    pub explanation: String,
    pub question: String,
    pub option_a: ConflictOption,
    pub option_b: ConflictOption,
    pub recommendation: String,
    pub reasoning: String,
}

#[derive(Debug, Deserialize)]
pub struct ConflictOption {
    pub label: String,
    pub description: String,
    pub trade_off: String,
}

/// AI-driven merge: detect conflicts and merge in one step
pub async fn ai_merge(client: &LlmClient, skills: Vec<SkillIR>) -> Result<MergeResult> {
    let start = std::time::Instant::now();
    let total_skills = skills.len();
    let total_instructions: usize = skills.iter().map(|s| s.instructions.len()).sum();

    // Step 1: AI detects semantic conflicts
    println!("AI is analyzing skills for conflicts...");
    let conflicts = detect_semantic_conflicts(client, &skills).await?;
    let conflicts_found = conflicts.len();
    println!("AI found {} conflict(s)", conflicts_found);

    // Step 2: AI merges the skills
    println!("AI is merging skills...");
    let skills_json = serde_json::to_string_pretty(
        &skills
            .iter()
            .map(|s| {
                serde_json::json!({
                    "name": s.name,
                    "description": s.description,
                    "instructions": s.instructions.iter().map(|i| serde_json::json!({
                        "command": i.command,
                        "content": i.content,
                        "category": i.category,
                        "priority": i.priority,
                    })).collect::<Vec<_>>(),
                })
            })
            .collect::<Vec<_>>(),
    )?;

    let conflicts_json = serde_json::to_string_pretty(&conflicts.iter().map(|c| serde_json::json!({
        "type": c.conflict_type.to_string(),
        "severity": c.severity.to_string(),
        "description": c.description,
        "instruction_a": format!("{}: {}", c.instruction_a.skill_name, c.instruction_a.content),
        "instruction_b": format!("{}: {}", c.instruction_b.skill_name, c.instruction_b.content),
    })).collect::<Vec<_>>())?;

    let response = client
        .ask(
            prompts::merge_system(),
            &prompts::merge_prompt(&skills_json, &conflicts_json),
        )
        .await?;

    let ai_result: AiMergeResult = parse_json_response(&response)?;

    // Step 3: Build merged SkillIR from AI result
    let mut merged_instructions = Vec::new();
    for ai_instr in &ai_result.merged_skill.instructions {
        let instr = Instruction::new(ai_instr.command.clone(), ai_instr.content.clone())
            .with_priority(ai_instr.priority)
            .with_category(ai_instr.category.clone());
        merged_instructions.push(instr);
    }

    let mut merged_skill = SkillIR::new(
        ai_result.merged_skill.name.clone(),
        std::path::PathBuf::from("merged"),
        skills
            .first()
            .map(|s| s.source.format)
            .unwrap_or(InputFormat::Markdown),
    );
    merged_skill.description = Some(ai_result.merged_skill.description.clone());
    merged_skill.instructions = merged_instructions;
    merged_skill.metadata = merge_metadata(&skills);

    // Map AI resolutions to Conflict resolutions
    let conflicts_resolved: Vec<Conflict> = conflicts
        .into_iter()
        .map(|mut c| {
            // Find matching AI resolution
            let matching_resolution = ai_result.resolutions.iter().find(|r| {
                c.description.contains(&r.conflict) || r.conflict.contains(&c.description)
            });

            c.suggested_resolution = Some(Resolution {
                strategy: "ai-merge".to_string(),
                selected: matching_resolution
                    .map(|r| match r.resolution.to_lowercase().as_str() {
                        "chosen" if r.chosen_instruction.contains(&c.instruction_a.content) => {
                            ResolutionChoice::UseA
                        }
                        "chosen" if r.chosen_instruction.contains(&c.instruction_b.content) => {
                            ResolutionChoice::UseB
                        }
                        "merged" => ResolutionChoice::Merge,
                        _ => ResolutionChoice::Merge,
                    })
                    .unwrap_or(ResolutionChoice::Merge),
                rationale: matching_resolution
                    .map(|r| r.rationale.clone())
                    .unwrap_or_else(|| "AI determined this resolution".to_string()),
            });
            c
        })
        .collect();

    let warnings = Vec::new();

    Ok(MergeResult {
        merged_skill,
        conflicts_resolved,
        conflicts_unresolved: Vec::new(),
        warnings,
        statistics: MergeStatistics {
            total_skills,
            total_instructions,
            conflicts_found,
            conflicts_resolved: conflicts_found,
            merge_duration_ms: start.elapsed().as_millis() as u64,
        },
    })
}

/// Ask AI to generate a question for the user about a conflict
pub async fn ask_about_conflict(
    client: &LlmClient,
    conflict: &Conflict,
) -> Result<ConflictQuestion> {
    let response = client
        .ask(
            prompts::conflict_question_system(),
            &prompts::conflict_question_prompt(
                &conflict.description,
                &conflict.instruction_a.content,
                &conflict.instruction_b.content,
                &conflict.instruction_a.skill_name,
                &conflict.instruction_b.skill_name,
            ),
        )
        .await?;

    parse_json_response(&response)
}

/// Ask AI to generate the final polished output
pub async fn generate_ai_output(client: &LlmClient, result: &MergeResult) -> Result<String> {
    let merged_json = serde_json::to_string_pretty(&serde_json::json!({
        "name": result.merged_skill.name,
        "description": result.merged_skill.description,
        "instructions": result.merged_skill.instructions.iter().map(|i| serde_json::json!({
            "command": i.command,
            "content": i.content,
            "category": i.category,
            "priority": i.priority,
        })).collect::<Vec<_>>(),
        "resolutions": result.conflicts_resolved.iter().map(|c| {
            let resolution = c.suggested_resolution.as_ref();
            serde_json::json!({
                "conflict": c.description,
                "resolution": resolution.map(|r| format!("{:?}", r.selected)).unwrap_or_default(),
                "rationale": resolution.map(|r| r.rationale.clone()).unwrap_or_default(),
            })
        }).collect::<Vec<_>>(),
    }))?;

    client
        .ask(
            prompts::generate_output_system(),
            &prompts::generate_output_prompt(&merged_json),
        )
        .await
}

fn merge_metadata(skills: &[SkillIR]) -> SkillMetadata {
    let mut metadata = SkillMetadata::default();
    metadata.tags = skills
        .iter()
        .flat_map(|s| s.metadata.tags.clone())
        .collect();
    metadata.tags.sort();
    metadata.tags.dedup();
    metadata
}

fn parse_json_response<T: serde::de::DeserializeOwned>(response: &str) -> Result<T> {
    if let Ok(result) = serde_json::from_str(response) {
        return Ok(result);
    }

    if let Some(start) = response.find("```json") {
        let json_start = start + 7;
        if let Some(end) = response[json_start..].find("```") {
            let json_str = &response[json_start..json_start + end];
            return serde_json::from_str(json_str.trim()).map_err(|e| {
                crate::error::SkillsMergeError::ParseError {
                    file: "AI response".to_string(),
                    reason: format!("Failed to parse AI JSON response: {}", e),
                }
            });
        }
    }

    if let Some(start) = response.find("```") {
        let json_start = start + 3;
        if let Some(end) = response[json_start..].find("```") {
            let json_str = &response[json_start..json_start + end];
            return serde_json::from_str(json_str.trim()).map_err(|e| {
                crate::error::SkillsMergeError::ParseError {
                    file: "AI response".to_string(),
                    reason: format!("Failed to parse AI JSON response: {}", e),
                }
            });
        }
    }

    Err(crate::error::SkillsMergeError::ParseError {
        file: "AI response".to_string(),
        reason: "Could not extract JSON from AI response".to_string(),
    })
}
