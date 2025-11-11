use hyperlit_base::error::{bail, err};
use hyperlit_base::result::HyperlitResult;
use hyperlit_model::book_definition::{BookDefinition, ChapterDefinition};
use toml_span::parse;
/* 📖 hyperlit.toml configuration file #config #howto

The hyperlit.toml configuration file is used to configure the document generation process.
It contains the following fields:

 */

/// Hyperlit configuration used to configure the document generation process
#[derive(Debug, Clone)]
pub struct HyperlitConfig {
    /// Title of the resulting document
    pub title: String,
    /// path to the hyperlit.toml file
    pub config_path: String,
    // 📖 ... - `src_directory`: root path to source code. This may be the repository root (i.e., ".") to scan all directories in the repository
    /// Root path to source code. This may be the repository root to collect all files
    pub src_directory: String,
    // 📖 ... - `src_globs`: globs to use when searching for source files, these may be prefixed with "!" to exclude files or directories
    /// Globs to use when searching for source files, these may be prefixed with "!" to exclude files or directories
    pub src_globs: Vec<String>,
    // 📖 ... - `docs_directory`: path to the docs directory, this should contain the documentation files
    /// Path to the docs directory
    pub docs_directory: String,
    // 📖 ... - `doc_globs`: globs to use when searching for documentation files, may be "*" to include all files
    /// Globs to use when searching for documentation files, may be "*" to include all files
    pub doc_globs: Vec<String>,
    // 📖 ... - `build_directory`: path to a build directory used for temporary files
    /// Path to a build directory used for temporary files
    pub build_directory: String,
    // 📖 ... - `output_directory`: directory to write the complete documentation output to
    /// Directory to write the complete documentation output to
    pub output_directory: String,
    // 📖 ... - `doc_markers`: list of marker strings used to identify documentation segments to extract from the source code, defaults to `["📖", "DOC"]`
    /// List of marker strings used to identify documentation segments to extract from the source code
    pub doc_markers: Vec<String>,
    // 📖 ... - `source_link_template`: Template used to generate links to source code (e.g. on github, etc.), placeholders `${path}` and `${line}` will be replaced
    /// Template used to generate links to source code (e.g. on github, etc.)
    pub source_link_template: Option<String>,
    // 📖 ... Book structure
    /// Structure of the book
    pub structure: BookDefinition,
}

impl HyperlitConfig {
    pub fn from_string(string: &str) -> HyperlitResult<Self> {
        let toml = parse(string)?;
        let table = toml.as_table().ok_or_else(|| err!("not a table"))?;

        Ok(Self {
            title: get_string(table, "title")?,
            config_path: table
                .get("config_path")
                .and_then(|x| x.as_str())
                .unwrap_or("hyperlit.toml")
                .to_string(),
            src_directory: get_string(table, "src_directory")?,
            docs_directory: get_string(table, "docs_directory")?,
            build_directory: get_string_or(table, "build_directory", "build")?,
            output_directory: get_string_or(table, "output_directory", "output")?,
            doc_globs: get_string_array(table, "doc_globs")?,
            src_globs: get_string_array(table, "src_globs")?,
            doc_markers: get_string_array_or(table, "doc_markers", &["📖", "DOC"])?,
            source_link_template: get_string(table, "source_link_template").ok(),
            structure: parse_structure(
                toml.pointer("/structure/chapter")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| err!("structure not found"))?,
            )?,
        })
    }
}

fn parse_structure(chapters_array: &toml_span::value::Array) -> HyperlitResult<BookDefinition> {
    let mut chapters = vec![];
    for chapter in chapters_array {
        let table = chapter
            .as_table()
            .ok_or_else(|| err!("chapter is not a table"))?;

        let label = get_string(table, "label")?;
        let tags = get_string_array(table, "tags")?;
        let directories = get_string_array_or(table, "directories", &[])?;
        chapters.push(ChapterDefinition {
            label,
            tags,
            directories: if directories.is_empty() {
                None
            } else {
                Some(directories)
            },
            chapters: vec![],
        })
    }
    Ok(BookDefinition {
        title: "<untitled>".into(),
        chapters,
    })
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
            title = "foo"
            src_directory = "the_source"
            src_globs = ["*.rs"]
            docs_directory = "the_docs"
            doc_globs = ["*.md", "*.mdx"]

            [[structure.chapter]]
            label = "foo"
            tags = ["bar"]
        "#,
        )?;

        expect![[r#"
            HyperlitConfig {
                title: "foo",
                config_path: "hyperlit.toml",
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
                source_link_template: None,
                structure: BookDefinition {
                    title: "<untitled>",
                    chapters: [
                        ChapterDefinition {
                            label: "foo",
                            tags: [
                                "bar",
                            ],
                            directories: None,
                            chapters: [],
                        },
                    ],
                },
            }
        "#]]
        .assert_debug_eq(&config);
        Ok(())
    }

    #[test]
    fn read_config_with_no_defaults() -> HyperlitResult<()> {
        let config = HyperlitConfig::from_string(
            r#"
            title = "foo"
            src_directory = "the_source"
            src_globs = ["*.rs"]
            docs_directory = "the_docs"
            doc_globs = ["*.md", "*.mdx"]
            build_directory = "the_build"
            output_directory = "the_output"
            doc_markers = ["foo", "bar"]

            [[structure.chapter]]
            label = "foo"
            directories = ["foo"]
            tags = ["bar"]
        "#,
        )?;

        expect![[r#"
            HyperlitConfig {
                title: "foo",
                config_path: "hyperlit.toml",
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
                source_link_template: None,
                structure: BookDefinition {
                    title: "<untitled>",
                    chapters: [
                        ChapterDefinition {
                            label: "foo",
                            tags: [
                                "bar",
                            ],
                            directories: Some(
                                [
                                    "foo",
                                ],
                            ),
                            chapters: [],
                        },
                    ],
                },
            }
        "#]]
        .assert_debug_eq(&config);
        Ok(())
    }
}
