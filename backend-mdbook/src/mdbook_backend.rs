/* 📖 DR-0001 Use mdbook as first backend to generate documentation #decision #backend

Status: Approved\
Date: 2025-06-08

### Decision

To generate HTML documentation, we will use [mdbook](https://rust-lang.github.io/mdBook/) as the first backend.

### Context

The goal of hyperlit is to create documentation artifacts that are straightforward to read and understand.

The requirements for the initial backend were:

1. Easy to integrate
2. Support for Markdown
3. HTML output
3. Good search capabilities

### Consequences

mdbook is the "default" backend for hyperlit.

This backend should be maintained and supported in the future.

### Considered Alternatives

#### Custom backend

A custom backend could be implemented, but it would be more complex to create, maintain and update.

Such a backend may make sense in the future to better support more complex use cases.

#### typst backend

[typst](https://github.com/typst/typst) might also work as a backend.

However, it is currently not well established. Its HTML export functionality is still a work in progress.

*/

use hyperlit_base::error::HyperlitError;
use hyperlit_base::result::HyperlitResult;
use hyperlit_model::backend::{Backend, BackendCompileParams};
use hyperlit_model::segment::Segment;
use mdbook::MDBook;

#[derive(Default)]
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
            acc.push_str(tag);
            acc.push('*');
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
