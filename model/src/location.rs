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

    pub fn filepath(&self) -> &str {
        &self.filepath
    }
    
    pub fn line(&self) -> u32 {
        self.line
    }
    
    pub fn column(&self) -> u32 {
        self.column
    }   
}