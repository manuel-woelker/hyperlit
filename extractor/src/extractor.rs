use hyperlit_base::result::HyperlitResult;
use hyperlit_model::file_source::FileSource;
use hyperlit_model::segment::Segment;

pub struct Extractor {
    src: Box<dyn FileSource>,
}

impl Extractor {

    pub fn extract(&self) -> HyperlitResult<Vec<Segment>> {
        Ok(vec![])
    }
}
