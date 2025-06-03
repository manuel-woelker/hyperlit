use crate::location::Location;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct Segment {
    pub title: String,
    pub text: String,
    pub location: Location,
}

impl Segment {
    pub fn new(title: impl Into<String>, text: impl Into<String>, location: Location) -> Segment {
        Segment {
            title: title.into(),
            text: text.into(),
            location,
        }
    }
}