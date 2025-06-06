use crate::Database;
use hyperlit_base::result::HyperlitResult;
use hyperlit_model::segment::Segment;

#[derive(Debug, Default)]
pub struct InMemoryDatabase {
    pub segments: Vec<Segment>,
}

impl InMemoryDatabase {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Database for InMemoryDatabase {
    fn add_segments(&mut self, segments: Vec<Segment>) -> HyperlitResult<()> {
        self.segments.extend(segments);
        Ok(())
    }

    fn get_segments(&self) -> HyperlitResult<Vec<&Segment>> {
        Ok(self.segments.iter().collect())
    }
}
