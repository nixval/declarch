use crate::config::kdl_modules::helpers::packages::extract_packages_to;
use crate::config::kdl_modules::types::RawConfig;
use crate::error::Result;
use kdl::KdlNode;

/// Soar (static binaries) backend parser (DEPRECATED in v0.6+)
/// 
/// In v0.6+, use `pkg { soar { packages } }` syntax instead.
pub struct SoarParser;

impl super::BackendParser for SoarParser {
    fn name(&self) -> &'static str {
        "soar"
    }

    fn aliases(&self) -> &[&'static str] {
        &["app"] // "app" is an alias for "soar"
    }

    fn parse(&self, node: &KdlNode, config: &mut RawConfig) -> Result<()> {
        extract_packages_to(node, &mut config.legacy_packages);
        Ok(())
    }
}
