/* 📖 DR-0002 Use `syntect` to extract doc comments from code #decision #extractor

Status: Approved\
Date: 2025-06-19

### Decision

To extract doc comments from source code, we will use the [syntect](https://crates.io/crates/syntect) crate.

### Context

To extract doc comments from code, we need to find all the comments in the code, for various languages.

The requirements for this extractor were:

1. Wide support for various programming language formats
2. Robustness against invalid code/syntax
3. Good performance

### Consequences

syntect is used to extract doc comments from the code.

To support as many languages as possible, the [two-face](https://crates.io/crates/two-face) crate is used.

### Considered Alternatives

#### Custom lexer

A custom lexer could be implemented to find comments.
Due to the number of languages and the complexity of handling different syntaxes, this might not be a good idea.
Especially handling "comment-like" syntax in strings would potentially mean having a custom lexer for each language.

#### tree-sitter

tree-sitter parsers could be used to extract the comments from source files.

The drawback is that these parsers need to be curated, are platform-specific and are relatively heavyweight.

#### inkjet

[inkjet](https://crates.io/crates/inkjet) bundles ~70 tree-sitter parsers for various languages.

The downside of this approach is that all these parsers need to be compiled (making the compilation much slower) and bundled in the binary (making the binary much larger)
*/

use hyperlit_base::error::err;
use hyperlit_base::result::HyperlitResult;
use std::collections::HashSet;
use std::ops::Range;
use std::str::FromStr;
use syntect::easy::ScopeRegionIterator;
use syntect::highlighting::ScopeSelectors;
use syntect::parsing::{ParseState, ScopeStack, SyntaxSet};

pub struct Extractor {
    syntax_set: SyntaxSet,
    doc_comment_markers: HashSet<String>,
}

impl Extractor {
    pub fn new(doc_comment_markers: &[&str]) -> Self {
        Self {
            doc_comment_markers: doc_comment_markers.iter().map(|s| s.to_string()).collect(),
            syntax_set: two_face::syntax::extra_newlines(),
        }
    }
}

enum ExtractorState {
    Code,
    Comment,
    DocComment { start_offset: usize },
}

#[derive(Debug)]
pub struct ExtractionResult {
    pub byte_range: Range<usize>,
    // TODO: Line numbers
}

#[derive(Debug)]
struct Selectors {
    comment: ScopeSelectors,
}

impl Default for Selectors {
    fn default() -> Selectors {
        Selectors {
            comment: ScopeSelectors::from_str("comment - punctuation").unwrap(),
        }
    }
}

struct FileExtractor<'a> {
    source: &'a str,
    doc_comment_markers: &'a HashSet<String>,
    parse_state: ParseState,
    syntax_set: &'a SyntaxSet,
}

impl<'a> FileExtractor<'a> {
    pub fn new(
        source: &'a str,
        parse_state: ParseState,
        syntax_set: &'a SyntaxSet,
        doc_comment_markers: &'a HashSet<String>,
    ) -> Self {
        Self {
            source,
            parse_state,
            syntax_set,
            doc_comment_markers,
        }
    }

