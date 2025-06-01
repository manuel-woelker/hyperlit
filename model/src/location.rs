use hyperlit_base::shared_string::SharedString;


#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct Location {
    filepath: SharedString,
    line: u32,
    column: u32,
}

impl Location {
    pub fn new(filepath: impl Into<SharedString>, line: u32, column: u32) -> Location {
        Location {
            filepath: filepath.into(),
            line,
            column,
        }
    }
}