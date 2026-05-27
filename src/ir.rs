use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// SKILLS文件中间表示
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillIR {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub instructions: Vec<Instruction>,
    pub config: HashMap<String, serde_json::Value>,
    pub metadata: SkillMetadata,
    pub source: SourceInfo,
}

/// 指令结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instruction {
    pub id: Uuid,
    pub command: String,
    pub content: String,
    pub category: Option<String>,
    pub priority: i32,
    pub parameters: Vec<Parameter>,
    pub examples: Vec<String>,
}

/// 参数定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub value: serde_json::Value,
    pub required: bool,
}

/// 元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub version: Option<String>,
    pub priority: Option<i32>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
    pub extra: HashMap<String, serde_json::Value>,
}

impl Default for SkillMetadata {
    fn default() -> Self {
        Self {
            version: None,
            priority: None,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
            tags: Vec::new(),
            extra: HashMap::new(),
        }
    }
}

/// 来源信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfo {
    pub path: PathBuf,
    pub format: InputFormat,
    pub loaded_at: DateTime<Utc>,
}

/// 输入格式枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputFormat {
    Markdown,
    Json,
    Yaml,
    Toml,
}

/// 冲突类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictType {
    InstructionOverride,
    PriorityConflict,
    ParameterIncompatible,
    SemanticConflict,
    CircularDependency,
}

/// 冲突严重程度
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Info = 1,
    Low = 2,
    Medium = 3,
    High = 4,
    Critical = 5,
}

/// 冲突结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub id: Uuid,
    pub conflict_type: ConflictType,
    pub severity: Severity,
    pub sources: Vec<ConflictSource>,
    pub instruction_a: InstructionRef,
    pub instruction_b: InstructionRef,
    pub description: String,
    pub suggested_resolution: Option<Resolution>,
}

/// 冲突来源
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictSource {
    pub skill_id: Uuid,
    pub skill_name: String,
    pub instruction_id: Uuid,
}

/// 指令引用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionRef {
    pub skill_name: String,
    pub instruction_id: Uuid,
    pub command: String,
    pub content: String,
}

/// 解决方案
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolution {
    pub strategy: String,
    pub selected: ResolutionChoice,
    pub rationale: String,
}

/// 解决选择
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionChoice {
    UseA,
    UseB,
    Merge,
    Skip,
}

/// 合并策略
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeStrategy {
    PreserveAll,
    AutoSelect,
    Interactive,
    SemanticMerge,
    Custom(String),
}

/// 输出格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Markdown,
    Json,
    Yaml,
    Toml,
}

/// 合并结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeResult {
    pub merged_skill: SkillIR,
    pub conflicts_resolved: Vec<Conflict>,
    pub conflicts_unresolved: Vec<Conflict>,
    pub warnings: Vec<String>,
    pub statistics: MergeStatistics,
}

/// 合并统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeStatistics {
    pub total_skills: usize,
    pub total_instructions: usize,
    pub conflicts_found: usize,
    pub conflicts_resolved: usize,
    pub merge_duration_ms: u64,
}

/// 警告类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Warning {
    pub code: String,
    pub message: String,
    pub source: Option<String>,
}

impl SkillIR {
    pub fn new(name: String, source_path: PathBuf, format: InputFormat) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description: None,
            instructions: Vec::new(),
            config: HashMap::new(),
            metadata: SkillMetadata {
                version: None,
                priority: None,
                created_at: Some(Utc::now()),
                updated_at: Some(Utc::now()),
                tags: Vec::new(),
                extra: HashMap::new(),
            },
            source: SourceInfo {
                path: source_path,
                format,
                loaded_at: Utc::now(),
            },
        }
    }
}

impl Instruction {
    pub fn new(command: String, content: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            command,
            content,
            category: None,
            priority: 0,
            parameters: Vec::new(),
            examples: Vec::new(),
        }
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_category(mut self, category: String) -> Self {
        self.category = Some(category);
        self
    }
}

impl Conflict {
    pub fn new(
        conflict_type: ConflictType,
        severity: Severity,
        instruction_a: InstructionRef,
        instruction_b: InstructionRef,
        description: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            conflict_type,
            severity,
            sources: Vec::new(),
            instruction_a,
            instruction_b,
            description,
            suggested_resolution: None,
        }
    }
}

impl std::fmt::Display for InputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputFormat::Markdown => write!(f, "Markdown"),
            InputFormat::Json => write!(f, "JSON"),
            InputFormat::Yaml => write!(f, "YAML"),
            InputFormat::Toml => write!(f, "TOML"),
        }
    }
}

impl std::fmt::Display for ConflictType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConflictType::InstructionOverride => write!(f, "instruction_override"),
            ConflictType::PriorityConflict => write!(f, "priority_conflict"),
            ConflictType::ParameterIncompatible => write!(f, "parameter_incompatible"),
            ConflictType::SemanticConflict => write!(f, "semantic_conflict"),
            ConflictType::CircularDependency => write!(f, "circular_dependency"),
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Info => write!(f, "INFO"),
            Severity::Low => write!(f, "LOW"),
            Severity::Medium => write!(f, "MEDIUM"),
            Severity::High => write!(f, "HIGH"),
            Severity::Critical => write!(f, "CRITICAL"),
        }
    }
}

impl std::str::FromStr for MergeStrategy {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "preserve-all" | "preserve_all" => Ok(MergeStrategy::PreserveAll),
            "auto" | "auto-select" | "auto_select" => Ok(MergeStrategy::AutoSelect),
            "interactive" => Ok(MergeStrategy::Interactive),
            "semantic" | "semantic-merge" | "semantic_merge" => Ok(MergeStrategy::SemanticMerge),
            other => Ok(MergeStrategy::Custom(other.to_string())),
        }
    }
}

impl std::str::FromStr for OutputFormat {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "markdown" | "md" => Ok(OutputFormat::Markdown),
            "json" => Ok(OutputFormat::Json),
            "yaml" | "yml" => Ok(OutputFormat::Yaml),
            "toml" => Ok(OutputFormat::Toml),
            other => Err(format!("Unknown output format: {}", other)),
        }
    }
}
