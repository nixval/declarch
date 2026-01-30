use crate::config::kdl_modules::helpers::packages::extract_packages_to;
use crate::config::kdl_modules::types::RawConfig;
use crate::error::Result;
use kdl::KdlNode;

/// Bun backend parser
pub struct BunParser;

impl super::BackendParser for BunParser {
    fn name(&self) -> &'static str {
        "bun"
    }

    fn parse(&self, node: &KdlNode, config: &mut RawConfig) -> Result<()> {
        extract_packages_to(node, &mut config.bun_packages);
        Ok(())
    }
}
