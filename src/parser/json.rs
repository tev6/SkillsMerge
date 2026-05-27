use std::collections::HashMap;
use std::path::Path;

use crate::error::{Result, SkillsMergeError};
use crate::ir::{InputFormat, Instruction, SkillIR, SkillMetadata};

#[derive(serde::Deserialize)]
struct JsonSkill {
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    instructions: Vec<JsonInstruction>,
    #[serde(default)]
    config: Option<serde_json::Value>,
}

#[derive(serde::Deserialize)]
struct JsonInstruction {
    #[serde(default)]
    command: Option<String>,
    content: String,
    #[serde(default)]
    category: Option<String>,
    #[serde(default)]
    priority: Option<i32>,
}

pub fn parse(content: &str, path: &Path) -> Result<SkillIR> {
    let json_skill: JsonSkill = serde_json::from_str(content).map_err(|e| {
        SkillsMergeError::ParseError {
            file: path.display().to_string(),
            reason: format!("Invalid JSON: {}", e),
        }
    })?;

    let mut instructions = Vec::new();
    for ji in &json_skill.instructions {
        let cmd = ji.command.clone().unwrap_or_else(|| {
            ji.content
                .split_whitespace()
                .next()
                .unwrap_or("unknown")
                .to_string()
        });
        let mut instr = Instruction::new(cmd, ji.content.clone());
        if let Some(cat) = &ji.category {
            instr = instr.with_category(cat.clone());
        }
        if let Some(pri) = ji.priority {
            instr = instr.with_priority(pri);
        }
        instructions.push(instr);
    }

    let mut config = HashMap::new();
    if let Some(serde_json::Value::Object(map)) = &json_skill.config {
        for (k, v) in map {
            config.insert(k.clone(), v.clone());
        }
    }

    let mut metadata = SkillMetadata::default();
    if let Some(serde_json::Value::Number(n)) = config.get("priority") {
        metadata.priority = Some(n.as_i64().unwrap_or(0) as i32);
    }
    if let Some(serde_json::Value::String(v)) = config.get("version") {
        metadata.version = Some(v.clone());
    }

    let mut skill = SkillIR::new(json_skill.name, path.to_path_buf(), InputFormat::Json);
    skill.description = json_skill.description;
    skill.instructions = instructions;
    skill.config = config;
    skill.metadata = metadata;
    Ok(skill)
}
