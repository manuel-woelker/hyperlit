#[derive(Clone)]
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
}
