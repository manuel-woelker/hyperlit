use hyperlit_base::result::HyperlitResult;
use hyperlit_base::shared_string::SharedString;
use pulldown_cmark::{Event, MetadataBlockKind, Options, Tag, TagEnd};

pub struct MarkdownInfo {
    pub title: SharedString,
}

pub fn extract_markdown_info(markdown: &str) -> HyperlitResult<MarkdownInfo> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    options.insert(Options::ENABLE_GFM);
    options.insert(Options::ENABLE_DEFINITION_LIST);
    options.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
    let parser = pulldown_cmark::Parser::new_ext(markdown, options);
    let iter = parser.into_offset_iter().peekable();
    let mut title = String::new();
    let mut metadata = String::new();
    let mut in_metadata = false;
    let mut in_title = false;
    for (event, _range) in iter {
        match event {
            Event::Start(Tag::MetadataBlock(MetadataBlockKind::YamlStyle)) => {
                in_metadata = true;
            }
            Event::End(TagEnd::MetadataBlock(MetadataBlockKind::YamlStyle)) => {
                in_metadata = false;
            }
            Event::Start(Tag::Heading { .. }) => {
                in_title = true;
            }
            Event::End(TagEnd::Heading { .. }) => {
                break;
            }
            Event::Text(text) => {
                if in_metadata {
                    metadata.push_str(&text);
                } else if in_title {
                    title.push_str(&text);
                }
            }
            _ => {}
        }
    }
    // TODO: convert metadata
    /*    root.walk_mut(|element| {
        if element.tag() == &tags::METADATA {
            let mut metadata_string = String::new();
            let children = std::mem::take(element.children_mut());
            for child in children {
                if let Value::Text(text) = child {
                    metadata_string.push_str(text.content());
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
                            element
                                .set_attribute(key, Value::new_text_unspanned(value.to_string()));
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
    })?;*/
    Ok(MarkdownInfo {
        title: title.into(),
    })
}

#[cfg(test)]
mod tests {
    // TODO: implement tests
    /*
    use super::extract_markdown_info;
    use expect_test::{Expect, expect};
    use hyperlit_base::result::HyperlitResult;

    fn test_parse(markdown: &str, expected: Expect) -> HyperlitResult<()> {
        let element = extract_markdown_info(markdown, 99)?;
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
    */
}
