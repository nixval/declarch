use crate::config::kdl_modules::helpers::packages::extract_packages_to;
use crate::config::kdl_modules::types::RawConfig;
use crate::error::Result;
use kdl::KdlNode;

/// pip backend parser
pub struct PipParser;

impl super::BackendParser for PipParser {
    fn name(&self) -> &'static str {
        "pip"
    }

    fn parse(&self, node: &KdlNode, config: &mut RawConfig) -> Result<()> {
        extract_packages_to(node, &mut config.pip_packages);
        Ok(())
    }
}
