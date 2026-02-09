//! Enhanced error reporting for KDL parsing
//!
//! Provides beautiful, helpful error messages similar to rustc.
//! Shows line numbers, visual indicators, context, and actionable hints.

use kdl::KdlError;
use std::fmt::Write;

/// Format a beautiful error report from KdlError
/// 
/// Example output:
/// ```text
/// error: Unexpected token in KDL document
///   --> ~/.config/declarch/declarch.kdl:15:5
///    |
/// 15 |     format whitespace
///    |     ^^^^^^ Expected string, found bare word
///    |
///    = hint: Try using "whitespace" (with quotes) instead of whitespace
/// ```
pub fn format_error_report(
    content: &str,
    file_path: Option<&str>,
    error: &KdlError,
) -> String {
    let mut report = String::new();
    
    // Get diagnostics from the error
    let diagnostics = &error.diagnostics;
    
    if diagnostics.is_empty() {
        // Fallback if no diagnostics
        let _ = writeln!(&mut report, "\x1b[31merror\x1b[0m: Failed to parse KDL document");
        return report;
    }
    
    // Process each diagnostic
    for (i, diag) in diagnostics.iter().enumerate() {
        if i > 0 {
            let _ = writeln!(&mut report);
        }
        
        // Get line and column from span
        let (line_num, col_num) = offset_to_line_col(content, diag.span.offset());
        let message = diag.message.clone().unwrap_or_else(|| "Parse error".to_string());
        
        // Header: error type and message
        let _ = writeln!(&mut report, "\x1b[31merror\x1b[0m: {}", message);
        
        // Location: file path and position
        if let Some(path) = file_path {
            let _ = writeln!(&mut report, "  \x1b[34m-->\x1b[0m {}:{}:{}", path, line_num, col_num);
        }
        
        // Source context with line numbers and visual indicator
        let context = build_source_context(content, line_num, Some(col_num), diag.span.len());
        let _ = writeln!(&mut report, "{}", context);
        
        // Help text from diagnostic
        if let Some(help) = &diag.help {
            let _ = writeln!(&mut report, "  \x1b[34m=\x1b[0m \x1b[33mhelp\x1b[0m: {}", help);
        } else {
            // Helpful hint based on error type
            if let Some(hint) = get_error_hint(content, line_num, &message) {
                let _ = writeln!(&mut report, "  \x1b[34m=\x1b[0m {}", hint);
            }
        }
    }
    
    report
}

/// Convert byte offset to line and column numbers
fn offset_to_line_col(content: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    
    for (i, c) in content.chars().enumerate() {
        if i >= offset {
            break;
        }
        if c == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    
    (line, col)
}

/// Build source context with line numbers and error indicator
fn build_source_context(content: &str, error_line: usize, error_col: Option<usize>, span_len: usize) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut output = String::new();
    
    if lines.is_empty() {
        return output;
    }
    
    // Calculate context range (2 lines before and after error)
    let start = error_line.saturating_sub(2).max(1);
    let end = (error_line + 2).min(lines.len());
    
    // Find the width needed for line numbers
    let line_num_width = lines.len().to_string().len();
    
    // Top border
    let _ = writeln!(&mut output, " {:width$} │", "", width = line_num_width);
    
    for line_num in start..=end {
        let line_idx = line_num - 1;
        if line_idx >= lines.len() {
            continue;
        }
        
        let line_content = lines[line_idx];
        
        if line_num == error_line {
            // Error line with highlighting
            let _ = writeln!(
                &mut output,
                " \x1b[34m{:>width$} │\x1b[0m {}",
                line_num,
                line_content,
                width = line_num_width
            );
            
            // Visual indicator pointing to error
            if let Some(col) = error_col {
                let indent = " ".repeat(line_num_width + 1);
                let spaces = " ".repeat(col.saturating_sub(1));
                let highlight_len = span_len.clamp(1, 20); // Limit highlight length
                let carets = "^".repeat(highlight_len);
                // Build the indicator line piece by piece to avoid format string issues
                let _ = write!(&mut output, " {}", indent);
                let _ = write!(&mut output, "\x1b[34m│\x1b[0m ");
                let _ = write!(&mut output, "{}", spaces);
                let _ = write!(&mut output, "\x1b[31m{}\x1b[0m", carets);
                let _ = writeln!(&mut output);
            }
        } else {
            // Context lines
            let _ = writeln!(
                &mut output,
                " \x1b[34m{:>width$} │\x1b[0m {}",
                line_num,
                line_content,
                width = line_num_width
            );
        }
    }
    
    // Bottom border
    let _ = writeln!(&mut output, " {:width$} │", "", width = line_num_width);
    
    output
}

/// Get helpful hint based on error type and content
fn get_error_hint(content: &str, error_line: usize, message: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    let error_content = lines.get(error_line.saturating_sub(1)).unwrap_or(&"");
    
    // Common error patterns and their solutions
    if message.contains("unexpected token") || message.contains("bare word") {
        if error_content.contains("format ") && !error_content.contains("format \"") {
            return Some("\x1b[33mhint\x1b[0m: Format values need quotes. Try: \x1b[32mformat \"whitespace\"\x1b[0m".to_string());
        }
        if error_content.contains("needs_sudo ") || error_content.contains("needs_sudo ") {
            return Some("\x1b[33mhint\x1b[0m: Boolean values need quotes. Try: \x1b[32mneeds_sudo \"true\"\x1b[0m or \x1b[32mneeds_sudo \"false\"\x1b[0m".to_string());
        }
        if error_content.contains("true") || error_content.contains("false") {
            return Some("\x1b[33mhint\x1b[0m: Bare words like `true` or `false` need quotes. Try: \x1b[32m\"true\"\x1b[0m or \x1b[32m\"false\"\x1b[0m".to_string());
        }
        return Some("\x1b[33mhint\x1b[0m: Check for missing quotes around values or invalid keywords".to_string());
    }
    
    if message.contains("unmatched") || message.contains("mismatched") {
        return Some("\x1b[33mhint\x1b[0m: You might have an extra or missing brace `{` or `}`. Check that all blocks are properly closed.".to_string());
    }
    
    if message.contains("unexpected end of file") {
        return Some("\x1b[33mhint\x1b[0m: The file ended unexpectedly. Check for missing closing braces `}` or quotes.".to_string());
    }
    
    if message.contains("expected") {
        return Some("\x1b[33mhint\x1b[0m: Check KDL syntax. Format: \x1b[32mnode-name \"value\" { child-nodes }\x1b[0m".to_string());
    }
    
    None
}

/// Parse KDL content with enhanced error reporting
pub fn parse_with_report(content: &str, file_path: Option<&str>) -> Result<kdl::KdlDocument, String> {
    content.parse::<kdl::KdlDocument>().map_err(|e: kdl::KdlError| {
        format_error_report(content, file_path, &e)
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_offset_to_line_col() {
        use super::offset_to_line_col;
        
        let content = "line1\nline2\nline3";
        assert_eq!(offset_to_line_col(content, 0), (1, 1));
        assert_eq!(offset_to_line_col(content, 6), (2, 1));
        assert_eq!(offset_to_line_col(content, 12), (3, 1));
    }
}
