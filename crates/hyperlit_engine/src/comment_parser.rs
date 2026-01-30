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

/// Parses code comments from source files using syntect for language detection.
pub struct CommentParser {
    syntax_set: syntect::parsing::SyntaxSet,
}

impl CommentParser {
    /// Create a new comment parser with built-in syntax definitions.
    pub fn new() -> Self {
        let syntax_set = syntect::parsing::SyntaxSet::load_defaults_newlines();
        Self { syntax_set }
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

    /// Extract all emoji-marked comments from source code.
    ///
    /// Returns a vector of extracted comments. If the file extension is not recognized,
    /// returns an empty vector. If syntect parsing fails, returns empty.
    pub fn extract_doc_comments(
        &self,
        content: &str,
        file_extension: &str,
    ) -> Vec<ExtractedComment> {
        // Check if this file type is supported
        if self.get_syntax_for_extension(file_extension).is_none() {
            return Vec::new(); // Unknown extension, skip
        }

        // For now, use simple line-by-line scanning for comment patterns
        // In a real implementation, this would use syntect's tokenizer properly
        let mut extracted = Vec::new();
        let mut current_byte = 0;

        for (line_number, line) in content.lines().enumerate() {
            let line_num = line_number + 1; // 1-indexed

            // Simple pattern matching for comments with emoji marker
            // Look for lines containing the emoji marker
            if line.contains('📖') {
                // Try to extract from this line
                let line_start = current_byte;
                if let Some(doc) =
                    extract_marker_content(line, line_start, line_start + line.len(), line_num)
                {
                    extracted.push(doc);
                }
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

/// Check if a token scope indicates a comment.
#[allow(dead_code)]
fn is_comment_scope(scope: &str) -> bool {
    scope.contains("comment.") && (scope.contains("line") || scope.contains("block"))
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
    fn test_scope_has_comment() {
        let s1 = "comment_line";
        assert!(!is_comment_scope(s1)); // "comment_line" doesn't match pattern

        let s2 = "comment.line.rust";
        assert!(is_comment_scope(s2)); // "comment.line" matches pattern
    }

    #[test]
    fn test_extract_marker() {
        let e = String::new();
        let result = extract_marker_content(&e, 0, 0, 1);
        assert!(result.is_none());
    }
}
