use hyperlit_base::result::HyperlitResult;
use hyperlit_model::segment::Segment;

pub trait Database {
    fn add_segments(&mut self, segment: Vec<Segment>) -> HyperlitResult<()>;
    fn get_segments(&self) -> HyperlitResult<Vec<&Segment>>;
}

pub type DatabaseBox = Box<dyn Database>;