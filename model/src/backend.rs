use crate::segment::{Segment, SegmentId};
use hyperlit_base::result::HyperlitResult;
use std::path::Path;

/* 📖 Adding a new output backend #backend #howto

To add a new output backend, you need to implement the `Backend` trait.

See `mdbook_backend.rs` for an example.

 */

pub trait BackendCompileParams {
    fn docs_directory(&self) -> &Path;
    fn build_directory(&self) -> &Path;
    fn output_directory(&self) -> &Path;
    fn get_segments_by_tag(&self, tag: &str) -> HyperlitResult<Vec<&Segment>>;
    fn set_segment_included(&mut self, segment_id: SegmentId) -> HyperlitResult<()>;
}

pub trait Backend {
    fn prepare(&mut self, _params: &mut dyn BackendCompileParams) -> HyperlitResult<()> {
        Ok(())
    }
    fn compile(&self, params: &dyn BackendCompileParams) -> HyperlitResult<()>;
    fn transform_segment(&self, segment: &Segment) -> HyperlitResult<String>;
}

pub type BackendBox = Box<dyn Backend>;
