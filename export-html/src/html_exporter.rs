use crate::convert_code_block::ConvertCodeBlock;
use crate::convert_document::ConvertDocument;
use crate::convert_heading::ConvertHeading;
use crate::convert_link::ConvertLink;
use crate::convert_passthru::ConvertPassthru;
use crate::convert_simple::ConvertSimple;
use crate::convert_tag::{ConversionContext, ConvertTag};
use hyperlit_base::result::HyperlitResult;
use hyperlit_model::book::Book;
use hyperlit_model::chapter::Chapter;
use hyperlit_model::element::Element;
use hyperlit_model::key::Key;
use hyperlit_model::tags;
use hyperlit_model::value::Value;
use pulldown_cmark_escape::{IoWriter, escape_html_body_text};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::io::{Cursor, Write};

pub struct HtmlExporter {
    converter_map: HashMap<Key, Box<dyn ConvertTag>>,
    unhandled_tags: RefCell<HashSet<Key>>,
}

impl HtmlExporter {
    pub fn new() -> Self {
        let mut exporter = Self {
            converter_map: HashMap::new(),
            unhandled_tags: RefCell::new(HashSet::new()),
        };
        exporter.register_converter(tags::DOCUMENT, ConvertPassthru::new());
        exporter.register_converter(tags::ARTICLE, ConvertDocument::new());
        exporter.register_converter(tags::HEADING, ConvertHeading::new());
        exporter.register_converter(tags::LINK, ConvertLink::new());
        exporter.register_converter(tags::CODE_BLOCK, ConvertCodeBlock::new());
        let mut register_tag = |hyperlit_tag: Key, html_tag: &'static str| {
            exporter.register_converter(hyperlit_tag, ConvertSimple::new(html_tag));
        };

        register_tag(tags::PARAGRAPH, "p");
        register_tag(tags::STRONG, "strong");
        register_tag(tags::CODE, "code");
        register_tag(tags::LIST, "ul");
        register_tag(tags::ITEM, "li");
        register_tag(tags::TITLE, "<title>");
        exporter
    }

    pub fn register_converter(
        &mut self,
        key: impl Into<Key>,
        converter: impl ConvertTag + 'static,
    ) {
        self.converter_map.insert(key.into(), Box::new(converter));
    }

    pub fn unhandled_tags(&self) -> HashSet<Key> {
        self.unhandled_tags.borrow().clone()
    }

    pub fn export_book_to_html(&self, book: &Book) -> HyperlitResult<String> {
        let mut cursor = Cursor::new(Vec::new());
        self.write_front_matter(&mut cursor, book)?;
        self.write_toc(&mut cursor, book)?;
        self.write_content(&mut cursor, book)?;
        Ok(String::from_utf8(cursor.into_inner())?)
    }

    fn write_front_matter(&self, output_file: &mut dyn Write, book: &Book) -> HyperlitResult<()> {
        let title = &book.title;
        writeln!(output_file, "<header><h1>",)?;
        self.export_value_to_html(output_file, title)?;
        writeln!(output_file, "</h1>")?;
        for author in &book.authors {
            write!(output_file, "<em>")?;
            self.export_value_to_html(output_file, author)?;
            writeln!(output_file, "</em>")?;
        }
        writeln!(output_file, "</header>")?;
        Ok(())
    }

    fn write_toc(&self, output_file: &mut dyn Write, book: &Book) -> HyperlitResult<()> {
        writeln!(output_file, "<div class=\"toc\"><ul>")?;
        for chapter in &book.chapters {
            self.write_toc_entry(output_file, chapter)?;
        }
        writeln!(output_file, "</ul></div>")?;
        Ok(())
    }

    fn write_toc_entry(
        &self,
        output_file: &mut dyn Write,
        chapter: &Chapter,
    ) -> HyperlitResult<()> {
        writeln!(output_file, "<li>")?;
        writeln!(output_file, "<a href=\"#{}\">", chapter.id,)?;
        self.export_value_to_html(output_file, &chapter.label)?;
        writeln!(output_file, "</a>",)?;
        if !chapter.sub_chapters.is_empty() {
            writeln!(output_file, "<ul>")?;
            for sub_chapter in &chapter.sub_chapters {
                self.write_toc_entry(output_file, sub_chapter)?;
            }
            writeln!(output_file, "</ul>")?;
        }
        writeln!(output_file, "</li>")?;
        Ok(())
    }

    fn write_content(&self, output_file: &mut dyn Write, book: &Book) -> HyperlitResult<()> {
        writeln!(output_file, "<main>")?;
        for chapter in &book.chapters {
            self.write_chapter(output_file, chapter, 1)?;
        }
        writeln!(output_file, "</main>")?;
        Ok(())
    }

    fn write_chapter(
        &self,
        output_file: &mut dyn Write,
        chapter: &Chapter,
        level: usize,
    ) -> HyperlitResult<()> {
        writeln!(output_file, "<section>")?;
        writeln!(output_file, "<a id=\"{}\"><h{level}>", chapter.id)?;
        self.export_value_to_html(output_file, &chapter.label)?;
        writeln!(output_file, "</h{level}></a>",)?;
        self.export_value_to_html(output_file, &chapter.body)?;
        if !chapter.sub_chapters.is_empty() {
            for sub_chapter in &chapter.sub_chapters {
                self.write_chapter(output_file, sub_chapter, level + 1)?;
            }
        }
        writeln!(output_file, "</section>")?;
        Ok(())
    }
}

