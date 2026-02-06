use crate::config::kdl_modules::helpers::packages::extract_packages_to;
use crate::config::kdl_modules::types::RawConfig;
use crate::error::Result;
use kdl::KdlNode;

/// pnpm backend parser (DEPRECATED in v0.6+)
/// 
/// In v0.6+, use `pkg { pnpm { packages } }` syntax instead.
pub struct PnpmParser;

impl super::BackendParser for PnpmParser {
    fn name(&self) -> &'static str {
        "pnpm"
    }

    fn parse(&self, node: &KdlNode, config: &mut RawConfig) -> Result<()> {
        extract_packages_to(node, &mut config.legacy_packages);
        Ok(())
    }
}
