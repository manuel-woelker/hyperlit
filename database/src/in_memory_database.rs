use crate::Database;
use hyperlit_base::err;
use hyperlit_base::result::HyperlitResult;
use hyperlit_model::segment::{Segment, SegmentId};

#[derive(Debug, Default)]
pub struct InMemoryDatabase {
    pub segments: Vec<Segment>,
}

impl InMemoryDatabase {
    pub fn new() -> Self {
        Self::default()
    }

    fn get_mut_segment_by_id(&mut self, id: SegmentId) -> HyperlitResult<&mut Segment> {
        self.segments
            .get_mut(id as usize)
            .ok_or_else(|| err!("Segment with id {} not found", id))
    }
}

impl Database for InMemoryDatabase {
    fn add_segments(&mut self, segments: Vec<Segment>) -> HyperlitResult<()> {
        let old_len = self.segments.len();
        self.segments.extend(segments);
        let mut index = old_len as u32;
        for segment in self.segments.iter_mut().skip(old_len) {
            segment.id = index;
            index += 1;
        }
        Ok(())
    }

    fn get_segments(&self) -> HyperlitResult<Vec<&Segment>> {
        Ok(self.segments.iter().collect())
    }

    fn get_segment_by_id(&self, id: SegmentId) -> HyperlitResult<&Segment> {
        self.segments
            .get(id as usize)
            .ok_or_else(|| err!("Segment with id {} not found", id))
    }

    fn set_segment_included(&mut self, id: SegmentId) -> HyperlitResult<()> {
        self.get_mut_segment_by_id(id)?.is_included = true;
        Ok(())
    }
}
