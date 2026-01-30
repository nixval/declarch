use crate::config::kdl_modules::helpers::packages::extract_packages_to;
use crate::config::kdl_modules::types::RawConfig;
use crate::error::Result;
use kdl::KdlNode;

/// Cargo backend parser
pub struct CargoParser;

impl super::BackendParser for CargoParser {
    fn name(&self) -> &'static str {
        "cargo"
    }

    fn parse(&self, node: &KdlNode, config: &mut RawConfig) -> Result<()> {
        extract_packages_to(node, &mut config.cargo_packages);
        Ok(())
    }
}
