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
    /// Documentation markers to look for in comments (e.g., "📖", "DOC:", "DOCS:", "HINT:")
    markers: Vec<String>,
}

impl CommentParser {
    /// Create a new comment parser with built-in syntax definitions and default markers.
    ///
    /// Default markers: 📖 (emoji), DOC:, DOCS:, HINT:, NOTE:, INFO:
    pub fn new() -> Self {
        Self::with_markers(vec![
            "📖".to_string(),
            "DOC:".to_string(),
            "DOCS:".to_string(),
            "HINT:".to_string(),
            "NOTE:".to_string(),
            "INFO:".to_string(),
        ])
    }

    /// Create a comment parser with custom documentation markers.
    ///
    /// # Arguments
    /// * `markers` - List of strings to recognize as documentation markers
    ///
    /// # Example
    /// ```
    /// use hyperlit_engine::comment_parser::CommentParser;
    /// let parser = CommentParser::with_markers(vec!["📖".to_string(), "DOC:".to_string()]);
    /// ```
    pub fn with_markers(markers: Vec<String>) -> Self {
        let syntax_set = syntect::parsing::SyntaxSet::load_defaults_newlines();
        // Create a scope selector that matches comment scopes
        let comment_selector =
            ScopeSelectors::from_str("comment").expect("Failed to create comment scope selector");
        Self {
            syntax_set,
            comment_selector,
            markers,
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

                    // Check if comment contains any of our markers
                    if contains_any_marker(comment_text, &self.markers) {
                        let token_start = line_start_byte + byte_pos;
                        let token_end = line_start_byte + line.len();

                        if let Some(mut doc) = extract_marker_content(
                            comment_text,
                            token_start,
                            token_end,
                            line_num,
                            &self.markers,
                        ) {
                            doc.raw_comment = comment_text.to_string();
                            extracted.push(doc);
                            break; // Found it, move to next line
                        }
                    }
                }

                byte_pos = *offset;
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

/// Check if a comment contains any of the configured markers.
fn contains_any_marker(text: &str, markers: &[String]) -> bool {
    markers.iter().any(|marker| text.contains(marker.as_str()))
}

/// Find the first occurrence of any marker in the text and return its position and the marker itself.
fn find_first_marker<'a>(text: &str, markers: &'a [String]) -> Option<(usize, &'a str)> {
    let mut earliest_pos = None;
    let mut earliest_marker = None;

    for marker in markers {
        if let Some(pos) = text.find(marker.as_str())
            && (earliest_pos.is_none() || pos < earliest_pos.unwrap())
        {
            earliest_pos = Some(pos);
            earliest_marker = Some(marker.as_str());
        }
    }

    earliest_pos.map(|pos| (pos, earliest_marker.unwrap()))
}

/// Extract documentation content from a comment if it contains any of the configured markers.
///
/// Returns Some(ExtractedComment) if a marker is found, None otherwise.
fn extract_marker_content(
    comment_text: &str,
    start_byte: usize,
    end_byte: usize,
    line_num: usize,
    markers: &[String],
) -> Option<ExtractedComment> {
    // Look for any configured marker
    let (marker_pos, marker) = find_first_marker(comment_text, markers)?;

    // Extract content after the marker
    let after_marker = &comment_text[marker_pos + marker.len()..];

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
    let content_start = start_byte + marker_pos + marker.len();

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
        let markers = vec!["📖".to_string(), "DOC:".to_string()];
        let result = extract_marker_content(&e, 0, 0, 1, &markers);
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

        let code = "# 📖 # Bash documentation\necho \"hello\"";
        let result = parser.extract_doc_comments(code, "sh");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].start_line, 1);
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
    fn test_custom_marker_doc() {
        let parser = CommentParser::new();

        let code = "// DOC: # Using custom markers\n// This is documentation\nfn main() {}";
        let result = parser.extract_doc_comments(code, "rs");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].start_line, 1);
        assert!(result[0].content.contains("Using custom markers"));
    }

    #[test]
    fn test_custom_marker_hint() {
        let parser = CommentParser::new();

        let code = "// HINT: # Performance optimization\n// Use a buffer pool\nfn process() {}";
        let result = parser.extract_doc_comments(code, "rs");

        assert_eq!(result.len(), 1);
        assert!(result[0].content.contains("Performance optimization"));
    }

    #[test]
    fn test_custom_marker_note() {
        let parser = CommentParser::new();

        let code = "// NOTE: # Implementation detail\n// Uses lazy evaluation\nlet x = 5;";
        let result = parser.extract_doc_comments(code, "rs");

        assert_eq!(result.len(), 1);
        assert!(result[0].content.contains("Implementation detail"));
    }

    #[test]
    fn test_with_custom_markers_only() {
        let parser = CommentParser::with_markers(vec!["CUSTOM:".to_string()]);

        // Should find CUSTOM: marker
        let code1 = "// CUSTOM: # My marker\nfn foo() {}";
        let result1 = parser.extract_doc_comments(code1, "rs");
        assert_eq!(result1.len(), 1);
        assert!(result1[0].content.contains("My marker"));

        // Should not find emoji marker
        let code2 = "// 📖 # Not found\nfn bar() {}";
        let result2 = parser.extract_doc_comments(code2, "rs");
        assert_eq!(result2.len(), 0);
    }

    #[test]
    fn test_multiple_markers_in_file() {
        let parser = CommentParser::new();

        let code = "// 📖 # First with emoji\nfn foo() {}\n// DOC: # Second with DOC\nfn bar() {}\n// HINT: # Third with HINT\nfn baz() {}";
        let result = parser.extract_doc_comments(code, "rs");

        assert_eq!(result.len(), 3);
        assert!(result[0].content.contains("First with emoji"));
        assert!(result[1].content.contains("Second with DOC"));
        assert!(result[2].content.contains("Third with HINT"));
    }

    #[test]
    fn test_marker_case_sensitive() {
        let parser = CommentParser::new();

        // DOC: should match (uppercase)
        let code1 = "// DOC: # Upper case\nfn foo() {}";
        let result1 = parser.extract_doc_comments(code1, "rs");
        assert_eq!(result1.len(), 1);

        // doc: should NOT match (not in default markers)
        let code2 = "// doc: # Lower case\nfn bar() {}";
        let result2 = parser.extract_doc_comments(code2, "rs");
        assert_eq!(result2.len(), 0);
    }

    #[test]
    fn test_contains_any_marker() {
        let markers = vec!["📖".to_string(), "DOC:".to_string(), "HINT:".to_string()];

        assert!(contains_any_marker("// 📖 test", &markers));
        assert!(contains_any_marker("// DOC: test", &markers));
        assert!(contains_any_marker("// HINT: test", &markers));
        assert!(!contains_any_marker("// just a comment", &markers));
    }

    #[test]
    fn test_find_first_marker() {
        let markers = vec!["📖".to_string(), "DOC:".to_string()];

        // Find emoji marker
        let result1 = find_first_marker("// 📖 test", &markers);
        assert_eq!(result1, Some((3, "📖")));

        // Find DOC: marker
        let result2 = find_first_marker("// DOC: test", &markers);
        assert_eq!(result2, Some((3, "DOC:")));

        // Find first when both present
        let result3 = find_first_marker("// DOC: and 📖 both", &markers);
        assert_eq!(result3, Some((3, "DOC:")));

        // No marker found
        let result4 = find_first_marker("// no marker", &markers);
        assert_eq!(result4, None);
    }
}
