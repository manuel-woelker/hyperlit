use hyperlit_base::result::HyperlitResult;
use hyperlit_model::segment::{Segment, SegmentId};

pub trait Database {
    fn add_segments(&mut self, segment: Vec<Segment>) -> HyperlitResult<()>;
    fn get_segments(&self) -> HyperlitResult<Vec<&Segment>>;
    fn get_segment_by_id(&self, id: SegmentId) -> HyperlitResult<&Segment>;
    fn set_segment_included(&mut self, id: SegmentId) -> HyperlitResult<()>;
}

pub type DatabaseBox = Box<dyn Database>;
