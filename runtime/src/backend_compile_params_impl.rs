use crate::evaluate_directive::evaluate_directive;
use hyperlit_backend::backend::BackendCompileParams;
use hyperlit_base::result::HyperlitResult;
use hyperlit_model::database::Database;
use hyperlit_model::directive_evaluation::DirectiveEvaluation;
use hyperlit_model::segment::SegmentId;
use std::path::Path;

pub struct BackendCompileParamsImpl<'a> {
    docs_directory: &'a Path,
    build_directory: &'a Path,
    output_directory: &'a Path,
    database: &'a mut dyn Database,
}

impl<'a> BackendCompileParamsImpl<'a> {
    pub fn new(
        docs_directory: &'a Path,
        build_directory: &'a Path,
        output_directory: &'a Path,
        database: &'a mut dyn Database,
    ) -> Self {
        Self {
            docs_directory,
            build_directory,
            output_directory,
            database,
        }
    }
}

impl BackendCompileParams for BackendCompileParamsImpl<'_> {
    fn docs_directory(&self) -> &Path {
        self.docs_directory
    }

    fn build_directory(&self) -> &Path {
        self.build_directory
    }

    fn output_directory(&self) -> &Path {
        self.output_directory
    }

    fn evaluate_directive(&self, directive: &str) -> HyperlitResult<DirectiveEvaluation> {
        evaluate_directive(directive, self.database)
    }

    fn set_segment_included(&mut self, segment_id: SegmentId) -> HyperlitResult<()> {
        self.database.set_segment_included(segment_id)
    }
}
