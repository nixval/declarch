use crate::config::kdl_modules::helpers::packages::extract_packages_to;
use crate::config::kdl_modules::types::{Backend, RawConfig};
use crate::error::Result;
use kdl::KdlNode;

/// Yarn backend parser
pub struct YarnParser;

impl super::BackendParser for YarnParser {
    fn name(&self) -> &'static str {
        "yarn"
    }

    fn parse(&self, node: &KdlNode, config: &mut RawConfig) -> Result<()> {
        extract_packages_to(node, config.packages_for_mut(&Backend::Yarn));
        Ok(())
    }
}
