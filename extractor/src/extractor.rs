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

use hyperlit_base::error::{bail, err};
use hyperlit_base::result::HyperlitResult;
use hyperlit_model::file_source::FileSource;
use hyperlit_model::location::Location;
use hyperlit_model::segment::Segment;
use std::collections::HashSet;
use std::io::{BufRead, BufReader, Read};
use std::str::{FromStr, from_utf8};
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
    DocComment,
}
const NEWLINE: char = '\n';
const MAXIMUM_LINE_LENGTH: usize = 4096;

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
    doc_comment_markers: &'a HashSet<String>,
    source: &'a dyn FileSource,
    parse_state: ParseState,
    syntax_set: &'a SyntaxSet,
}

impl<'a> FileExtractor<'a> {
    pub fn new(
        source: &'a dyn FileSource,
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

    pub fn extract(&mut self) -> HyperlitResult<Vec<Segment>> {
        let filepath = self.source.filepath()?;
        let mut reader = BufReader::new(Box::new(self.source.open()?));
        let mut segments = Vec::new();
        let mut line_number = 0;
        let mut read_buffer = Vec::with_capacity(MAXIMUM_LINE_LENGTH + 2);
        let mut stack = ScopeStack::new();
        let selectors = Selectors::default();
        let mut state = ExtractorState::Code;
        let relative_filepath = self.source.filepath()?;
        let mut line_complete;
        'for_each_line: loop {
            // Limit line length
            let bytes_read = {
                let mut limited_reader = reader.take(MAXIMUM_LINE_LENGTH as u64);
                read_buffer.clear();
                let bytes_read = limited_reader.read_until(b'\n', &mut read_buffer)?;
                reader = limited_reader.into_inner();
                if bytes_read == MAXIMUM_LINE_LENGTH {
                    bail!(
                        "{filepath}:{line_number} - Line too too long (> {MAXIMUM_LINE_LENGTH} bytes)"
                    );
                }
                line_complete = from_utf8(&read_buffer[0..bytes_read])?;
                bytes_read
            };
            if bytes_read == 0 {
                break 'for_each_line;
            }
            line_number += 1;
            let ops = self
                .parse_state
                .parse_line(line_complete, self.syntax_set)?;
            for (text, op) in ScopeRegionIterator::new(&ops, line_complete) {
                stack.apply(op)?;
                if text.is_empty() {
                    // skip empty strings
                    continue;
                }
                if selectors.comment.does_match(stack.as_slice()).is_some() {
                    // Comment

                    match &mut state {
                        ExtractorState::Code => {
                            let Some((indicator, text_rest)) =
                                text.trim_start().split_once(char::is_whitespace)
                            else {
                                // No whitespace found
                                state = ExtractorState::Comment;
                                continue;
                            };
                            if !self.doc_comment_markers.contains(indicator) {
                                // Not a doc comment
                                state = ExtractorState::Comment;
                                continue;
                            }
                            if let Some(line_rest) = text_rest.strip_prefix("...") {
                                // Found ellipsis -> continue previous segment
                                let line_rest = line_rest.trim();
                                let last_segment: &mut Segment = segments
                                    .last_mut()
                                    .ok_or_else(|| err!("No previous segment"))?;
                                last_segment.text.push_str(line_rest);
                                last_segment.text.push(NEWLINE);
                                last_segment.text.push(NEWLINE);
                            } else {
                                // No ellipsis -> start a new segment
                                let tag_extraction_result = extract_hash_tags(text_rest);
                                segments.push(Segment::new(
                                    0,
                                    0, // TODO: proper file index
                                    tag_extraction_result.text,
                                    tag_extraction_result.tags,
                                    "",
                                    Location::new(&relative_filepath, line_number),
                                ));
                            }
                            state = ExtractorState::DocComment;
                        }
                        ExtractorState::DocComment => {
                            let last_segment = segments.last_mut().unwrap();
                            last_segment.text.push_str(text);
                        }
                        ExtractorState::Comment => {
                            // ignore plain comments
                        }
                    }
                } else {
                    state = ExtractorState::Code;
                }
            }
        }
        Ok(segments)
    }
}

