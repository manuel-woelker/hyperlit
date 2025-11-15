use hyperlit_base::result::HyperlitResult;
use pulldown_cmark::{Event, MetadataBlockKind, Options, Tag, TagEnd};
use std::collections::HashMap;
use std::mem;

#[derive(Debug)]
pub struct MarkdownMetadata {
    pub heading: Option<String>,
    pub plain_heading: Option<String>,
    pub front_matter: HashMap<String, String>,
}

pub fn extract_markdown_metadata(markdown: &str) -> HyperlitResult<MarkdownMetadata> {
    let parser = pulldown_cmark::Parser::new_ext(
        markdown,
        Options::ENABLE_YAML_STYLE_METADATA_BLOCKS
            | Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS,
    );
    let mut metadata = MarkdownMetadata {
        heading: None,
        plain_heading: None,
        front_matter: HashMap::new(),
    };
    let mut plain_text = String::new();
    for (event, range) in parser.into_offset_iter() {
        match event {
            Event::Start(Tag::Heading { .. }) => {
                if metadata.heading.is_none() {
                    metadata.heading = Some(markdown[range].to_string());
                }
                plain_text.clear();
            }
            Event::Start(Tag::MetadataBlock(MetadataBlockKind::YamlStyle)) => {
                plain_text.clear();
            }
            Event::End(TagEnd::Heading { .. }) => {
                if metadata.plain_heading.is_none() && !plain_text.is_empty() {
                    metadata.plain_heading = Some(mem::take(&mut plain_text));
                    break;
                }
            }
            Event::End(TagEnd::MetadataBlock(MetadataBlockKind::YamlStyle)) => {
                metadata.front_matter = parse_yaml_frontmatter(&plain_text)?;
            }
            Event::Text(text) | Event::Code(text) => {
                plain_text.push_str(&text);
            }
            _ => {
                // Ignore everything else
            }
        }
    }
    Ok(metadata)
}

fn parse_yaml_frontmatter(yaml: &str) -> HyperlitResult<HashMap<String, String>> {
    Ok(serde_yaml::from_str(yaml)?)
}

// tests
#[cfg(test)]
mod tests {
    use crate::markdown_metadata::extract_markdown_metadata;
    use expect_test::{Expect, expect};

    fn test(markdown: &str, expected: Expect) {
        expected.assert_debug_eq(&extract_markdown_metadata(markdown));
    }

    macro_rules! test {
        ($name:ident, $input:expr, $expected:expr) => {
            #[test]
            fn $name() {
                test($input, $expected);
            }
        };
    }

    test!(
        heading,
        "# Hello world",
        expect![[r##"
            Ok(
                MarkdownMetadata {
                    heading: Some(
                        "# Hello world",
                    ),
                    plain_heading: Some(
                        "Hello world",
                    ),
                    front_matter: {},
                },
            )
        "##]]
    );
    test!(
        styled,
        "# Hello **world**",
        expect![[r##"
            Ok(
                MarkdownMetadata {
                    heading: Some(
                        "# Hello **world**",
                    ),
                    plain_heading: Some(
                        "Hello world",
                    ),
                    front_matter: {},
                },
            )
        "##]]
    );
    test!(
        styled2,
        "# Hello `world`",
        expect![[r##"
            Ok(
                MarkdownMetadata {
                    heading: Some(
                        "# Hello `world`",
                    ),
                    plain_heading: Some(
                        "Hello world",
                    ),
                    front_matter: {},
                },
            )
        "##]]
    );

    test!(
        h2,
        "## Hello world",
        expect![[r###"
            Ok(
                MarkdownMetadata {
                    heading: Some(
                        "## Hello world",
                    ),
                    plain_heading: Some(
                        "Hello world",
                    ),
                    front_matter: {},
                },
            )
        "###]]
    );

    test!(
        toml_simple,
        r#"---
title: 'Hello frontmatter'
---

        ## Hello world"#,
        expect![[r#"
            Ok(
                MarkdownMetadata {
                    heading: None,
                    plain_heading: None,
                    front_matter: {
                        "title": "Hello frontmatter",
                    },
                },
            )
        "#]]
    );
}
