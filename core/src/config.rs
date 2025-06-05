use hyperlit_base::result::HyperlitResult;
use hyperlit_base::{bail, context, err};
use path_absolutize::Absolutize;
use std::path::Path;
use toml_span::parse;

#[derive(Debug)]
pub struct HyperlitConfig {
    pub src_directory: String,
    pub src_extensions: Vec<String>,
    pub docs_directory: String,
    pub doc_extensions: Vec<String>,
    pub build_directory: String,
    pub output_directory: String,
}

impl HyperlitConfig {
    pub fn from_path(path: impl AsRef<Path>) -> HyperlitResult<Self> {
        let absolute_path = path.as_ref().absolutize()?.to_path_buf();
        context!("read config from file: {:?}", absolute_path => {
            let string = std::fs::read_to_string(path.as_ref())?;
            Self::from_str(&string)
        })
    }

    pub fn from_str(string: &str) -> HyperlitResult<Self> {
        let toml = parse(string)?;
        let table = toml.as_table().ok_or_else(|| err!("not a table"))?;

        Ok(Self {
            src_directory: get_string(table, "src_directory")?,
            docs_directory: get_string(table, "docs_directory")?,
            build_directory: get_string_or(table, "build_directory", "build")?,
            output_directory: get_string_or(table, "output_directory", "output")?,
            doc_extensions: get_string_array(table, "doc_extensions")?,
            src_extensions: get_string_array(table, "src_extensions")?,
        })
    }
}

/// Helper method to get a string value from a TOML table
fn get_string(table: &toml_span::value::Table, key: &str) -> HyperlitResult<String> {
    table
        .get(key)
        .ok_or_else(|| err!("{} not found", key))?
        .as_str()
        .ok_or_else(|| err!("{} is not a string", key))
        .map(|s| s.to_string())
}

/// Helper method to get a string value with a default
fn get_string_or(
    table: &toml_span::value::Table,
    key: &str,
    default: &str,
) -> HyperlitResult<String> {
    match table.get(key) {
        None => Ok(default.to_string()),
        Some(value) => value
            .as_str()
            .ok_or_else(|| err!("{} is not a string", key))
            .map(|s| s.to_string()),
    }
}

/// Helper method to get a string array
fn get_string_array(table: &toml_span::value::Table, key: &str) -> HyperlitResult<Vec<String>> {
    match table.get(key) {
        None => {
            bail!("{} not found", key);
        }
        Some(value) => value
            .as_array()
            .ok_or_else(|| err!("{} is not an array", key))?
            .iter()
            .map(|v| {
                v.as_str()
                    .ok_or_else(|| err!("{} is not a string", key))
                    .map(|s| s.to_string())
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use crate::config::HyperlitConfig;
    use expect_test::expect;
    use hyperlit_base::result::HyperlitResult;

    #[test]
    fn read_config_with_defaults() -> HyperlitResult<()> {
        let config = HyperlitConfig::from_str(
            r#"

            src_directory = "the_source"
            docs_directory = "the_docs"
            doc_extensions = ["md", "mdx"]
        "#,
        )?;

        expect![[r#"
            HyperlitConfig {
                src_directory: "the_source",
                docs_directory: "the_docs",
                build_directory: "build",
                output_directory: "output",
                doc_extensions: [
                    "md",
                    "mdx",
                ],
            }
        "#]]
        .assert_debug_eq(&config);
        Ok(())
    }

    #[test]
    fn read_config_with_no_defaults() -> HyperlitResult<()> {
        let config = HyperlitConfig::from_str(
            r#"
            src_directory = "the_source"
            docs_directory = "the_docs"
            doc_extensions = ["md", "mdx"]
            build_directory = "the_build"
            output_directory = "the_output"
        "#,
        )?;

        expect![[r#"
            HyperlitConfig {
                src_directory: "the_source",
                docs_directory: "the_docs",
                build_directory: "the_build",
                output_directory: "the_output",
                doc_extensions: [
                    "md",
                    "mdx",
                ],
            }
        "#]]
        .assert_debug_eq(&config);
        Ok(())
    }

    #[test]
    fn read_config_with_no_defaults_from_file() -> HyperlitResult<()> {
        let config = HyperlitConfig::from_path("test/hyperlit-test.toml")?;

        expect![[r#"
            HyperlitConfig {
                src_directory: "the_source",
                docs_directory: "the_docs",
                build_directory: "the_build",
                output_directory: "the_output",
                doc_extensions: [
                    "md",
                    "mdx",
                ],
            }
        "#]]
        .assert_debug_eq(&config);
        Ok(())
    }
}
