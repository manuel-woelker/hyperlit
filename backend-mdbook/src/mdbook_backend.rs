use hyperlit_base::error::HyperlitError;
use hyperlit_base::result::HyperlitResult;
use hyperlit_model::backend::{Backend, BackendCompileParams};
use hyperlit_model::segment::Segment;
use mdbook::MDBook;

pub struct MdBookBackend {}

impl MdBookBackend {
    pub fn new() -> Self {
        Self {}
    }
}

impl Backend for MdBookBackend {
    fn compile(&self, params: &BackendCompileParams) -> HyperlitResult<()> {
        (|| -> mdbook::errors::Result<()> {
            let mut book = MDBook::load(&params.build_directory)?;
            book.config.build.build_dir = params.output_directory.clone();
            book.build()?;
            Ok(())
        })()
        .map_err(|e| HyperlitError::from_boxed(e.into_boxed_dyn_error()))?;
        Ok(())
    }

    fn transform_segment(&self, segment: &Segment) -> HyperlitResult<String> {
        let title = segment.title.as_str();
        let text = segment.text.as_str();
        let line = segment.location.line();
        let filepath = segment.location.filepath();
        let tags = segment.tags.iter().fold(String::new(), |mut acc, tag| {
            acc.push_str(" *#");
            acc.push_str(&tag);
            acc.push_str("*");
            acc
        });
        let result_text = format!(
            "## {title}\n\n<span class=\"tags\">{tags}</span>\n\n{text}\n\n`{filepath}:{line}`\n\n"
        );
        Ok(result_text)
    }
}

#[cfg(test)]
mod tests {
    use hyperlit_base::result::HyperlitResult;
    use hyperlit_base::shared_string::SharedString;
    use hyperlit_model::backend::Backend;
    use hyperlit_model::location::Location;
    use hyperlit_model::segment::Segment;

    #[test]
    fn transform_segment() -> HyperlitResult<()> {
        let segment = Segment::new(
            "<title>",
            vec!["atag".to_string(), "btag".to_string()],
            "<text>",
            Location::new(SharedString::from("<filepath>"), 42, 99),
        );
        let backend = super::MdBookBackend::new();
        assert_eq!(
            backend.transform_segment(&segment)?,
            "## <title>\n\n<span class=\"tags\"> *#atag* *#btag*</span>\n\n<text>\n\n`<filepath>:42`\n\n"
        );
        Ok(())
    }
}
