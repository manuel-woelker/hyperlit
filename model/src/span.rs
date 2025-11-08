#[derive(Clone, Debug, Default)]
pub struct Span {
    pub file_index: usize,
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(file_index: usize, start: usize, end: usize) -> Span {
        Span {
            file_index,
            start,
            end,
        }
    }

    pub fn as_sloc(&self) -> String {
        format!("{}:{}-{}", self.file_index, self.start, self.end)
    }
}