impl Default for HtmlExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl HtmlExporter {
    pub fn export_to_html(&self, write: &mut dyn Write, document: &Element) -> HyperlitResult<()> {
        self.export_element(write, document)
    }

    pub fn export_value_to_html(
        &self,
        mut write: &mut dyn Write,
        value: &Value,
    ) -> HyperlitResult<()> {
        match value {
            Value::Element(element) => {
                self.export_element(write, element)?;
            }
            Value::String(text) => {
                let mut writer = IoWriter(&mut write);
                escape_html_body_text(&mut writer, text)?;
            }
        }
        Ok(())
    }

    fn export_element(&self, write: &mut dyn Write, element: &Element) -> HyperlitResult<()> {
        let converter = self.converter_map.get(element.tag());
        let conversion_context = ConversionContext::new(element);
        if let Some(converter) = converter {
            converter.emit_before(write, &conversion_context)?;
        } else {
            self.unhandled_tags
                .borrow_mut()
                .insert(element.tag().clone());
        }
        for child in element.children() {
            self.export_value_to_html(write, child)?;
        }
        if let Some(converter) = converter {
            converter.emit_after(write, &conversion_context)?;
        }
        Ok(())
    }
}

pub fn export_to_html(write: &mut dyn Write, document: &Element) -> HyperlitResult<()> {
    let exporter = HtmlExporter::new();
    exporter.export_to_html(write, document)
}

pub fn export_book_to_html(book: &Book) -> HyperlitResult<String> {
    let exporter = HtmlExporter::new();
    exporter.export_book_to_html(book)
}

#[cfg(test)]
mod tests {
    use crate::convert_document::ConvertDocument;
    use crate::html_exporter::HtmlExporter;
    use expect_test::{Expect, expect};
    use hyperlit_base::result::HyperlitResult;
    use hyperlit_model::element::Element;
    use hyperlit_model::value::Value;
    use hyperlit_parser_markdown::markdown::parse_markdown;
    use std::collections::HashSet;

    #[test]
    fn test_export_empty_to_html() {
        let document = Element::new_tag("document");
        let mut buffer = Vec::new();
        let mut exporter = HtmlExporter::new();
        exporter.register_converter("document", ConvertDocument::new_inline_css("body {}"));
        exporter.export_to_html(&mut buffer, &document).unwrap();
        expect![[r#"
            <!DOCTYPE html>
            <html>
            <head>
            <style>
            body {}
            </style>
            </head>
            <body>
            <main>
            </main>
            </body>
            </html>
        "#]]
        .assert_eq(str::from_utf8(&buffer).unwrap());
        assert_eq!(exporter.unhandled_tags(), HashSet::new());
    }

    #[test]
    fn test_export_simple_to_html() {
        let mut document = Element::new_tag("document");
        document
            .children_mut()
            .push(Value::String("Hello, World!".to_string()));
        let mut buffer = Vec::new();
        let mut exporter = HtmlExporter::new();
        exporter.register_converter("document", ConvertDocument::new_inline_css("body {}"));
        exporter.export_to_html(&mut buffer, &document).unwrap();
        expect![[r#"
            <!DOCTYPE html>
            <html>
            <head>
            <style>
            body {}
            </style>
            </head>
            <body>
            <main>
            Hello, World!</main>
            </body>
            </html>
        "#]]
        .assert_eq(str::from_utf8(&buffer).unwrap());
        assert_eq!(exporter.unhandled_tags(), HashSet::new());
    }

    fn test_export(input: &str, expected: Expect) -> HyperlitResult<()> {
        let document = parse_markdown(input)?;
        let mut buffer = Vec::new();
        let exporter = HtmlExporter::new();
        exporter.export_to_html(&mut buffer, &document).unwrap();
        expected.assert_eq(str::from_utf8(&buffer).unwrap());
        assert_eq!(exporter.unhandled_tags(), HashSet::new());
        Ok(())
    }

    macro_rules! test_export {
        ($name:ident, $input:expr, $expected:expr) => {
            #[test]
            fn $name() -> HyperlitResult<()> {
                test_export($input, $expected)
            }
        };
    }

    test_export!(empty, "", expect![""]);
    test_export!(simple, "foobar", expect!["<p>foobar</p>"]);
    test_export!(
        bold,
        "**Bolded**",
        expect!["<p><strong>Bolded</strong></p>"]
    );
    test_export!(
        escapes,
        "LT: < GT: > AMP: & QUOTE: \" SINGLE QUOTE: '",
        expect![[r#"<p>LT: &lt; GT: &gt; AMP: &amp; QUOTE: " SINGLE QUOTE: '</p>"#]]
    );
}
