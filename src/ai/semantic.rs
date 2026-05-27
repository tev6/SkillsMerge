use crate::ai::client::LlmClient;
use crate::ai::prompts;
use crate::error::Result;
use crate::ir::{Conflict, ConflictType, InstructionRef, Severity, SkillIR};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ConflictAnalysis {
    conflicts: Vec<AiDetectedConflict>,
    #[serde(default)]
    #[allow(dead_code)]
    compatible_pairs: Vec<CompatiblePair>,
}

#[derive(Debug, Deserialize)]
struct AiDetectedConflict {
    instruction_a: AiInstructionRef,
    instruction_b: AiInstructionRef,
    conflict_type: String,
    severity: String,
    description: String,
    #[serde(default)]
    scenario: String,
}

#[derive(Debug, Deserialize)]
struct AiInstructionRef {
    skill_name: String,
    instruction: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct CompatiblePair {
    instruction_a: AiInstructionRef,
    instruction_b: AiInstructionRef,
    #[serde(default)]
    note: String,
}

/// Use AI to detect semantic conflicts between skills
pub async fn detect_semantic_conflicts(
    client: &LlmClient,
    skills: &[SkillIR],
) -> Result<Vec<Conflict>> {
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

    let response = client
        .ask(
            prompts::semantic_analysis_system(),
            &prompts::detect_conflicts_prompt(&skills_json),
        )
        .await?;

    let analysis: ConflictAnalysis = parse_json_response(&response)?;

    let mut conflicts = Vec::new();
    for ac in analysis.conflicts {
        let conflict_type = match ac.conflict_type.to_lowercase().as_str() {
            "contradiction" => ConflictType::SemanticConflict,
            "incompatible" => ConflictType::ParameterIncompatible,
            "overlap" => ConflictType::InstructionOverride,
            "partial_conflict" => ConflictType::PriorityConflict,
            _ => ConflictType::SemanticConflict,
        };

        let severity = match ac.severity.to_lowercase().as_str() {
            "critical" => Severity::Critical,
            "high" => Severity::High,
            "medium" => Severity::Medium,
            "low" => Severity::Low,
            _ => Severity::Info,
        };

        let instr_a = InstructionRef {
            skill_name: ac.instruction_a.skill_name.clone(),
            instruction_id: uuid::Uuid::new_v4(),
            command: ac
                .instruction_a
                .instruction
                .split_whitespace()
                .next()
                .unwrap_or("unknown")
                .to_string(),
            content: ac.instruction_a.instruction.clone(),
        };

        let instr_b = InstructionRef {
            skill_name: ac.instruction_b.skill_name.clone(),
            instruction_id: uuid::Uuid::new_v4(),
            command: ac
                .instruction_b
                .instruction
                .split_whitespace()
                .next()
                .unwrap_or("unknown")
                .to_string(),
            content: ac.instruction_b.instruction.clone(),
        };

        let mut conflict = Conflict::new(conflict_type, severity, instr_a, instr_b, ac.description);

        if !ac.scenario.is_empty() {
            conflict.description = format!("{}\n\nScenario: {}", conflict.description, ac.scenario);
        }

        conflicts.push(conflict);
    }

    // Sort by severity descending
    conflicts.sort_by_key(|b| std::cmp::Reverse(b.severity));
    Ok(conflicts)
}

/// Try to parse JSON from an AI response, handling markdown code blocks
fn parse_json_response<T: serde::de::DeserializeOwned>(response: &str) -> Result<T> {
    // Try direct parse first
    if let Ok(result) = serde_json::from_str(response) {
        return Ok(result);
    }

    // Try extracting from markdown code block
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

    // Try extracting from plain code block
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
