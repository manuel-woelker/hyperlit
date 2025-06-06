use crate::location::Location;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct Segment {
    pub title: String,
    pub tags: Vec<String>,
    pub text: String,
    pub location: Location,
}

impl Segment {
    pub fn new(
        title: impl Into<String>,
        tags: Vec<String>,
        text: impl Into<String>,
        location: Location,
    ) -> Segment {
        Segment {
            title: title.into(),
            tags,
            text: text.into(),
            location,
        }
    }
}
