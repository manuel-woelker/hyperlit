use crate::markdown_metadata::{MarkdownMetadata, extract_markdown_metadata};
use hyperlit_base::result::HyperlitResult;
use hyperlit_base::shared_string::SharedString;

#[derive(Debug)]
pub struct MarkdownInfo {
    pub title: SharedString,
    pub markdown_metadata: MarkdownMetadata,
}

pub fn extract_markdown_info(markdown: &str) -> HyperlitResult<MarkdownInfo> {
    let metadata = extract_markdown_metadata(markdown)?;
    let title = metadata
        .front_matter
        .get("title")
        .or(metadata.plain_title.as_ref())
        .map(|x| x.as_str())
        .unwrap_or("-Untitled-")
        .into();
    Ok(MarkdownInfo {
        title,
        markdown_metadata: metadata,
    })
}

#[cfg(test)]
mod tests {
    use super::extract_markdown_info;
    use expect_test::{Expect, expect};
    use hyperlit_base::result::HyperlitResult;

    fn test_parse(markdown: &str, expected: Expect) -> HyperlitResult<()> {
        let element = extract_markdown_info(markdown)?;
        expected.assert_eq(&format!("{:?}", element));
        Ok(())
    }

    #[test]
    fn test_parse_empty() -> HyperlitResult<()> {
        test_parse(
            "",
            expect!([
                r#"MarkdownInfo { title: "-Untitled-", markdown_metadata: MarkdownMetadata { title: None, plain_title: None, front_matter: {} } }"#
            ]),
        )
    }

    #[test]
    fn test_parse_plain() -> HyperlitResult<()> {
        test_parse(
            "foobar",
            expect!([
                r#"MarkdownInfo { title: "-Untitled-", markdown_metadata: MarkdownMetadata { title: None, plain_title: None, front_matter: {} } }"#
            ]),
        )
    }

    #[test]
    fn test_parse_headings() -> HyperlitResult<()> {
        test_parse(
            r#"# one

## two

"#,
            expect!([
                r##"MarkdownInfo { title: "one", markdown_metadata: MarkdownMetadata { title: Some("# one\n"), plain_title: Some("one"), front_matter: {} } }"##
            ]),
        )
    }

    #[test]
    fn test_parse_bold() -> HyperlitResult<()> {
        test_parse(
            "# foo **bar** baz",
            expect!([
                r##"MarkdownInfo { title: "foo bar baz", markdown_metadata: MarkdownMetadata { title: Some("# foo **bar** baz"), plain_title: Some("foo bar baz"), front_matter: {} } }"##
            ]),
        )
    }

    #[test]
    fn test_parse_mixed() -> HyperlitResult<()> {
        test_parse(
            "# **foo bar _fizz buzz_**",
            expect!([
                r##"MarkdownInfo { title: "foo bar fizz buzz", markdown_metadata: MarkdownMetadata { title: Some("# **foo bar _fizz buzz_**"), plain_title: Some("foo bar fizz buzz"), front_matter: {} } }"##
            ]),
        )
    }

    #[test]
    fn test_parse_metadata() -> HyperlitResult<()> {
        test_parse(
            r#"
---
title: fizz
---
## two

"#,
            expect!([
                r###"MarkdownInfo { title: "fizz", markdown_metadata: MarkdownMetadata { title: Some("## two\n"), plain_title: Some("two"), front_matter: {"title": "fizz"} } }"###
            ]),
        )
    }
}
