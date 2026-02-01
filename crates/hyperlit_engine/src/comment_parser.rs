/// Represents an extracted comment
///
/// This struct captures the location and content of documentation markers
/// within source code comments.
#[derive(Debug, Clone)]
pub struct ExtractedComment {
    /// The documentation content (markdown) after the document marker
    pub content: String,
    /// Starting line number (1-indexed)
    pub start_line: usize,
    /// Starting byte offset in the file
    pub start_byte: usize,
    /// Ending byte offset in the file
    pub end_byte: usize,
}

use hyperlit_base::{HyperlitResult, bail};
use std::collections::HashSet;
use std::str::FromStr;
use syntect::easy::ScopeRegionIterator;
use syntect::highlighting::ScopeSelectors;

/// Parses code comments from source files using syntect for language detection.
pub struct CommentParser {
    syntax_set: syntect::parsing::SyntaxSet,
    comment_selector: ScopeSelectors,
    punctuation_selector: ScopeSelectors,
    /// Documentation markers to look for in comments (e.g., "📖", "DOC:", "DOCS:", "HINT:")
    doc_comment_markers: HashSet<String>,
}

impl CommentParser {
    /// Create a new comment parser with built-in syntax definitions and default markers.
    ///
    /// Default markers: 📖 (emoji), DOC:, DOCS:, HINT:, NOTE:, INFO:
    pub fn new() -> Self {
        Self::with_markers(["📖", "DOC:", "DOCS:", "HINT:", "NOTE:", "INFO:"])
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
    pub fn with_markers(markers: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let syntax_set = syntect::parsing::SyntaxSet::load_defaults_newlines();
        // Create a scope selector that matches comment scopes but excludes punctuation
        // This gives us the comment content without the comment delimiters (// # /* etc.)
        let comment_selector =
            ScopeSelectors::from_str("comment").expect("Failed to create comment scope selector");
        let punctuation_selector = ScopeSelectors::from_str("punctuation")
            .expect("Failed to create comment punctuation scope selector");
        Self {
            syntax_set,
            comment_selector,
            punctuation_selector,
            doc_comment_markers: markers.into_iter().map(|s| s.into()).collect(),
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
    ) -> HyperlitResult<Vec<ExtractedComment>> {
        // Get the syntax definition for this file extension
        let syntax = match self.get_syntax_for_extension(file_extension) {
            Some(s) => s,
            None => {
                bail!("Unknown extension: {}", file_extension)
            }
        };
        // Build scope stack to track which parts are comments
        let mut scope_stack = syntect::parsing::ScopeStack::new();

        let mut extracted = Vec::new();
        let mut parse_state = syntect::parsing::ParseState::new(&syntax);
        let mut current_byte = 0;
        #[derive(Debug)]
        enum ExtractorState {
            Code,
            DocComment(ExtractedComment),
            PlainComment,
        }
        let mut extractor_state = ExtractorState::Code;
        for (line_idx, line) in content.split_inclusive('\n').enumerate() {
            let line_num = line_idx + 1;
            let line_start_byte = current_byte;

            // Parse this line with syntect to get scope operations
            let ops = parse_state.parse_line(line, &self.syntax_set)?;

            'inner: for (text, op) in ScopeRegionIterator::new(&ops, line) {
                // Apply the scope operation to our stack
                scope_stack.apply(op)/*TODO: .with_context(format!("Error applying op in line {line_num}"))?*/?;
                if text.is_empty() {
                    // skip empty strings
                    continue;
                }
                let end_byte = current_byte + text.len();
                // Check if current scope matches comment selector (comment without punctuation)
                let is_in_comment = self
                    .comment_selector
                    .does_match(scope_stack.as_slice())
                    .is_some();
                if is_in_comment {
                    if self
                        .punctuation_selector
                        .does_match(scope_stack.as_slice())
                        .is_some()
                    {
                        current_byte += text.len();
                        continue 'inner;
                    }
                    match &mut extractor_state {
                        ExtractorState::Code => 'code: {
                            let Some((indicator, text_rest)) =
                                text.trim_start().split_once(char::is_whitespace)
                            else {
                                // No whitespace found -> no potential indicator present
                                extractor_state = ExtractorState::PlainComment;
                                break 'code;
                            };
                            if !self.doc_comment_markers.contains(indicator) {
                                // Not a doc comment
                                extractor_state = ExtractorState::PlainComment;
                                break 'code;
                            }
                            let doc_comment = text_rest.trim_start();
                            let start_byte = line_start_byte + doc_comment.as_ptr() as usize
                                - line.as_ptr() as usize;
                            let content = doc_comment.to_string();
                            extractor_state = ExtractorState::DocComment(ExtractedComment {
                                start_byte,
                                end_byte,
                                start_line: line_num,
                                content,
                            });
                        }
                        ExtractorState::DocComment(doc_comment) => {
                            doc_comment
                                .content
                                .push_str(text.strip_prefix(" ").unwrap_or(text));
                            doc_comment.end_byte = end_byte;
                        }
                        ExtractorState::PlainComment => {
                            // ignore
                        }
                    }
                } else if !text.trim().is_empty() {
                    // When text is whitespace only, keep the state in order to merge line comments
                    if let ExtractorState::DocComment(doc_comment) = extractor_state {
                        extracted.push(doc_comment);
                    }
                    extractor_state = ExtractorState::Code;
                }
                current_byte += text.len();
            }

            current_byte = line_start_byte + line.len() + 1; // Account for newline
        }
        // handle last comment
        if let ExtractorState::DocComment(doc_comment) = extractor_state {
            extracted.push(doc_comment);
        }
        Ok(extracted)
    }
}

