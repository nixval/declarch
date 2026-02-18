use crate::error::Result;
use kdl::KdlDocument;

pub(super) fn parse_document_with_error_reporting(
    content: &str,
    file_path: Option<&str>,
) -> Result<KdlDocument> {
    content.parse().map_err(|e: kdl::KdlError| {
        let report =
            crate::config::kdl_modules::error_reporter::format_error_report(content, file_path, &e);
        crate::error::DeclarchError::ConfigError(report)
    })
}
