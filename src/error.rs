use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DeclarchError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error at '{path}': {source}")]
    IoError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("IO error: {0}")]
    StdIoError(#[from] std::io::Error),

    #[error("Parsing error in '{file}': {message}")]
    ParseError { file: String, message: String },

    #[error("KDL parse error: {0}")]
    KdlError(#[from] kdl::KdlError),

    #[error("Package manager error: {0}")]
    PackageManagerError(String),

    #[error("Target not found: {0}")]
    TargetNotFound(String),

    #[error("Operation interrupted by user")]
    Interrupted,

    #[error("System dependency missing: {0}")]
    DependencyMissing(String),

    #[error(transparent)]
    JsonError(#[from] serde_json::Error),

    #[error(transparent)]
    YamlError(#[from] serde_yaml::Error),

    #[error("Config file not found at: {path}")]
    ConfigNotFound { path: PathBuf },

    #[error("System command '{command}' failed: {reason}")]
    SystemCommandFailed { command: String, reason: String },

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, DeclarchError>;
