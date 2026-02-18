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

#[cfg(test)]
mod tests {
    use super::parse_document_with_error_reporting;

    #[test]
    fn parse_document_accepts_valid_kdl() {
        let doc = parse_document_with_error_reporting("pkg { aur { bat } }", None)
            .expect("valid kdl should parse");
        assert_eq!(doc.nodes().len(), 1);
    }

    #[test]
    fn parse_document_reports_invalid_kdl() {
        let err = parse_document_with_error_reporting("pkg {", Some("/tmp/sample.kdl"))
            .expect_err("invalid kdl should fail");
        let msg = err.to_string();
        assert!(msg.contains("sample.kdl") || msg.contains("Config error"));
    }
}
