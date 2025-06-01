use mdbook::MDBook;
use hyperlit_base::error::{HyperlitError};
use hyperlit_base::result::HyperlitResult;
use hyperlit_model::backend::{Backend, BackendCompileParams};

pub struct MdBookBackend {

}

impl MdBookBackend {
    pub fn new() -> Self {
        Self {}
    }
}

impl Backend for MdBookBackend {
    fn compile(&self, params: &BackendCompileParams) -> HyperlitResult<()> {
        (|| -> mdbook::errors::Result<()> {
            dbg!(&params.build_directory);
            let mut book = MDBook::load(&params.build_directory)?;
            dbg!(&book.config);
            book.config.build.build_dir = params.output_directory.clone();
            book.build()?;
            let output_directory = params.output_directory.clone();
            Ok(())
        })().map_err(|e| HyperlitError::from_boxed(e.into_boxed_dyn_error()))?;
        Ok(())
    }
}