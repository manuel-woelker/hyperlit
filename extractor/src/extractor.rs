use hyperlit_base::result::HyperlitResult;
use hyperlit_model::file_source::FileSource;
use hyperlit_model::location::Location;
use hyperlit_model::segment::Segment;
use std::collections::HashSet;
use std::io::{BufRead, BufReader};

pub struct Extractor {
    doc_comment_markers: HashSet<String>,
}

impl Extractor {
    pub fn new(doc_comment_markers: &[&str]) -> Self {
        Self {
            doc_comment_markers: doc_comment_markers.iter().map(|s| s.to_string()).collect(),
        }
    }
}

enum ExtractorState<'a> {
    Code,
    DocComment { segment: &'a mut Segment },
}
const NEWLINE: char = '\n';

impl Extractor {
    pub fn extract(&self, source: &dyn FileSource) -> HyperlitResult<Vec<Segment>> {
        self.extract_impl(source)
    }

    fn extract_impl(&self, source: &dyn FileSource) -> HyperlitResult<Vec<Segment>> {
        let filepath = source.filepath()?;
        let mut reader = BufReader::new(Box::new(source.open()?));
        let mut line_complete = String::new();
        let mut state = ExtractorState::Code;
        let block_comment_start = "/*".to_string();
        let block_comment_end = "*/".to_string();
        let mut segments = Vec::new();
        let mut line_number = 0;
        'for_each_line: loop {
            line_complete.clear();
            let bytes_read = reader.read_line(&mut line_complete)?;
            if bytes_read == 0 {
                break 'for_each_line;
            }
            line_number += 1;
            let mut line_rest = line_complete.as_str();
            while !line_rest.is_empty() {
                match &mut state {
                    ExtractorState::Code => {
                        let Some(comment_start_index) = line_rest.find(&block_comment_start) else {
                            // No comment start found
                            continue 'for_each_line;
                        };
                        let comment_rest = &line_rest
                            [comment_start_index + block_comment_start.len()..]
                            .trim_start();
                        let Some((indicator, line_rest)) =
                            comment_rest.split_once(char::is_whitespace)
                        else {
                            // No whitespace found
                            continue 'for_each_line;
                        };
                        if !self.doc_comment_markers.contains(indicator) {
                            // Not a doc comment
                            continue 'for_each_line;
                        }
                        // Found doc comment start
                        let line_rest = line_rest.trim();
                        let tag_extraction_result = extract_hash_tags(line_rest);
                        let title = tag_extraction_result.text;
                        let segment = Segment::new(
                            segments.len() as u32,
                            title,
                            tag_extraction_result.tags,
                            String::new(),
                            Location::new(&filepath, line_number, comment_start_index as u32),
                        );
                        segments.push(segment);
                        state = ExtractorState::DocComment {
                            segment: segments.last_mut().unwrap(),
                        };
                        continue 'for_each_line;
                    }
                    ExtractorState::DocComment { segment } => {
                        let Some(comment_end_index) = line_rest.find(&block_comment_end) else {
                            // No comment end found, collect the rest of the line
                            segment.text.push_str(line_rest);
                            continue 'for_each_line;
                        };
                        // Found comment end
                        segment.text.push_str(&line_rest[..comment_end_index]);
                        segment.text.push(NEWLINE);
                        line_rest = line_rest[comment_end_index + block_comment_end.len()..].trim();
                        state = ExtractorState::Code;
                    }
                }
            }
        }
        Ok(segments)
    }
}

#[derive(Debug, PartialEq)]
struct TagExtractionResult {
    pub tags: Vec<String>,
    pub text: String,
}

fn extract_hash_tags(input: &str) -> TagExtractionResult {
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
    use crate::extractor::{Extractor, TagExtractionResult, extract_hash_tags};
    use hyperlit_base::result::HyperlitResult;
    use hyperlit_model::file_source::InMemoryFileSource;
    use hyperlit_model::location::Location;
    use hyperlit_model::segment::Segment;
    use std::collections::HashMap;

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
        let extractor = Extractor::new(&["📖"]);
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
                "The title",
                vec!["atag".to_string(), "btag".to_string()],
                "This is a test \n",
                Location::new("testfile.rs", 2, 8)
            )]
        );
        Ok(())
    }

    #[test]
    fn ignore_normal_comments() -> HyperlitResult<()> {
        let extractor = Extractor::new(&["📖"]);
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
}
