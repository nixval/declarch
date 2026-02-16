//! KDL configuration parsing facade
//!
//! This module provides a simplified interface to the KDL configuration
//! parsing functionality. The actual implementation is in the kdl_modules subdirectory.

// Re-export types from kdl_modules for backward compatibility
pub use crate::config::kdl_modules::types::{
    ActionCondition, ActionType, ConflictEntry, ErrorBehavior, LifecycleAction, LifecycleConfig,
    LifecyclePhase, McpConfig, PackageEntry, PolicyConfig, ProjectMetadata, RawConfig,
};

// Re-export BackendParser trait (now in registry for backward compatibility)
pub use crate::config::kdl_modules::registry::BackendParser;

// Re-export BackendParserRegistry
pub use crate::config::kdl_modules::registry::BackendParserRegistry;

// Re-export parse_kdl_content function
pub use crate::config::kdl_modules::parser::{parse_kdl_content, parse_kdl_content_with_path};

// Re-export helper functions for backward compatibility
pub use crate::config::kdl_modules::helpers::{
    conflicts, env, hooks, meta, packages, policy, repositories,
};
