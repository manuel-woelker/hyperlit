use crate::segment::{Segment, SegmentId};
use hyperlit_base::result::HyperlitResult;

pub trait Database: Send + Sync + 'static {
    fn add_segments(&mut self, segment: Vec<Segment>) -> HyperlitResult<()>;

    fn get_all_segments(&self) -> HyperlitResult<Vec<&Segment>>;
    fn get_segment_by_id(&self, id: SegmentId) -> HyperlitResult<&Segment>;

    fn get_segments_by_tag(&self, tag: &str) -> HyperlitResult<Vec<&Segment>> {
        Ok(self
            .get_all_segments()?
            .into_iter()
            .filter(|segment| segment.tags.iter().any(|segment_tag| segment_tag == tag))
            .collect())
    }

    fn set_segment_included(&mut self, id: SegmentId) -> HyperlitResult<()>;
}

pub type DatabaseBox = Box<dyn Database>;
