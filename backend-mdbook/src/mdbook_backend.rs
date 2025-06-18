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
use hyperlit_backend::backend::{Backend, BackendCompileParams};
use hyperlit_base::error::HyperlitError;
use hyperlit_base::result::HyperlitResult;
use hyperlit_model::directive_evaluation::DirectiveEvaluation;
use hyperlit_model::segment::Segment;
use mdbook::MDBook;
use mdbook::book::Link;
use mdbook::book::{Summary, SummaryItem, parse_summary};
use std::borrow::Cow;
use std::fs::{File, create_dir_all, read_to_string};
use std::io::Write;
use std::mem::take;

const SUMMARY_PATH: &str = "src/SUMMARY.md";

#[derive(Default)]
pub struct MdBookBackend {
    summary: Summary,
}

impl MdBookBackend {
    pub fn new() -> Self {
        Self {
            summary: Summary::default(),
        }
    }
}

impl Backend for MdBookBackend {
    fn prepare(&mut self, params: &mut dyn BackendCompileParams) -> HyperlitResult<()> {
        let mut summary = (|| -> mdbook::errors::Result<Summary> {
            let summary_string = read_to_string(params.docs_directory().join(SUMMARY_PATH))?;
            parse_summary(&summary_string)
        })()
        .map_err(|e| HyperlitError::from_boxed(e.into_boxed_dyn_error()))?;
        self.transform_summary_items(&mut summary.numbered_chapters, params)?;
        self.summary = summary;
        Ok(())
    }

    fn compile(&self, params: &dyn BackendCompileParams) -> HyperlitResult<()> {
        self.write_summary_file(&self.summary, &params.build_directory().join(SUMMARY_PATH))?;
        (|| -> mdbook::errors::Result<()> {
            let mut book = MDBook::load(params.build_directory())?;
            book.config.build.build_dir = params.output_directory().to_path_buf();
            book.build()?;
            Ok(())
        })()
        .map_err(|e| {
            dbg!(&e);
            HyperlitError::from_boxed(e.into_boxed_dyn_error())
        })?;
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
        let modification = format!(
            "{} {}",
            segment
                .last_modification
                .author
                .as_ref()
                .map_or("", |s| s.as_str()),
            segment
                .last_modification
                .date
                .as_ref()
                .map_or("".to_string(), |timestamp| timestamp.to_rfc3339())
        );
        let result_text = format!(
            "## {title}\n\n<span class=\"tags\">{tags}</span> {modification}\n\n{text}\n\n`{filepath}:{line}`\n\n"
        );
        Ok(result_text)
    }
}

impl MdBookBackend {
    fn transform_summary_items(
        &self,
        summary_items: &mut Vec<SummaryItem>,
        params: &mut dyn BackendCompileParams,
    ) -> HyperlitResult<()> {
        let mut included_segment_ids = Vec::new();
        create_dir_all(params.build_directory().join("src"))?;
        let old_summary_items = take(summary_items);
        for summary_item in old_summary_items {
            match summary_item {
                SummaryItem::Link(mut link) => {
                    let linkname = link.name.trim();
                    let evaluation = params.evaluate_directive(linkname)?;
                    match evaluation {
                        DirectiveEvaluation::Segments { segments } => {
                            for segment in segments {
                                let output_path = params
                                    .build_directory()
                                    .join(format!("src/{}.md", segment.title));
                                included_segment_ids.push(segment.id);
                                let mut output_file = File::create(output_path)?;
                                output_file
                                    .write_all(self.transform_segment(segment)?.as_bytes())?;
                                let link = Link::new(
                                    segment.title.to_string(),
                                    format!("{}.md", segment.title).replace(" ", "%20"),
                                );
                                summary_items.push(SummaryItem::Link(link));
                            }
                            continue;
                        }
                        DirectiveEvaluation::NoDirective => {
                            self.transform_summary_items(&mut link.nested_items, params)?;
                            summary_items.push(SummaryItem::Link(link));
                        }
                    }
                }
                other => {
                    summary_items.push(other);
                }
            }
        }
        for segment_id in included_segment_ids {
            params.set_segment_included(segment_id)?;
        }
        Ok(())
    }

    fn write_summary_file(
        &self,
        summary: &Summary,
        summary_path: &std::path::Path,
    ) -> HyperlitResult<()> {
        let mut file = File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .open(summary_path)?;
        writeln!(file, "# Summary\n")?;
        write_summary_items(&mut file, &summary.prefix_chapters, "")?;
        writeln!(file)?;
        write_summary_items(&mut file, &summary.numbered_chapters, "- ")?;
        writeln!(file)?;
        write_summary_items(&mut file, &summary.suffix_chapters, "")?;
        Ok(())
    }
}

fn write_summary_items(
    file: &mut File,
    summary_items: &Vec<SummaryItem>,
    prefix: &str,
) -> HyperlitResult<()> {
    for summary_item in summary_items {
        match summary_item {
            SummaryItem::Link(link) => {
                let url = link
                    .location
                    .as_ref()
                    .map(|location| location.to_string_lossy())
                    .unwrap_or_else(|| Cow::Borrowed(""));
                writeln!(
                    file,
                    "{}[{}]({})",
                    prefix,
                    link.name,
                    url.replace(" ", "%20")
                )?;
                write_summary_items(file, &link.nested_items, &("    ".to_owned() + prefix))?
            }
            SummaryItem::Separator => {}
            SummaryItem::PartTitle(_) => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use hyperlit_backend::backend::Backend;
    use hyperlit_base::result::HyperlitResult;
    use hyperlit_base::shared_string::SharedString;
    use hyperlit_model::last_modification_info::DateTime;
    use hyperlit_model::location::Location;
    use hyperlit_model::segment::Segment;

    #[test]
    fn transform_segment() -> HyperlitResult<()> {
        let mut segment = Segment::new(
            42,
            "<title>",
            vec!["atag".to_string(), "btag".to_string()],
            "<text>",
            Location::new(SharedString::from("<filepath>"), 42),
        );
        segment.last_modification.author = Some("the author".to_string());
        segment.last_modification.date = DateTime::from_timestamp_millis(1234567890123);
        let backend = super::MdBookBackend::new();
        assert_eq!(
            backend.transform_segment(&segment)?,
            "## <title>\n\n<span class=\"tags\"> *#atag* *#btag*</span> the author 2009-02-13T23:31:30.123+00:00\n\n<text>\n\n`<filepath>:42`\n\n"
        );
        Ok(())
    }
}
