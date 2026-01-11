use hyperlit_base::error::{bail, err};
use hyperlit_base::result::HyperlitResult;
use hyperlit_base::shared_string::SharedString;
use toml_span::parse;
/* 📖 hyperlit.toml configuration file #config #howto

The hyperlit.toml configuration file is used to configure the document generation process.
It contains the following fields:

 */

/// Hyperlit configuration used to configure the document generation process
#[derive(Debug, Clone)]
pub struct HyperlitConfig {
    /// Title of the resulting document
    pub title: SharedString,
    /// path to the hyperlit.toml file
    pub config_path: String,
    // 📖 ... - `src_directory`: root path to source code. This may be the repository root (i.e., ".") to scan all directories in the repository
    /// Path to a build directory used for temporary files
    pub build_directory: String,
    // 📖 ... - `output_directory`: directory to write the complete documentation output to
    /// Directory to write the complete documentation output to
    pub output_directory: String,

    /// Template used to generate links to source code (e.g. on github, etc.)
    pub source_link_template: Option<String>,

    /// Directories with source and documentation files
    pub directories: Vec<DirectoryConfig>,
}

#[derive(Debug, Clone)]
pub struct DirectoryConfig {
    pub paths: Vec<String>,
    pub globs: Vec<String>,
    pub markers: Vec<String>,
}

impl HyperlitConfig {
    pub fn from_string(string: &str) -> HyperlitResult<Self> {
        let toml = parse(string)?;
        let table = toml.as_table().ok_or_else(|| err!("not a table"))?;
        let directories = parse_directories(table.get("directory").and_then(|x| x.as_array()))?;
        Ok(Self {
            title: get_shared_string(table, "title")?,
            config_path: table
                .get("config_path")
                .and_then(|x| x.as_str())
                .unwrap_or("hyperlit.toml")
                .to_string(),
            build_directory: get_string_or(table, "build_directory", "build")?,
            output_directory: get_string_or(table, "output_directory", "output")?,
            source_link_template: get_string(table, "source_link_template").ok(),
            directories,
        })
    }
}

fn parse_directories(
    directories_array: Option<&toml_span::value::Array>,
) -> HyperlitResult<Vec<DirectoryConfig>> {
    let mut directories = vec![];
    let Some(directories_array) = directories_array else {
        return Ok(directories);
    };
    for directory in directories_array {
        let table = directory
            .as_table()
            .ok_or_else(|| err!("directory is not a table"))?;

        directories.push(DirectoryConfig {
            paths: get_string_array(table, "paths")?,
            globs: get_string_array(table, "globs")?,
            markers: get_string_array_or(table, "markers", &[])?,
        })
    }
    Ok(directories)
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

/// Helper method to get a string value from a TOML table
fn get_shared_string(table: &toml_span::value::Table, key: &str) -> HyperlitResult<SharedString> {
    table
        .get(key)
        .ok_or_else(|| err!("{} not found", key))?
        .as_str()
        .ok_or_else(|| err!("{} is not a string", key))
        .map(|s| s.into())
}

/// Helper method to get a string value from a TOML table
#[allow(unused)]
fn get_string_maybe(table: &toml_span::value::Table, key: &str) -> HyperlitResult<Option<String>> {
    Ok(match table.get(key) {
        None => None,
        Some(value) => Some(
            value
                .as_str()
                .ok_or_else(|| err!("{} is not a string", key))
                .map(|s| s.to_string())?,
        ),
    })
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
        "#,
        )?;

        expect![[r#"
            HyperlitConfig {
                title: "foo",
                config_path: "hyperlit.toml",
                build_directory: "build",
                output_directory: "output",
                source_link_template: None,
                directories: [],
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
            build_directory = "the_build"
            output_directory = "the_output"

            [[directory]]
            paths = ["docs", "docs2"]
            globs = ["*.md", "*.mdx"]

            [[directory]]
            paths = ["src", "ui/src"]
            globs = ["*.rs", "*.ts"]
            markers = ["DOC"]

        "#,
        )?;

        expect![[r#"
            HyperlitConfig {
                title: "foo",
                config_path: "hyperlit.toml",
                build_directory: "the_build",
                output_directory: "the_output",
                source_link_template: None,
                directories: [
                    DirectoryConfig {
                        paths: [
                            "docs",
                            "docs2",
                        ],
                        globs: [
                            "*.md",
                            "*.mdx",
                        ],
                        markers: [],
                    },
                    DirectoryConfig {
                        paths: [
                            "src",
                            "ui/src",
                        ],
                        globs: [
                            "*.rs",
                            "*.ts",
                        ],
                        markers: [
                            "DOC",
                        ],
                    },
                ],
            }
        "#]]
        .assert_debug_eq(&config);
        Ok(())
    }
}
