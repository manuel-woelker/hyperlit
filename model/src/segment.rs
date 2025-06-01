use crate::location::Location;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct Segment {
    pub id: String,
    pub text: String,
    pub location: Location,
}

impl Segment {
    pub fn new(id: impl Into<String>, text: impl Into<String>, location: Location) -> Segment {
        Segment {
            id: id.into(),
            text: text.into(),
            location,
        }
    }
}