use std::collections::HashSet;
use hyperlit_base::result::HyperlitResult;
use hyperlit_model::file_source::FileSource;
use hyperlit_model::segment::Segment;
use std::io::{BufRead, BufReader};
use hyperlit_model::location::Location;

pub struct Extractor {
 //   src: Box<dyn FileSource + 'a>,
    doc_comment_markers: HashSet<String>,
}

impl Extractor {
   /* pub fn new<T: FileSource + 'a>(src: T) -> Extractor<'a> {
        Extractor { src: Box::new(src) }
    }*/
    pub fn new(doc_comment_markers: &[&str]) -> Self {
        Self {
            doc_comment_markers: doc_comment_markers.iter().map(|s| s.to_string()).collect(),
        }
    }
}

enum ExtractorState<'a> {
    Code,
    DocComment {
        segment: &'a mut Segment,
    },
}
const NEWLINE: char = '\n';

impl Extractor{
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
                        let rest = line_rest[comment_start_index + block_comment_start.len()..].trim();
                        if self.doc_comment_markers.contains(rest) {
                            // Found doc comment start
                            let segment = Segment::new(String::new(), String::new(), Location::new(
                                &filepath, line_number, comment_start_index as u32));
                            segments.push(segment);
                            state = ExtractorState::DocComment {
                                segment: segments.last_mut().unwrap(),
                            };
                        }
                        continue 'for_each_line;
                    }
                    ExtractorState::DocComment { segment } => {
                        let Some(comment_end_index) = line_rest.find(&block_comment_end) else {
                            // No comment end found, collect the rest of the line
                            segment.text.push_str(line_rest);
                            segment.text.push(NEWLINE);
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

#[cfg(test)]
mod tests {
    use crate::extractor::Extractor;
    use hyperlit_base::result::HyperlitResult;
    use hyperlit_model::file_source::InMemoryFileSource;
    use hyperlit_model::location::Location;
    use hyperlit_model::segment::Segment;

    #[test]
    fn it_works() -> HyperlitResult<()> {
        let extractor = Extractor::new(&["📖"]);
        let result = extractor.extract(
            &InMemoryFileSource::new(
            "testfile.rs",
            format!(r#"
        /* {}
This is a test */
        1+2
        "#,"📖")
        ))?;
        assert_eq!(
            result,
            vec![Segment::new(
                "",
                "This is a test \n",
                Location::new("testfile.rs", 2, 8)
            )]
        );
        Ok(())
    }
}
