use crate::config::kdl_modules::types::RawConfig;
use crate::error::Result;
use crate::config::kdl::extract_packages_to;
use kdl::KdlNode;

/// npm backend parser
pub struct NpmParser;

impl super::BackendParser for NpmParser {
    fn name(&self) -> &'static str {
        "npm"
    }

    fn parse(&self, node: &KdlNode, config: &mut RawConfig) -> Result<()> {
        extract_packages_to(node, &mut config.npm_packages);
        Ok(())
    }
}
