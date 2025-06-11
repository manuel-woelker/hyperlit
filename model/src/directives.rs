use hyperlit_base::bail;
use hyperlit_base::result::HyperlitResult;

#[derive(Debug, PartialEq)]
pub enum Directive {
    IncludeByTag { tag: String },
    Include,
}

pub fn parse_directive(directive_string: &str) -> HyperlitResult<Directive> {
    let directive_string = directive_string.trim();
    Ok(
        if let Some(tag) = directive_string.strip_prefix("@include_by_tag:") {
            let tag = tag.trim();
            let tag = tag.strip_prefix("#").unwrap_or(tag);
            Directive::IncludeByTag {
                tag: tag.to_string(),
            }
        } else if directive_string == "@include" {
            Directive::Include
        } else {
            bail!("Unknown directive: '{}'", directive_string);
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_include_by_tag() {
        assert_eq!(
            parse_directive("@include_by_tag: #the_tag").unwrap(),
            Directive::IncludeByTag {
                tag: "the_tag".to_string()
            }
        );
    }

    #[test]
    fn test_parse_include() {
        assert_eq!(parse_directive("@include").unwrap(), Directive::Include,);
    }

    #[test]
    fn test_parse_other() {
        assert_eq!(
            parse_directive("foobar").unwrap_err().to_string(),
            "Unknown directive: 'foobar'"
        );
    }
}
