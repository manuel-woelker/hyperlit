use serde::Deserialize;

/// Configuration for a Hyperlit documentation site.
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Title of the documentation site.
    pub title: String,
    /// Template for generating source code links.
    pub source_link_template: String,
    /// Custom directory configurations.
    #[serde(default)]
    pub directory: Vec<DirectoryConfig>,
}

/// Configuration for a specific directory within the site.
#[derive(Debug, Deserialize)]
pub struct DirectoryConfig {
    /// Paths to the directory.
    pub paths: Vec<String>,
    /// Glob patterns for files in this directory.
    pub globs: Vec<String>,
}
