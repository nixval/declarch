use crate::config::kdl_modules::helpers::packages::extract_packages_to;
use crate::config::kdl_modules::types::{Backend, RawConfig};
use crate::error::Result;
use kdl::KdlNode;

/// Homebrew backend parser
pub struct BrewParser;

impl super::BackendParser for BrewParser {
    fn name(&self) -> &'static str {
        "brew"
    }

    fn parse(&self, node: &KdlNode, config: &mut RawConfig) -> Result<()> {
        extract_packages_to(node, config.packages_for_mut(&Backend::Brew));
        Ok(())
    }
}
