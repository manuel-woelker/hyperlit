use crate::markdown::parse_markdown_stack;
use hyperlit_base::error::err;
use hyperlit_base::result::HyperlitResult;
use hyperlit_model::value::Value;
use indenter::indented;
use pulldown_cmark::{Event, Options, Tag, TagEnd};
use std::fmt::Write;
use std::fmt::{Debug, Formatter};

#[derive(Default)]
pub struct Summary {
    entries: Vec<SummaryEntry>,
}

impl Summary {
    pub fn new() -> Summary {
        Summary {
            entries: Vec::new(),
        }
    }

    pub fn entries(&self) -> &[SummaryEntry] {
        &self.entries
    }
}

impl Debug for Summary {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Summary:")?;
        let mut indenter = indented(f);
        for entry in &self.entries {
            write!(indenter, "{:?}", entry)?;
        }
        Ok(())
    }
}

pub struct SummaryEntry {
    label: Value,
    path: String,
    children: Vec<SummaryEntry>,
}

impl SummaryEntry {
    pub fn new(label: Value, path: String) -> SummaryEntry {
        SummaryEntry {
            label,
            path,
            children: Vec::new(),
        }
    }

    pub fn label(&self) -> &Value {
        &self.label
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn children(&self) -> &[SummaryEntry] {
        &self.children
    }
}

impl Debug for SummaryEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}: {}", self.label, self.path)?;
        let mut indenter = indented(f);
        for child in &self.children {
            write!(indenter, "{:?}", child)?;
        }
        Ok(())
    }
}

pub fn parse_summary(input: &str) -> HyperlitResult<Summary> {
    let mut summary = Summary::new();
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    options.insert(Options::ENABLE_GFM);
    options.insert(Options::ENABLE_DEFINITION_LIST);
    options.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
    let parser = pulldown_cmark::Parser::new_ext(input, options);
    let mut iter = parser.into_offset_iter().peekable();
    let mut stack: Vec<SummaryEntry> = vec![SummaryEntry::new(
        Value::new_string("document"),
        "".to_string(),
    )];
    loop {
        let Some((event, ..)) = iter.next() else {
            break;
        };
        match event {
            Event::Start(Tag::Item) => {
                stack.push(SummaryEntry::new(Value::new_empty(), "".to_string()));
            }
            Event::Start(Tag::Link { dest_url, .. }) => {
                let top = stack.last_mut().ok_or_else(|| err!("empty stack"))?;
                top.path = dest_url.to_string();
                let element = parse_markdown_stack(&mut iter)?;
                top.label = Value::Element(element);
            }
            Event::End(TagEnd::Item) => {
                let entry = stack.pop().ok_or_else(|| err!("empty stack"))?;
                stack
                    .last_mut()
                    .ok_or_else(|| err!("empty stack"))?
                    .children
                    .push(entry);
            }
            _ => {}
        }
    }
    summary.entries = stack.pop().ok_or_else(|| err!("empty stack"))?.children;
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use crate::summary::parse_summary;
    use expect_test::{Expect, expect};

    #[test]
    fn empty() {
        test_summary(
            "",
            expect![[r#"
            Summary:

        "#]],
        );
    }

    #[test]
    fn simple() {
        test_summary(
            r#"
- [Introduction](Introduction.md)
- [Table of Contents](toc.md)
        "#,
            expect![[r#"
            Summary:
                Introduction: Introduction.md
                Table of Contents: toc.md

        "#]],
        );
    }

    #[test]
    fn nested() {
        test_summary(
            r#"
 - [foo](foo.md)
   - [bar](bar.md)
   - [baz](baz.md)
 - [fizz](fizz.md)
   - [buzz](buzz.md)
     - [bizz](bizz.md)
     - [`code`](code.md)
     - [*emph*](code.md)
     - [**strong**](code.md)
        "#,
            expect![[r#"
                Summary:
                    foo: foo.md
                        bar: bar.md
                        baz: baz.md
                    fizz: fizz.md
                        buzz: buzz.md
                            bizz: bizz.md
                            code: code.md
                            emph: code.md
                            strong: code.md

            "#]],
        );
    }

    fn test_summary(input: &str, expect: Expect) {
        let summary = parse_summary(input).unwrap();
        expect.assert_debug_eq(&summary);
    }
}
