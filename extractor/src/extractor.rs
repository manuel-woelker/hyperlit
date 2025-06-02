use hyperlit_base::result::HyperlitResult;
use hyperlit_model::file_source::FileSource;
use hyperlit_model::segment::Segment;
use std::io::{BufRead, BufReader};
use hyperlit_model::location::Location;

pub struct Extractor<'a> {
    src: Box<dyn FileSource + 'a>,
}

impl <'a> Extractor<'a> {
    pub fn new<T: FileSource + 'a>(src: T) -> Extractor<'a> {
        Extractor { src: Box::new(src) }
    }
}

enum ExtractorState<'a> {
    Code,
    DocComment {
        segment: &'a mut Segment,
    },
}
const NEWLINE: char = '\n';

impl Extractor<'_> {
    pub fn extract(&self) -> HyperlitResult<Vec<Segment>> {
        let filepath = self.src.filepath()?;
        let mut reader = BufReader::new(self.src.open()?);
        let mut line_complete = String::new();
        let mut state = ExtractorState::Code;
        let block_comment_start = "/* DOC:".to_string();
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
                        let segment = Segment::new(rest, String::new(), Location::new(
                            &filepath, line_number, comment_start_index as u32));
                        segments.push(segment);
                        state = ExtractorState::DocComment {
                            segment: segments.last_mut().unwrap(),
                        };
                        continue 'for_each_line;
                    }
                    ExtractorState::DocComment { segment } => {
                        let Some(comment_end_index) = line_rest.find(&block_comment_end) else {
                            // No comment end found, collect the rest of the line
                            segment.text.push_str(line_rest.trim());
                            segment.text.push(NEWLINE);
                            continue 'for_each_line;
                        };
                        // Found comment end
                        segment.text.push_str(&line_rest[..comment_end_index].trim());
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
        let extractor = Extractor::new(InMemoryFileSource::new(
            "testfile.rs",
            r#"
        /* DOC:testing

        This is a test */
        1+2
        "#,
        ));
        let result = extractor.extract()?;
        assert_eq!(
            result,
            vec![Segment::new(
                "testing",
                "\n        This is a test \n",
                Location::new("testfile.rs", 2, 8)
            )]
        );
        Ok(())
    }
}
