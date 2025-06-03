use std::path::PathBuf;
use hyperlit_base::result::HyperlitResult;
use crate::segment::Segment;

pub struct BackendCompileParams {
    pub build_directory: PathBuf,
    pub output_directory: PathBuf,
}

pub trait Backend {
    fn compile(&self, params: &BackendCompileParams) -> HyperlitResult<()>;
    fn transform_segment(&self, segment: &Segment) -> HyperlitResult<String>;
}