use std::collections::HashMap;
use std::path::Path;

use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Parser, Tag, TagEnd};

use crate::error::Result;
use crate::ir::{InputFormat, Instruction, SkillIR, SkillMetadata};

pub fn parse(content: &str, path: &Path) -> Result<SkillIR> {
    let (front_matter, body) = split_front_matter(content);

    let mut metadata = SkillMetadata::default();
    let mut name = String::from(
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unnamed"),
    );
    let mut description: Option<String> = None;
    let mut instructions: Vec<Instruction> = Vec::new();
    let mut config: HashMap<String, serde_json::Value> = HashMap::new();

    if let Some(fm) = front_matter {
        parse_front_matter(fm, &mut metadata, &mut name);
    }

    let parser = Parser::new(body);
    let mut current_section: Option<String> = None;
    let mut heading_level: Option<u8> = None;
    let mut heading_text = String::new();
    let mut in_code_block = false;
    let mut code_block_lang = String::new();
    let mut code_block_content = String::new();
    let mut in_list_item = false;
    let mut list_item_content = String::new();
    let mut first_paragraph_done = false;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                heading_level = Some(level as u8);
                heading_text.clear();
            }
            Event::End(TagEnd::Heading(level)) => {
                if level == HeadingLevel::H1 {
                    name = heading_text.trim().to_string();
                    current_section = None;
                } else if level == HeadingLevel::H2 {
                    current_section = Some(heading_text.trim().to_string());
                }
                heading_level = None;
                heading_text.clear();
            }
            Event::Start(Tag::Paragraph) => {}
            Event::End(TagEnd::Paragraph) => {}
            Event::Start(Tag::Item) => {
                in_list_item = true;
                list_item_content.clear();
            }
            Event::End(TagEnd::Item) => {
                in_list_item = false;
                if current_section.as_deref() == Some("Instructions")
                    && !list_item_content.trim().is_empty()
                {
                    instructions.push(parse_instruction_from_text(&list_item_content));
                }
                list_item_content.clear();
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                in_code_block = true;
                code_block_content.clear();
                code_block_lang = match kind {
                    CodeBlockKind::Fenced(lang) => lang.to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                if current_section.as_deref() == Some("Configuration") && code_block_lang == "json"
                {
                    if let Ok(serde_json::Value::Object(map)) =
                        serde_json::from_str(&code_block_content)
                    {
                        for (k, v) in map {
                            config.insert(k, v);
                        }
                    }
                }
                code_block_lang.clear();
                code_block_content.clear();
            }
            Event::Text(text) => {
                if in_code_block {
                    code_block_content.push_str(&text);
                } else if in_list_item {
                    list_item_content.push_str(&text);
                } else if heading_level.is_some() {
                    heading_text.push_str(&text);
                } else if description.is_none()
                    && !first_paragraph_done
                    && current_section.is_none()
                {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        description = Some(trimmed.to_string());
                        first_paragraph_done = true;
                    }
                }
            }
            _ => {}
        }
    }

    if let Some(serde_json::Value::Number(n)) = config.get("priority") {
        metadata.priority = Some(n.as_i64().unwrap_or(0) as i32);
    }
    if let Some(serde_json::Value::String(v)) = config.get("version") {
        metadata.version = Some(v.clone());
    }

    let mut skill = SkillIR::new(name, path.to_path_buf(), InputFormat::Markdown);
    skill.description = description;
    skill.instructions = instructions;
    skill.config = config;
    skill.metadata = metadata;
    Ok(skill)
}

fn split_front_matter(content: &str) -> (Option<&str>, &str) {
    let trimmed = content.trim_start();
    if let Some(stripped) = trimmed.strip_prefix("---") {
        if let Some(end) = stripped.find("---") {
            let fm = &stripped[..end];
            let body = &stripped[end + 3..];
            return (Some(fm.trim()), body);
        }
    }
    (None, content)
}

fn parse_front_matter(fm: &str, metadata: &mut SkillMetadata, name: &mut String) {
    for line in fm.lines() {
        let line = line.trim();
        if let Some(val) = line.strip_prefix("priority:") {
            metadata.priority = val.trim().parse().ok();
        } else if let Some(val) = line.strip_prefix("version:") {
            metadata.version = Some(val.trim().trim_matches('"').to_string());
        } else if let Some(val) = line.strip_prefix("name:") {
            *name = val.trim().trim_matches('"').to_string();
        } else if let Some(val) = line.strip_prefix("tags:") {
            metadata.tags = val
                .split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect();
        }
    }
}

fn parse_instruction_from_text(text: &str) -> Instruction {
    let text = text.trim();
    let (command, content) = if let Some(space) = text.find(' ') {
        (text[..space].to_string(), text[space + 1..].to_string())
    } else {
        (text.to_string(), String::new())
    };
    Instruction::new(command, content)
}
