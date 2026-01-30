/// Represents an extracted comment containing the emoji marker.
///
/// This struct captures the location and content of documentation markers
/// within source code comments.
#[derive(Debug, Clone)]
pub struct ExtractedComment {
    /// The documentation content (markdown) after the marker emoji
    pub content: String,
    /// Starting line number (1-indexed)
    pub start_line: usize,
    /// Starting byte offset in the file
    pub start_byte: usize,
    /// Ending byte offset in the file
    pub end_byte: usize,
    /// The raw comment text (before the marker is removed)
    pub raw_comment: String,
}

use std::str::FromStr;
use syntect::highlighting::ScopeSelectors;

/// Parses code comments from source files using syntect for language detection.
pub struct CommentParser {
    syntax_set: syntect::parsing::SyntaxSet,
    comment_selector: ScopeSelectors,
}

impl CommentParser {
    /// Create a new comment parser with built-in syntax definitions.
    pub fn new() -> Self {
        let syntax_set = syntect::parsing::SyntaxSet::load_defaults_newlines();
        // Create a scope selector that matches comment scopes
        let comment_selector =
            ScopeSelectors::from_str("comment").expect("Failed to create comment scope selector");
        Self {
            syntax_set,
            comment_selector,
        }
    }

    /// Get the syntect syntax for a file extension.
    ///
    /// Returns None if the extension is not recognized.
    fn get_syntax_for_extension(
        &self,
        extension: &str,
    ) -> Option<syntect::parsing::SyntaxReference> {
        self.syntax_set.find_syntax_by_extension(extension).cloned()
    }

    /// Extract all emoji-marked comments from source code using syntect's tokenizer.
    ///
    /// Uses syntect to parse the source file and identify which tokens are comments
    /// based on their scope information. Only extracts markdown documentation from
    /// actual comment tokens, preventing false positives from emoji in strings or code.
    ///
    /// Returns a vector of extracted comments. If the file extension is not recognized,
    /// returns an empty vector.
    pub fn extract_doc_comments(
        &self,
        content: &str,
        file_extension: &str,
    ) -> Vec<ExtractedComment> {
        // Get the syntax definition for this file extension
        let syntax = match self.get_syntax_for_extension(file_extension) {
            Some(s) => s,
            None => return Vec::new(), // Unknown extension
        };

        let mut extracted = Vec::new();
        let mut parse_state = syntect::parsing::ParseState::new(&syntax);
        let mut current_byte = 0;

        for (line_idx, line) in content.lines().enumerate() {
            let line_num = line_idx + 1;
            let line_start_byte = current_byte;

            // Only process lines that might contain the marker (optimization)
            if !line.contains('📖') {
                // Still need to advance parser state for multi-line constructs
                let _ = parse_state.parse_line(line, &self.syntax_set);
                current_byte += line.len() + 1;
                continue;
            }

            // Parse this line with syntect to get scope operations
            let ops = parse_state.parse_line(line, &self.syntax_set);

            // Build scope stack to track which parts are comments
            let mut scope_stack = syntect::parsing::ScopeStack::new();
            let mut byte_pos = 0;

            for (offset, op) in ops.iter().flatten() {
                // Apply the scope operation to our stack
                let _ = scope_stack.apply(op);

                // Check if current scope matches comment selector
                if self
                    .comment_selector
                    .does_match(scope_stack.as_slice())
                    .is_some()
                {
                    // We're in a comment scope - extract from this position to end of line
                    let comment_text = &line[byte_pos..];

                    if comment_text.contains('📖') {
                        let token_start = line_start_byte + byte_pos;
                        let token_end = line_start_byte + line.len();

                        if let Some(mut doc) =
                            extract_marker_content(comment_text, token_start, token_end, line_num)
                        {
                            doc.raw_comment = comment_text.to_string();
                            extracted.push(doc);
                            break; // Found it, move to next line
                        }
                    }
                }

                byte_pos = *offset;
            }

            // Fallback: If syntect didn't identify comment scope, try pattern matching
            // This handles cases where syntect's scope detection might miss comments
            if !extracted.iter().any(|doc| doc.start_line == line_num)
                && let Some(comment_text) = extract_comment_by_pattern(line)
                && let Some(mut doc) = extract_marker_content(
                    &comment_text,
                    line_start_byte,
                    line_start_byte + line.len(),
                    line_num,
                )
            {
                doc.raw_comment = comment_text.to_string();
                extracted.push(doc);
            }

            current_byte += line.len() + 1; // Account for newline
        }

        extracted
    }
}

impl Default for CommentParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract comment portion from a line using pattern matching.
///
/// Fallback for when syntect doesn't identify comment scopes properly.
/// Looks for common comment markers: //, #, --, /*, etc.
fn extract_comment_by_pattern(line: &str) -> Option<String> {
    let trimmed = line.trim_start();

    // Single-line comment markers
    let line_markers = ["//", "#", "--", "//!", "///"];
    for marker in &line_markers {
        if let Some(content) = trimmed.strip_prefix(marker) {
            return Some(content.to_string());
        }
    }

    // Block comment markers
    let block_markers = ["/*", "{-", "(*"];
    for marker in &block_markers {
        if let Some(content) = trimmed.strip_prefix(marker) {
            return Some(content.to_string());
        }
    }

    None
}

