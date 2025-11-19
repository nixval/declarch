use thiserror::Error;
use std::io;
use std::path::PathBuf;

#[derive(Error, Debug)]
pub enum DeclarchError {
    // File/IO errors
    #[error("Config file not found: {path}")]
    ConfigNotFound { path: PathBuf },

    #[error("Failed to read file {path}: {reason}")]
    FileReadError { path: PathBuf, reason: String },

    #[error("Failed to write file {path}: {reason}")]
    FileWriteError { path: PathBuf, reason: String },

    #[error("Config directory not found: {path}")]
    ConfigDirNotFound { path: PathBuf },

    // Parse errors
    #[error("Failed to parse KDL: {reason}")]
    KdlParseError { reason: String },

    #[error("Invalid config syntax at {file}:{line}: {message}")]
    InvalidSyntax {
        file: String,
        line: usize,
        message: String,
    },

    // Module/package errors
    #[error("Module not found: {name}")]
    ModuleNotFound { name: String },

    #[error("Package not found: {name}")]
    PackageNotFound { name: String },

    #[error("Host not found: {name}")]
    HostNotFound { name: String },

    // State errors
    #[error("State file corrupted: {reason}")]
    StateCorrupted { reason: String },

    #[error("Failed to deserialize state: {reason}")]
    StateDeserializeError { reason: String },

    // System errors
    #[error("System command failed: {command}\nReason: {reason}")]
    SystemCommandFailed { command: String, reason: String },

    #[error("Permission denied: {reason}")]
    PermissionDenied { reason: String },

    // Package manager errors
    #[error("Package manager error: {reason}")]
    PackageManagerError { reason: String },

    // Generic errors
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("UTF8 error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, DeclarchError>;