impl Extractor {
    pub fn extract(&self, source: &dyn FileSource) -> HyperlitResult<Vec<Segment>> {
        let filepath = source.filepath()?;
        // get extension
        let extension = filepath
            .rsplit_once('.')
            .ok_or(err!("No extension found in filepath: '{filepath}'"))?
            .1;
        let syntax = self
            .syntax_set
            .find_syntax_by_extension(extension)
            .ok_or_else(|| {
                err!("{filepath} - No syntax definition found for extension '{extension}'")
            })?;
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
    use crate::extractor::{
        Extractor, MAXIMUM_LINE_LENGTH, TagExtractionResult, extract_hash_tags,
    };
    use hyperlit_base::result::HyperlitResult;
    use hyperlit_model::file_source::InMemoryFileSource;
    use hyperlit_model::location::Location;
    use hyperlit_model::segment::Segment;
    use std::collections::HashMap;

    fn create_test_extractor() -> Extractor {
        Extractor::new(&["📖"])
    }

    #[test]
    fn test_extract_hash_tags() -> HyperlitResult<()> {
        let testcases = HashMap::from([
            (
                "#tag",
                TagExtractionResult {
                    tags: vec!["tag".to_string()],
                    text: "".to_string(),
                },
            ),
            (
                "#tag #tag2",
                TagExtractionResult {
                    tags: vec!["tag".to_string(), "tag2".to_string()],
                    text: "".to_string(),
                },
            ),
            (
                "#TAG_FOO",
                TagExtractionResult {
                    tags: vec!["TAG_FOO".to_string()],
                    text: "".to_string(),
                },
            ),
            (
                "alpha #beta gamma #delta epsilon",
                TagExtractionResult {
                    tags: vec!["beta".to_string(), "delta".to_string()],
                    text: "alpha gamma epsilon".to_string(),
                },
            ),
        ]);
        for (input, expected) in testcases {
            let result = extract_hash_tags(input);
            assert_eq!(result, expected, "input: {}", input);
        }
        Ok(())
    }
    #[test]
    fn extract_segment() -> HyperlitResult<()> {
        let extractor = create_test_extractor();
        let result = extractor.extract(&InMemoryFileSource::new(
            "testfile.rs",
            r#"
        /* 📖 The #atag title #btag
This is a test */
        1+2
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
                Location::new("testfile.rs", 2)
            )]
        );
        Ok(())
    }

    #[test]
    fn extract_from_line_comment() -> HyperlitResult<()> {
        let extractor = create_test_extractor();
        let result = extractor.extract(&InMemoryFileSource::new(
            "testfile.rs",
            r#"
        // One
        // 📖 Two
        Three
        // 📖 Four
        1+2
        code // 📖 Five
        "#,
        ))?;
        assert_eq!(
            result,
            vec![
                Segment::new(0, 0, "Two", vec![], "", Location::new("testfile.rs", 3)),
                Segment::new(0, 0, "Four", vec![], "", Location::new("testfile.rs", 5)),
                Segment::new(0, 0, "Five", vec![], "", Location::new("testfile.rs", 7)),
            ]
        );
        Ok(())
    }

    #[test]
    fn extract_from_line_comment_continued() -> HyperlitResult<()> {
        let extractor = create_test_extractor();
        let result = extractor.extract(&InMemoryFileSource::new(
            "testfile.rs",
            r#"
        // One
        // 📖 Two
        Three
        // 📖 ... Four
        1+2
        code // 📖 ... Five
        "#,
        ))?;
        assert_eq!(
            result,
            vec![Segment::new(
                0,
                0,
                "Two",
                vec![],
                "Four\n\nFive\n\n",
                Location::new("testfile.rs", 3)
            ),]
        );
        Ok(())
    }

    #[test]
    fn extract_from_block_comment_continued() -> HyperlitResult<()> {
        let extractor = create_test_extractor();
        let result = extractor.extract(&InMemoryFileSource::new(
            "testfile.rs",
            r#"
        // One
        /* 📖 Two
*/
        Three
        /* 📖 ... Four
*/
        1+2
        code /* 📖 ... Five
*/
        "#,
        ))?;
        assert_eq!(
            result,
            vec![Segment::new(
                0,
                0,
                "Two",
                vec![],
                "Four\n\nFive\n\n",
                Location::new("testfile.rs", 3)
            ),]
        );
        Ok(())
    }

    #[test]
    fn extract_interleaved_block_comment() -> HyperlitResult<()> {
        let extractor = create_test_extractor();
        let result = extractor.extract(&InMemoryFileSource::new(
            "testfile.rs",
            r#"
        /* 📖 One */
        /* Two */
        /* 📖 Three */
        /* 📖 Four */
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
    fn extract_interleaved_block_comment_single_line() -> HyperlitResult<()> {
        let extractor = create_test_extractor();
        let result = extractor.extract(&InMemoryFileSource::new(
            "testfile.rs",
            r#"
        /* 📖 One */         /* Two */        /* 📖 Three */        /* 📖 Four */   /* Five */     "#,
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
            r#"
        // 📖 One
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
            r#"
        /* The #atag title #btag
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
            r#"
        /* 📖 The #atag title #btag
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
                r#"
        /* 📖 The #atag title #btag
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
}
