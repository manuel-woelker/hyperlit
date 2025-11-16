use crate::template_string::{TemplateString, TemplateStringPart, TemplateStringSubstitution};
use hyperlit_base::result::{Context, HyperlitResult};

pub struct TemplateExpander {
    template_string: TemplateString,
}

impl TemplateExpander {
    pub fn new(template: impl AsRef<str>) -> HyperlitResult<TemplateExpander> {
        let template_string = TemplateString::try_from(template.as_ref())?;
        Ok(TemplateExpander { template_string })
    }

    pub fn expand<T: AsRef<str>>(
        &self,
        substitution_fn: impl Fn(&TemplateStringSubstitution) -> HyperlitResult<T>,
    ) -> HyperlitResult<String> {
        let mut result = String::new();
        for part in &self.template_string.parts {
            match part {
                TemplateStringPart::PlainText(text) => {
                    result.push_str(text);
                }
                TemplateStringPart::Substitution(substitution) => {
                    let expanded = substitution_fn(substitution)
                        .with_context(|| format!("Error substituting '{:?}'", substitution))?;
                    result.push_str(expanded.as_ref());
                }
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyperlit_base::error::bail;

    fn test(input: &str, expected: &str) -> HyperlitResult<()> {
        let template_expander = TemplateExpander::new(input)?;
        assert_eq!(
            template_expander.expand(|s| Ok(format!(
                "<{}>({})",
                &s.directive,
                s.arguments.join(", ")
            )))?,
            expected
        );
        Ok(())
    }

    macro_rules! test {
        ($name: ident, $input: expr, $expected: expr) => {
            #[test]
            fn $name() -> HyperlitResult<()> {
                test($input, $expected)
            }
        };
    }

    test!(empty, "", "");
    test!(plain, "foo", "foo");
    test!(simple, "${name}", "<name>()");
    test!(args, "${name:foo,bar}", "<name>(foo, bar)");
    test!(mixed, "Hello ${name}!", "Hello <name>()!");

    #[test]
    fn test_error() -> HyperlitResult<()> {
        let template_expander = TemplateExpander::new("Hello, ${name}!")?;
        let error = template_expander
            .expand::<String>(|_s| bail!("some error"))
            .expect_err("expected error");
        assert_eq!(
            error.to_string(),
            "Error substituting 'TemplateStringSubstitution { directive: \"name\", arguments: [\"\"] }'"
        );
        Ok(())
    }
}