    pub fn extract(&mut self) -> HyperlitResult<Vec<ExtractionResult>> {
        let mut result = Vec::new();
        let mut stack = ScopeStack::new();
        let selectors = Selectors::default();
        let mut state = ExtractorState::Code;
        let mut remaining_source = self.source;
        let mut line_byte_offset = 0usize;
        while !remaining_source.is_empty() {
            let line = if let Some(offset) = remaining_source.find('\n') {
                let line = &remaining_source[..offset + 1];
                remaining_source = &remaining_source[offset + 1..];
                line
            } else {
                let line = remaining_source;
                remaining_source = "";
                line
            };
            let mut byte_offset = line_byte_offset;
            let ops = self.parse_state.parse_line(line, self.syntax_set)?;
            for (text, op) in ScopeRegionIterator::new(&ops, line) {
                stack.apply(op)?;
                if text.is_empty() {
                    // skip empty strings
                    continue;
                }
                if selectors.comment.does_match(stack.as_slice()).is_some() {
                    // Comment

                    match &mut state {
                        ExtractorState::Code => {
                            let Some((indicator, _text_rest)) =
                                text.trim_start().split_once(char::is_whitespace)
                            else {
                                // No whitespace found -> no potential indicator present
                                state = ExtractorState::Comment;
                                continue;
                            };
                            if !self.doc_comment_markers.contains(indicator) {
                                // Not a doc comment
                                state = ExtractorState::Comment;
                                continue;
                            }
                            // Remove leading whitespace
                            byte_offset += text.find(indicator).unwrap();
                            // Remove indicator
                            byte_offset += indicator.len();
                            // Remove trailing whitespace
                            byte_offset += &self.source[byte_offset..]
                                .find(|c: char| !c.is_whitespace())
                                .unwrap();
                            state = ExtractorState::DocComment {
                                start_offset: byte_offset,
                            };
                        }
                        _ => {
                            // ignore other states
                        }
                    }
                } else {
                    if let ExtractorState::DocComment { start_offset } = state {
                        result.push(ExtractionResult {
                            byte_range: start_offset..byte_offset,
                        })
                    }
                    state = ExtractorState::Code;
                }
                byte_offset += text.len();
            }
            line_byte_offset += line.len();
        }
        Ok(result)
    }
}

impl Extractor {
    pub fn extract(&self, source: &str, extension: &str) -> HyperlitResult<Vec<ExtractionResult>> {
        let syntax = self
            .syntax_set
            .find_syntax_by_extension(extension)
            .ok_or_else(|| err!("No syntax definition found for extension '{extension}'"))?;
        let parse_state = ParseState::new(syntax);
        let mut file_extractor = FileExtractor::new(
            source,
            parse_state,
            &self.syntax_set,
            &self.doc_comment_markers,
        );
        file_extractor.extract()
    }
}

#[derive(Debug, PartialEq)]
pub struct TagExtractionResult {
    pub tags: Vec<String>,
    pub text: String,
}

pub fn extract_hash_tags(input: &str) -> TagExtractionResult {
    let mut tags = vec![];
    let mut text = String::new();
    let words = input.split_whitespace().collect::<Vec<_>>();
    for word in words {
        if let Some(tag) = word.strip_prefix("#") {
            tags.push(tag.to_string());
        } else {
            if !text.is_empty() {
                text.push(' ');
            }
            text.push_str(word);
        }
    }
    TagExtractionResult { tags, text }
}

#[cfg(test)]
mod tests {
    use crate::extractor::Extractor;
    use expect_test::{Expect, expect};
    use hyperlit_base::result::HyperlitResult;

    fn create_test_extractor() -> Extractor {
        Extractor::new(&["📖", "DOC", "DOC:", "DOCS", "DOCS:"])
    }

    fn run_test(extension: &str, source: &str, expect: Expect) -> HyperlitResult<()> {
        let extractor = create_test_extractor();
        let result = extractor.extract(source, extension)?;
        let snippets: Vec<_> = result
            .iter()
            .map(|extraction| &source[extraction.byte_range.clone()])
            .collect();
        expect.assert_debug_eq(&snippets);
        Ok(())
    }

