use hyperlit_base::shared_string::SharedString;


#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct Location {
    filepath: SharedString,
    line: u32,
    column: u32,
}