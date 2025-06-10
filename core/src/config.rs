use hyperlit_base::result::HyperlitResult;
use hyperlit_base::{bail, context, err};
use path_absolutize::Absolutize;
use std::path::Path;
use toml_span::parse;

/// Hyperlit configuration used to configure the document generation process
#[derive(Debug)]
pub struct HyperlitConfig {
    /// Root path to source code. This may be the repository root to collect all files
    pub src_directory: String,
    /// Globs to use when searching for source files, these may be prefixed with "!" to exclude files or directories
    pub src_globs: Vec<String>,
    /// Path to the docs directory
    pub docs_directory: String,
    /// Globs to use when searching for documentation files, may be "*" to include all files
    pub doc_globs: Vec<String>,
    /// Path to a build directory used for temporary files
    pub build_directory: String,
    /// Directory to write the complete documentation output to
    pub output_directory: String,
    /// List of marker strings used to identify documentation segments to extract from the source code
    pub doc_markers: Vec<String>,
}

impl HyperlitConfig {
    pub fn from_path(path: impl AsRef<Path>) -> HyperlitResult<Self> {
        let absolute_path = path.as_ref().absolutize()?.to_path_buf();
        context!("read config from file: {:?}", absolute_path => {
            let string = std::fs::read_to_string(path.as_ref())?;
            Self::from_string(&string)
        })
    }

    pub fn from_string(string: &str) -> HyperlitResult<Self> {
        let toml = parse(string)?;
        let table = toml.as_table().ok_or_else(|| err!("not a table"))?;

        Ok(Self {
            src_directory: get_string(table, "src_directory")?,
            docs_directory: get_string(table, "docs_directory")?,
            build_directory: get_string_or(table, "build_directory", "build")?,
            output_directory: get_string_or(table, "output_directory", "output")?,
            doc_globs: get_string_array(table, "doc_globs")?,
            src_globs: get_string_array(table, "src_globs")?,
            doc_markers: get_string_array_or(table, "doc_markers", &["📖", "DOC"])?,
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

/// Helper method to get a string array or default
fn get_string_array_or(
    table: &toml_span::value::Table,
    key: &str,
    default: &[&str],
) -> HyperlitResult<Vec<String>> {
    match table.get(key) {
        None => Ok(Vec::from_iter(default.iter().map(|s| s.to_string()))),
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
        let config = HyperlitConfig::from_string(
            r#"

            src_directory = "the_source"
            src_globs = ["*.rs"]
            docs_directory = "the_docs"
            doc_globs = ["*.md", "*.mdx"]
        "#,
        )?;

        expect![[r#"
            HyperlitConfig {
                src_directory: "the_source",
                src_globs: [
                    "*.rs",
                ],
                docs_directory: "the_docs",
                doc_globs: [
                    "*.md",
                    "*.mdx",
                ],
                build_directory: "build",
                output_directory: "output",
                doc_markers: [
                    "📖",
                    "DOC",
                ],
            }
        "#]]
        .assert_debug_eq(&config);
        Ok(())
    }

    #[test]
    fn read_config_with_no_defaults() -> HyperlitResult<()> {
        let config = HyperlitConfig::from_string(
            r#"
            src_directory = "the_source"
            src_globs = ["*.rs"]
            docs_directory = "the_docs"
            doc_globs = ["*.md", "*.mdx"]
            build_directory = "the_build"
            output_directory = "the_output"
            doc_markers = ["foo", "bar"]
        "#,
        )?;

        expect![[r#"
            HyperlitConfig {
                src_directory: "the_source",
                src_globs: [
                    "*.rs",
                ],
                docs_directory: "the_docs",
                doc_globs: [
                    "*.md",
                    "*.mdx",
                ],
                build_directory: "the_build",
                output_directory: "the_output",
                doc_markers: [
                    "foo",
                    "bar",
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
                src_globs: [
                    "*.rs",
                ],
                docs_directory: "the_docs",
                doc_globs: [
                    "*.md",
                    "*.mdx",
                ],
                build_directory: "the_build",
                output_directory: "the_output",
                doc_markers: [
                    "📖",
                    "DOC",
                ],
            }
        "#]]
        .assert_debug_eq(&config);
        Ok(())
    }
}
