use crate::config::kdl_modules::helpers::packages::extract_packages_to;
use crate::config::kdl_modules::types::{Backend, RawConfig};
use crate::error::Result;
use kdl::KdlNode;

/// pip backend parser
pub struct PipParser;

impl super::BackendParser for PipParser {
    fn name(&self) -> &'static str {
        "pip"
    }

    fn parse(&self, node: &KdlNode, config: &mut RawConfig) -> Result<()> {
        extract_packages_to(node, config.packages_for_mut(&Backend::Pip));
        Ok(())
    }
}
