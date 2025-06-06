use crate::segment::Segment;
use hyperlit_base::result::HyperlitResult;
use std::path::PathBuf;

/* 📖 Adding a new output backend #backend #howto

To add a new output backend, you need to implement the `Backend` trait.

See `mdbook_backend.rs` for an example.

 */

pub struct BackendCompileParams {
    pub build_directory: PathBuf,
    pub output_directory: PathBuf,
}

pub trait Backend {
    fn compile(&self, params: &BackendCompileParams) -> HyperlitResult<()>;
    fn transform_segment(&self, segment: &Segment) -> HyperlitResult<String>;
}

pub type BackendBox = Box<dyn Backend>;
