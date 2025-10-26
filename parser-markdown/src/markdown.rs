use hyperlit_base::error::bail;
use hyperlit_base::result::HyperlitResult;
use hyperlit_model::element::Element;
use hyperlit_model::key::Key;
use hyperlit_model::value::Value;
use hyperlit_model::{attributes, tags};
use pulldown_cmark::{CodeBlockKind, Event, MetadataBlockKind, OffsetIter, Options, Tag};
use std::iter::Peekable;

pub fn parse_markdown(markdown: &str) -> HyperlitResult<Element> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    options.insert(Options::ENABLE_GFM);
    options.insert(Options::ENABLE_DEFINITION_LIST);
    options.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
    let parser = pulldown_cmark::Parser::new_ext(markdown, options);
    let mut iter = parser.into_offset_iter().peekable();
    let mut root = parse_markdown_stack(&mut iter)?;
    if iter.next().is_some() {
        bail!("Unexpected event after root element");
    }
    // convert metadata
    root.walk_mut(|element| {
        if element.tag() == &tags::METADATA {
            let mut metadata_string = String::new();
            let children = std::mem::take(element.children_mut());
            for child in children {
                if let Value::String(str) = child {
                    metadata_string.push_str(&str);
                } else {
                    bail!("Found non-string child in metadata block: {:?}", child)
                }
            }
            let mut key_maybe = None;
            let parser = saphyr_parser::Parser::new_from_str(&metadata_string);
            let mut mapping_depth = 0;
            for event in parser {
                let (event, _span) = event?;
                use saphyr_parser::Event;
                match event {
                    Event::Nothing => {}
                    Event::StreamStart => {}
                    Event::StreamEnd => {}
                    Event::DocumentStart(_) => {}
                    Event::DocumentEnd => {}
                    Event::Alias(_) => {}
                    Event::Scalar(value, _, _, _) => {
                        if mapping_depth != 1 {
                            bail!(
                                "Unexpected mapping depth in YAML frontmatter: {:?}",
                                metadata_string
                            );
                        }
                        if let Some(key) = key_maybe {
                            element.set_attribute(key, Value::String(value.to_string()));
                            key_maybe = None;
                        } else {
                            key_maybe = Some(Key::from(value.to_string()));
                        }
                    }
                    Event::SequenceStart(_, _) => {}
                    Event::SequenceEnd => {}
                    Event::MappingStart(_, _) => {
                        mapping_depth += 1;
                    }
                    Event::MappingEnd => {
                        mapping_depth -= 1;
                    }
                }
            }
        }
        Ok(())
    })?;
    Ok(root)
}

