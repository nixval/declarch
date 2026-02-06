use crate::config::kdl_modules::helpers::packages::extract_packages_to;
use crate::config::kdl_modules::types::RawConfig;
use crate::error::Result;
use kdl::KdlNode;

/// AUR (Arch User Repository) backend parser (DEPRECATED in v0.6+)
/// 
/// In v0.6+, use `pkg { aur { packages } }` syntax instead.
pub struct AurParser;

impl super::BackendParser for AurParser {
    fn name(&self) -> &'static str {
        "aur"
    }

    fn parse(&self, node: &KdlNode, config: &mut RawConfig) -> Result<()> {
        // Add to legacy_packages for backward compatibility
        extract_packages_to(node, &mut config.legacy_packages);
        Ok(())
    }
}