/// Extract documentation content from a comment if it contains the emoji marker.
///
/// Returns Some(ExtractedComment) if the marker is found, None otherwise.
fn extract_marker_content(
    comment_text: &str,
    start_byte: usize,
    end_byte: usize,
    line_num: usize,
) -> Option<ExtractedComment> {
    // Look for the emoji marker
    let marker_pos = comment_text.find('📖')?;

    // Extract content after the marker
    let after_marker = &comment_text[marker_pos + "📖".len()..];

    // Skip the first heading marker if present (after the emoji marker)
    let content = if let Some(hash_pos) = after_marker.find('#') {
        let potential_heading = &after_marker[..hash_pos];
        // Check if it's just whitespace before the #
        if potential_heading.trim().is_empty() {
            // Include the # and everything after
            after_marker[hash_pos..].to_string()
        } else {
            after_marker.to_string()
        }
    } else {
        after_marker.to_string()
    };

    // Trim and clean the content
    let content = content
        .trim()
        .trim_end_matches("*/")
        .trim_end_matches('}')
        .trim_end()
        .to_string();

    if content.is_empty() {
        return None;
    }

    // Calculate precise byte range
    let marker_byte_offset = comment_text.find('📖').unwrap_or(0);
    let content_start = start_byte + marker_byte_offset + "📖".len();

    Some(ExtractedComment {
        content,
        start_line: line_num,
        start_byte: content_start,
        end_byte,
        raw_comment: comment_text.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_new() {
        let _parser = CommentParser::new();
    }

    #[test]
    fn test_parser_default() {
        let _parser = CommentParser::default();
    }

    #[test]
    fn test_extract_empty() {
        let parser = CommentParser::new();
        let result = parser.extract_doc_comments("", "");
        assert!(result.is_empty());
    }

    #[test]
    fn test_comment_selector_matches() {
        use syntect::parsing::{Scope, ScopeStack};

        let comment_selector = ScopeSelectors::from_str("comment").unwrap();

        // Test that comment scopes match
        let mut stack = ScopeStack::new();
        let _ = stack.push(Scope::new("comment.line.double-slash.rust").unwrap());
        assert!(comment_selector.does_match(stack.as_slice()).is_some());

        // Test that non-comment scopes don't match
        let mut stack2 = ScopeStack::new();
        let _ = stack2.push(Scope::new("string.quoted.double.rust").unwrap());
        assert!(comment_selector.does_match(stack2.as_slice()).is_none());
    }

    #[test]
    fn test_extract_marker() {
        let e = String::new();
        let result = extract_marker_content(&e, 0, 0, 1);
        assert!(result.is_none());
    }

    #[test]
    fn test_syntect_extracts_comment_not_string() {
        let parser = CommentParser::new();

        // Rust code with emoji marker in both comment and string
        let code = "// 📖 # This is documentation\nlet s = \"📖 not documentation\";";
        let result = parser.extract_doc_comments(code, "rs");

        // Should only extract from the comment, not the string
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].start_line, 1);
        assert!(result[0].content.contains("This is documentation"));
    }

    #[test]
    fn test_syntect_multiline_rust_comments() {
        let parser = CommentParser::new();

        let code = "// 📖 # First comment\nfn foo() {}\n// 📖 # Second comment\nfn bar() {}";
        let result = parser.extract_doc_comments(code, "rs");

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].start_line, 1);
        assert_eq!(result[1].start_line, 3);
        assert!(result[0].content.contains("First comment"));
        assert!(result[1].content.contains("Second comment"));
    }

    #[test]
    fn test_syntect_bash_comments() {
        let parser = CommentParser::new();

        let code = "#!/bin/bash\n# 📖 # Bash documentation\necho \"hello\"";
        let result = parser.extract_doc_comments(code, "sh");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].start_line, 2);
        assert!(result[0].content.contains("Bash documentation"));
    }

    #[test]
    fn test_syntect_javascript_comments() {
        let parser = CommentParser::new();

        let code = "// 📖 # JS documentation\nfunction foo() { const x = \"📖 not doc\"; }";
        let result = parser.extract_doc_comments(code, "js");

        // Should extract only from comment, not from string
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].start_line, 1);
        assert!(result[0].content.contains("JS documentation"));
    }

    #[test]
    fn test_extract_comment_by_pattern() {
        // Test line comment patterns
        assert_eq!(
            extract_comment_by_pattern("// test"),
            Some(" test".to_string())
        );
        assert_eq!(
            extract_comment_by_pattern("# test"),
            Some(" test".to_string())
        );
        assert_eq!(
            extract_comment_by_pattern("-- test"),
            Some(" test".to_string())
        );

        // Test block comment patterns
        assert_eq!(
            extract_comment_by_pattern("/* test"),
            Some(" test".to_string())
        );

        // Test non-comments
        assert_eq!(extract_comment_by_pattern("let x = 5;"), None);
    }
}
