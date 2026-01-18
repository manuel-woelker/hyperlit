use hyperlit_base::FilePath;
use hyperlit_base::shared_string::SharedString;
use std::ops::Range;

#[derive(Debug)]
pub struct DocumentData {
    pub id: SharedString,
    pub title: SharedString,
    pub file: FilePath,
    pub byte_range: Option<Range<usize>>,
}
