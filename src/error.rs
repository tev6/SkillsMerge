use thiserror::Error;

#[derive(Error, Debug)]
pub enum SkillsMergeError {
    #[error("[E001] Parse error in file '{file}': {reason}")]
    ParseError { file: String, reason: String },

    #[error("[E002] File not found: {path}")]
    FileNotFound { path: String },

    #[error("[E003] Permission denied: {path}")]
    PermissionDenied { path: String },

    #[error("[E004] Circular dependency detected: {cycle}")]
    CircularDependency { cycle: String },

    #[error("[E005] Invalid format: {format}")]
    InvalidFormat { format: String },

    #[error("[E006] {count} unresolved conflict(s) remain")]
    ConflictUnresolved { count: usize },

    #[error("[E007] Encoding error in file: {file}")]
    EncodingError { file: String },

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, SkillsMergeError>;