pub fn parse_markdown_stack(parser: &mut Peekable<OffsetIter>) -> HyperlitResult<Element> {
    let root = tags::DOCUMENT.new_element();
    let mut stack = vec![root];
    loop {
        if stack.len() == 1
            && let Some((Event::End(_), _)) = parser.peek()
        {
            // End found
            break;
        }
        let Some((event, range)) = parser.next() else {
            break;
        };
        match event {
            Event::Text(text) => {
                stack
                    .last_mut()
                    .expect("Top of stack is empty")
                    .children_mut()
                    .push(Value::String(text.into_string()));
            }
            Event::Start(tag) => {
                let mut element = Element::new();
                element.span_mut().start = range.start;
                element.span_mut().end = range.end;
                let tag = match tag {
                    Tag::Paragraph => tags::PARAGRAPH,
                    Tag::Heading { level, .. } => {
                        element
                            .set_attribute(attributes::HEADING_LEVEL, (level as usize).to_string());
                        tags::HEADING
                    }
                    Tag::Strong => tags::STRONG,
                    Tag::Emphasis => tags::EMPHASIS,
                    Tag::HtmlBlock => tags::HTML,
                    Tag::Link {
                        link_type: _,
                        dest_url,
                        title,
                        id,
                    } => {
                        element
                            .set_attribute(attributes::LINK_DESTINATION_URL, dest_url.to_string());
                        element.set_attribute(attributes::LINK_TILE, title.to_string());
                        element.set_attribute(attributes::ID, id.to_string());
                        tags::LINK
                    }
                    Tag::List(_firstitemnumber) => tags::LIST,
                    Tag::Item => tags::ITEM,
                    Tag::MetadataBlock(metadata) => {
                        match metadata {
                            MetadataBlockKind::YamlStyle => { /* supported */ }
                            _ => {
                                todo!("Unsupported metadata style: {:?}", metadata);
                            }
                        }
                        tags::METADATA
                    }
                    Tag::CodeBlock(CodeBlockKind::Indented) => tags::CODE_BLOCK,
                    Tag::CodeBlock(CodeBlockKind::Fenced(language)) => {
                        element
                            .set_attribute(attributes::CODEBLOCK_LANGUAGE, language.into_string());
                        tags::CODE_BLOCK
                    }
                    Tag::Table(_) | Tag::TableHead | Tag::TableRow | Tag::TableCell => {
                        // TODO: Implement table
                        tags::TABLE
                    }
                    Tag::Image { .. } => {
                        // TODO: Implement Image
                        tags::IMAGE
                    }
                    Tag::BlockQuote(_) => {
                        // TODO: Implement blockquote
                        tags::QUOTE
                    }
                    _ => todo!("Implement tag: {:?}", tag),
                };
                element.set_tag(tag);
                stack.push(element);
            }
            Event::End(_tag_end) => {
                let top = stack.pop().expect("Top of stack is empty");
                stack
                    .last_mut()
                    .expect("Top of stack is empty")
                    .children_mut()
                    .push(Value::Element(top));
            }
            Event::Rule => {
                dbg!(event);
            }
            Event::SoftBreak => {
                //dbg!(event);
            }
            Event::HardBreak => {
                //dbg!(event);
            }
            Event::Html(_html) => {
                //dbg!(html);
            }
            Event::InlineHtml(_html) => {
                //dbg!(html);
            }
            Event::Code(code) => {
                let mut element = tags::CODE.new_element();
                element.span_mut().start = range.start;
                element.span_mut().end = range.end;
                element
                    .children_mut()
                    .push(Value::String(code.into_string()));
                stack
                    .last_mut()
                    .expect("Top of stack is empty")
                    .children_mut()
                    .push(Value::Element(element));
            }
            other => todo!("Implement {:?}", other),
        }
    }
    let root = stack.pop().expect("stack empty");
    Ok(root)
}

#[cfg(test)]
mod tests {
    use super::parse_markdown;
    use expect_test::{Expect, expect};
    use hyperlit_base::result::HyperlitResult;

    fn test_parse(markdown: &str, expected: Expect) -> HyperlitResult<()> {
        let element = parse_markdown(markdown)?;
        expected.assert_eq(&format!("{:?}", element));
        Ok(())
    }

    #[test]
    fn test_parse_empty() -> HyperlitResult<()> {
        test_parse(
            "",
            expect!([r#"
                document (0+0)
            "#]),
        )
    }

    #[test]
    fn test_parse_plain() -> HyperlitResult<()> {
        test_parse(
            "foobar",
            expect!([r#"
                document (0+0)
                  paragraph (0+6)
                    "foobar"
            "#]),
        )
    }

    #[test]
    fn test_parse_headings() -> HyperlitResult<()> {
        test_parse(
            r#"# one

## two

"#,
            expect!([r#"
                document (0+0)
                  heading (0+6)
                    @level: "1"
                    "one"
                  heading (7+7)
                    @level: "2"
                    "two"
            "#]),
        )
    }

    #[test]
    fn test_parse_bold() -> HyperlitResult<()> {
        test_parse(
            "foo **bar** baz",
            expect!([r#"
                document (0+0)
                  paragraph (0+15)
                    "foo "
                    strong (4+7)
                      "bar"
                    " baz"
            "#]),
        )
    }

    #[test]
    fn test_parse_mixed() -> HyperlitResult<()> {
        test_parse(
            "**foo bar _fizz buzz_**",
            expect!([r#"
                document (0+0)
                  paragraph (0+23)
                    strong (0+23)
                      "foo bar "
                      emphasis (10+11)
                        "fizz buzz"
            "#]),
        )
    }

    #[test]
    fn test_parse_metadata() -> HyperlitResult<()> {
        test_parse(
            r#"
---
foo: bar
fizz: buzz
---
## two

"#,
            expect!([r#"
                document (0+0)
                  metadata (1+27)
                    @fizz: "buzz"
                    @foo: "bar"
                  heading (29+7)
                    @level: "2"
                    "two"
            "#]),
        )
    }
}
