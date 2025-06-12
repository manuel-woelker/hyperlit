use hyperlit_base::result::HyperlitResult;
use hyperlit_model::directive_evaluation::DirectiveEvaluation;
use hyperlit_model::segment::{Segment, SegmentId};
use std::path::Path;
/* 📖 Adding a new output backend #backend #howto

To add a new output backend, you need to implement the `Backend` trait.

See `mdbook_backend.rs` for an example.

 */

/// Parameters for the compilation process
pub trait BackendCompileParams {
    /// Path to the directory containing the documentation files
    fn docs_directory(&self) -> &Path;
    /// Path to the directory where the documentation will be built
    fn build_directory(&self) -> &Path;
    /// Path to the directory where the documentation will be output
    fn output_directory(&self) -> &Path;
    /// Retrieve all segments containing the given tag
    fn evaluate_directive(&self, tag: &str) -> HyperlitResult<DirectiveEvaluation>;

    /// Mark a segment as included in the output
    fn set_segment_included(&mut self, segment_id: SegmentId) -> HyperlitResult<()>;
}

/// An output backend
pub trait Backend {
    /// Perform an (optional)preparation step before files are copied to the build directory
    fn prepare(&mut self, _params: &mut dyn BackendCompileParams) -> HyperlitResult<()> {
        Ok(())
    }
    /// Perform the actual compilation of the documentation
    /// In this step the files in the build_directory should be transformed into the output_directory
    fn compile(&self, params: &dyn BackendCompileParams) -> HyperlitResult<()>;
    /// Transform a given segment into its representation in the backend language (e.g., markdown)
    fn transform_segment(&self, segment: &Segment) -> HyperlitResult<String>;
}

pub type BackendBox = Box<dyn Backend>;
