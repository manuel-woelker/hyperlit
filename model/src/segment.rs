use crate::last_modification_info::LastModificationInfo;
use crate::location::Location;

pub type SegmentId = u32;

#[derive(Debug, Clone, PartialEq)]
pub struct Segment {
    /// document-wide unique identifier
    pub id: SegmentId,
    /// File index
    pub file_index: usize,
    /// title of this segment
    pub title: String,
    /// tags of this segment
    pub tags: Vec<String>,
    /// content text of this segment - may be markdown formatted
    pub text: String,
    /// location of this segment
    pub location: Location,
    /// whether the segment is already included in the output
    pub location_url: Option<String>,
    /// url of this location (e.g. in github, etc)
    pub is_included: bool,
    /// last modification info, usually from git
    pub last_modification: LastModificationInfo,
}

impl Segment {
    pub fn new(
        id: SegmentId,
        file_index: usize,
        title: impl Into<String>,
        tags: Vec<String>,
        text: impl Into<String>,
        location: Location,
    ) -> Segment {
        Segment {
            id,
            file_index,
            title: title.into(),
            tags,
            text: text.into(),
            location,
            is_included: false,
            last_modification: LastModificationInfo::default(),
            location_url: None,
        }
    }
}

pub fn segments_sort_by_title(segments: &mut [&Segment]) {
    segments.sort_by(|a, b| a.title.cmp(&b.title));
}