    #[test]
    fn extract_from_rust_source() -> HyperlitResult<()> {
        run_test(
            "rs",
            r#"
        use foo;
        /* 📖 The #atag title #btag
This is a test *//* should not be in doc comment */
        1+2
        // 📖 Just a line comment
        3+4
        "#,
            expect![[r#"
                [
                    "The #atag title #btag\nThis is a test ",
                    "Just a line comment\n",
                ]
            "#]],
        )
    }
    #[test]
    fn extract_interleaved_block_comment() -> HyperlitResult<()> {
        run_test(
            "rs",
            r#" /* 📖 One */
    /* doc Should not be extracted */
    /* DOC Two */
    /* DOCS: Three */
    "#,
            expect![[r#"
                [
                    "One */\n   ",
                    "Two */\n  ",
                    "Three */\n    ",
                ]
            "#]],
        )
    }
    /*
    #[test]
    fn extract_interleaved_block_comment_single_line() -> HyperlitResult<()> {
        let extractor = create_test_extractor();
        let result = extractor.extract(&InMemoryFileSource::new(
            "testfile.rs",
            r#" /* 📖 One */
    /* Two */
    /* 📖 Three */
    /* 📖 Four */
    /* Five */
    "#,
        ))?;
        assert_eq!(
            result,
            vec![
                Segment::new(0, 0, "One", vec![], "", Location::new("testfile.rs", 2)),
                Segment::new(0, 0, "Three", vec![], "", Location::new("testfile.rs", 2)),
                Segment::new(0, 0, "Four", vec![], "", Location::new("testfile.rs", 2)),
            ]
        );
        Ok(())
    }

    #[test]
    fn extract_interleaved_line_comment() -> HyperlitResult<()> {
        let extractor = create_test_extractor();
        let result = extractor.extract(&InMemoryFileSource::new(
            "testfile.rs",
            r#" // 📖 One
    // Two
    // 📖 Three
    // 📖 Four
    "#,
        ))?;
        assert_eq!(
            result,
            vec![
                Segment::new(0, 0, "One", vec![], "", Location::new("testfile.rs", 2)),
                Segment::new(0, 0, "Three", vec![], "", Location::new("testfile.rs", 4)),
                Segment::new(0, 0, "Four", vec![], "", Location::new("testfile.rs", 5)),
            ]
        );
        Ok(())
    }

    #[test]
    fn ignore_normal_comments() -> HyperlitResult<()> {
        let extractor = create_test_extractor();
        let result = extractor.extract(&InMemoryFileSource::new(
            "testfile.rs",
            r#" /* The #atag title #btag
    This is a test */
    1+2
        "#,
        ))?;
        assert_eq!(result, vec![]);
        Ok(())
    }

    #[test]
    fn ignore_comments_in_strings() -> HyperlitResult<()> {
        let extractor = create_test_extractor();
        let result = extractor.extract(&InMemoryFileSource::new(
            "testfile.rs",
            r#"
        "/* 📖 The #atag title #btag
This is a test */"
        b"/* 📖 The #atag title #btag
This is a test */"

        "#,
        ))?;
        assert_eq!(result, vec![]);
        Ok(())
    }

    #[test]
    fn test_sass() -> HyperlitResult<()> {
        let extractor = create_test_extractor();
        let result = extractor.extract(&InMemoryFileSource::new(
            "testfile.sass",
            r#" /* 📖 The #atag title #btag
    This is a test */
    "#,
        ))?;
        assert_eq!(
            result,
            vec![Segment::new(
                0,
                0,
                "The title",
                vec!["atag".to_string(), "btag".to_string()],
                "This is a test ",
                Location::new("testfile.sass", 2)
            )]
        );
        Ok(())
    }

    #[test]
    fn test_unknown_filetype() -> HyperlitResult<()> {
        let extractor = create_test_extractor();
        let result = extractor
            .extract(&InMemoryFileSource::new(
                "testfile.unknown",
                r#" /* 📖 The #atag title #btag
    This is a test */
    "#,
            ))
            .expect_err("unknown filetype should fail");
        assert_eq!(
            result.to_string(),
            "testfile.unknown - No syntax definition found for extension 'unknown'"
        );
        Ok(())
    }

    #[test]
    fn bail_line_too_long() -> HyperlitResult<()> {
        let extractor = create_test_extractor();
        let long_line = "a".repeat(MAXIMUM_LINE_LENGTH + 1);
        let result = extractor
            .extract(&InMemoryFileSource::new("testfile.java", long_line))
            .expect_err("too long line should fail");
        assert_eq!(
            result.to_string(),
            "testfile.java:0 - Line too too long (> 4096 bytes)"
        );
        Ok(())
    }

    #[test]
    fn bail_line_too_long_multibyte_char() -> HyperlitResult<()> {
        let extractor = create_test_extractor();
        let mut long_line = "a".repeat(MAXIMUM_LINE_LENGTH - 1);
        // put a multibyte char at the maximum line length boundary so that the resulting buffer is not valid UTF-8
        long_line += "📖";
        let result = extractor
            .extract(&InMemoryFileSource::new("testfile.java", long_line))
            .expect_err("too long line should fail");
        assert_eq!(
            result.to_string(),
            "testfile.java:0 - Line too too long (> 4096 bytes)"
        );
        Ok(())
    }
    
 */
}
