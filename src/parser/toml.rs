use std::collections::HashMap;
use std::path::Path;

use crate::error::{Result, SkillsMergeError};
use crate::ir::{InputFormat, Instruction, SkillIR, SkillMetadata};

#[derive(serde::Deserialize)]
struct TomlSkill {
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    instructions: Vec<TomlInstruction>,
    #[serde(default)]
    config: Option<toml::Value>,
}

#[derive(serde::Deserialize)]
struct TomlInstruction {
    #[serde(default)]
    command: Option<String>,
    content: String,
    #[serde(default)]
    category: Option<String>,
    #[serde(default)]
    priority: Option<i32>,
}

pub fn parse(content: &str, path: &Path) -> Result<SkillIR> {
    let toml_skill: TomlSkill =
        toml::from_str(content).map_err(|e| SkillsMergeError::ParseError {
            file: path.display().to_string(),
            reason: format!("Invalid TOML: {}", e),
        })?;

    let mut instructions = Vec::new();
    for ti in &toml_skill.instructions {
        let cmd = ti.command.clone().unwrap_or_else(|| {
            ti.content
                .split_whitespace()
                .next()
                .unwrap_or("unknown")
                .to_string()
        });
        let mut instr = Instruction::new(cmd, ti.content.clone());
        if let Some(cat) = &ti.category {
            instr = instr.with_category(cat.clone());
        }
        if let Some(pri) = ti.priority {
            instr = instr.with_priority(pri);
        }
        instructions.push(instr);
    }

    let mut config = HashMap::new();
    if let Some(toml::Value::Table(table)) = &toml_skill.config {
        for (k, v) in table {
            if let Ok(json_val) = toml_to_json(v) {
                config.insert(k.clone(), json_val);
            }
        }
    }

    let mut metadata = SkillMetadata::default();
    if let Some(serde_json::Value::Number(n)) = config.get("priority") {
        metadata.priority = Some(n.as_i64().unwrap_or(0) as i32);
    }
    if let Some(serde_json::Value::String(v)) = config.get("version") {
        metadata.version = Some(v.clone());
    }

    let mut skill = SkillIR::new(toml_skill.name, path.to_path_buf(), InputFormat::Toml);
    skill.description = toml_skill.description;
    skill.instructions = instructions;
    skill.config = config;
    skill.metadata = metadata;
    Ok(skill)
}

fn toml_to_json(value: &toml::Value) -> Result<serde_json::Value> {
    match value {
        toml::Value::String(s) => Ok(serde_json::Value::String(s.clone())),
        toml::Value::Integer(i) => Ok(serde_json::Value::Number((*i).into())),
        toml::Value::Float(f) => serde_json::Number::from_f64(*f)
            .map(serde_json::Value::Number)
            .ok_or_else(|| SkillsMergeError::ParseError {
                file: String::new(),
                reason: "Invalid float value".to_string(),
            }),
        toml::Value::Boolean(b) => Ok(serde_json::Value::Bool(*b)),
        toml::Value::Array(arr) => {
            let vals: Vec<serde_json::Value> =
                arr.iter().map(toml_to_json).collect::<Result<Vec<_>>>()?;
            Ok(serde_json::Value::Array(vals))
        }
        toml::Value::Table(table) => {
            let mut map = serde_json::Map::new();
            for (k, v) in table {
                map.insert(k.clone(), toml_to_json(v)?);
            }
            Ok(serde_json::Value::Object(map))
        }
        toml::Value::Datetime(dt) => Ok(serde_json::Value::String(dt.to_string())),
    }
}
