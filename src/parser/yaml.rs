use std::collections::HashMap;
use std::path::Path;

use crate::error::{Result, SkillsMergeError};
use crate::ir::{InputFormat, Instruction, SkillIR, SkillMetadata};

#[derive(serde::Deserialize)]
struct YamlSkill {
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    instructions: Vec<YamlInstruction>,
    #[serde(default)]
    config: Option<serde_json::Value>,
}

#[derive(serde::Deserialize)]
struct YamlInstruction {
    #[serde(default)]
    command: Option<String>,
    content: String,
    #[serde(default)]
    category: Option<String>,
    #[serde(default)]
    priority: Option<i32>,
}

pub fn parse(content: &str, path: &Path) -> Result<SkillIR> {
    let yaml_skill: YamlSkill = serde_yaml::from_str(content).map_err(|e| {
        SkillsMergeError::ParseError {
            file: path.display().to_string(),
            reason: format!("Invalid YAML: {}", e),
        }
    })?;

    let mut instructions = Vec::new();
    for yi in &yaml_skill.instructions {
        let cmd = yi.command.clone().unwrap_or_else(|| {
            yi.content
                .split_whitespace()
                .next()
                .unwrap_or("unknown")
                .to_string()
        });
        let mut instr = Instruction::new(cmd, yi.content.clone());
        if let Some(cat) = &yi.category {
            instr = instr.with_category(cat.clone());
        }
        if let Some(pri) = yi.priority {
            instr = instr.with_priority(pri);
        }
        instructions.push(instr);
    }

    let mut config = HashMap::new();
    if let Some(serde_json::Value::Object(map)) = &yaml_skill.config {
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

    let mut skill = SkillIR::new(yaml_skill.name, path.to_path_buf(), InputFormat::Yaml);
    skill.description = yaml_skill.description;
    skill.instructions = instructions;
    skill.config = config;
    skill.metadata = metadata;
    Ok(skill)
}