impl Default for CommentParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;

    /// 📖 # Why a macro for testing comment extraction?
    /// Testing comment extraction requires comparing complex structured output
    /// against expected snapshots. This macro provides a declarative way to:
    /// - Specify input source code and file extension
    /// - Define expected extracted comments via expect-test snapshots
    /// - Handle both successful extractions and expected errors
    macro_rules! assert_extracted_comments {
        ($source:expr, $ext:expr, $expected:expr) => {{
            let parser = CommentParser::new();
            let result = parser.extract_doc_comments($source, $ext);

            let actual = match result {
                Ok(comments) => {
                    if comments.is_empty() {
                        "(no comments extracted)".to_string()
                    } else {
                        comments
                            .iter()
                            .map(|c| format!("line {}:\n{}", c.start_line, c.content))
                            .collect::<Vec<_>>()
                            .join("\n---\n")
                    }
                }
                Err(e) => format!("Error: {}", e),
            };

            $expected.assert_eq(&actual);
        }};
    }

    #[test]
    fn test_extract_rust_doc_comment() {
        assert_extracted_comments!(
            r#"fn main() {
    // 📖 This is documentation
    println!("hello");
}"#,
            "rs",
            expect![[r#"
                line 2:
                This is documentation
            "#]]
        );
    }

    #[test]
    fn test_extract_rust_doc_comment_multiline() {
        assert_extracted_comments!(
            r#"fn main() {
    // 📖 This is documentatixon
    // This should be in there as well
    println!("hello");
}"#,
            "rs",
            expect![[r#"
                line 2:
                This is documentatixon
                This should be in there as well
            "#]]
        );
    }

    #[test]
    fn test_extract_multiple_markers() {
        assert_extracted_comments!(
            r#"// 📖 First doc comment
// 📖 Second doc comment
fn foo() {}

// HINT: This is a hint
// NOTE: This is a note
// INFO: This is info
// DOC: This uses DOC marker
// DOCS: This uses DOCS marker"#,
            "rs",
            expect![[r#"
                line 1:
                First doc comment
                📖 Second doc comment

                ---
                line 5:
                This is a hint
                NOTE: This is a note
                INFO: This is info
                DOC: This uses DOC marker
                DOCS: This uses DOCS marker"#]]
        );
    }

    #[test]
    fn test_block_comment_markers() {
        assert_extracted_comments!(
            r#"fn foo() {
/* 📖 Block comment docs
Second line */
println!("hello");
}"#,
            "rs",
            expect![[r#"
                line 2:
                Block comment docs
                Second line "#]]
        );
    }

    #[test]
    fn test_no_comments_extracted() {
        assert_extracted_comments!(
            r#"fn main() {
    // This is a regular comment
    println!("hello");
}"#,
            "rs",
            expect!["(no comments extracted)"]
        );
    }

    #[test]
    fn test_unknown_extension() {
        assert_extracted_comments!(
            "// 📖 some doc",
            "xyz",
            expect!["Error: Unknown extension: xyz"]
        );
    }

    #[test]
    fn test_doc_in_string_not_extracted() {
        assert_extracted_comments!(
            r#"fn main() {
    let s = "📖 This is not a doc";
}"#,
            "rs",
            expect!["(no comments extracted)"]
        );
    }

    #[test]
    fn test_python_doc_comments() {
        assert_extracted_comments!(
            r#"def foo():
    # 📖 This is Python documentation
    pass

    # 📖 Another doc comment
    # with multiple lines
    x = 1"#,
            "py",
            expect![[r#"
                line 2:
                This is Python documentation

                ---
                line 5:
                Another doc comment
                with multiple lines
            "#]]
        );
    }

    #[test]
    fn test_javascript_doc_comments() {
        assert_extracted_comments!(
            r#"function foo() {
    // 📖 JavaScript documentation
    return 42;
}

/* 📖 Block comment
   documentation */"#,
            "js",
            expect![[r#"
                line 2:
                JavaScript documentation

                ---
                line 6:
                Block comment
                  documentation "#]]
        );
    }

    #[test]
    fn test_doc_marker_without_space() {
        assert_extracted_comments!(
            "// 📖No space after marker",
            "rs",
            expect!["(no comments extracted)"]
        );
    }
}
