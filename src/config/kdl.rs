//! KDL configuration parsing facade
//!
//! This module provides a simplified interface to the KDL configuration
//! parsing functionality. The actual implementation is in the kdl_modules subdirectory.

// Re-export types from kdl_modules for backward compatibility
pub use crate::config::kdl_modules::types::{
    ActionCondition, ActionType, ConflictEntry, ErrorBehavior, LifecycleAction, LifecycleConfig,
    LifecyclePhase, PackageEntry, PolicyConfig, ProjectMetadata, RawConfig,
};

// Re-export BackendParser trait (now in registry for backward compatibility)
pub use crate::config::kdl_modules::registry::BackendParser;

// Re-export BackendParserRegistry
pub use crate::config::kdl_modules::registry::BackendParserRegistry;

// Re-export parse_kdl_content function
pub use crate::config::kdl_modules::parser::parse_kdl_content;

// Re-export helper functions for backward compatibility
pub use crate::config::kdl_modules::helpers::{
    conflicts, env, hooks, meta, package_mappings, packages, policy, repositories,
};

#[cfg(test)]
mod tests {
    use super::*;

    include!("kdl_tests.rs");
}
