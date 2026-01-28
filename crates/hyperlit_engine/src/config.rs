use serde::Deserialize;

use hyperlit_base::{FilePath, HyperlitError, HyperlitResult, PalHandle};

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

/// Load a configuration file from the filesystem using the PAL.
///
/// Reads a TOML configuration file from the given path and deserializes it into a Config struct.
///
/// # Arguments
/// * `pal` - Platform Abstraction Layer handle for filesystem access
/// * `path` - Path to the configuration file
///
/// # Errors
/// Returns an error if the file cannot be read or if the TOML is invalid.
///
/// # Examples
/// ```no_run
/// use hyperlit_base::{RealPal, PalHandle, FilePath};
/// use hyperlit_engine::load_config;
///
/// let pal = PalHandle::new(RealPal::new(".".into()));
/// let config = load_config(&pal, &FilePath::from("hyperlit.toml")).unwrap();
/// println!("Config title: {}", config.title);
/// ```
pub fn load_config(pal: &PalHandle, path: &FilePath) -> HyperlitResult<Config> {
    let content = pal.read_file_to_string(path)?;
    toml::from_str(&content).map_err(|e| {
        Box::new(HyperlitError::message(format!(
            "Failed to parse configuration file '{}': {}",
            path, e
        )))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyperlit_base::pal::MockPal;

    #[test]
    fn test_load_config_success() {
        let mock_pal = MockPal::new();
        let config_content = r#"
title = "My Documentation"
source_link_template = "https://github.com/user/repo/blob/main/{path}#L{line}"

[[directory]]
paths = ["src"]
globs = ["*.rs"]

[[directory]]
paths = ["docs"]
globs = ["*.md"]
"#;

        let path = FilePath::from("hyperlit.toml");
        mock_pal.add_file(path.clone(), config_content.as_bytes().to_vec());

        let pal = PalHandle::new(mock_pal);
        let config = load_config(&pal, &path).unwrap();
        assert_eq!(config.title, "My Documentation");
        assert_eq!(
            config.source_link_template,
            "https://github.com/user/repo/blob/main/{path}#L{line}"
        );
        assert_eq!(config.directory.len(), 2);
        assert_eq!(config.directory[0].paths, vec!["src"]);
        assert_eq!(config.directory[0].globs, vec!["*.rs"]);
        assert_eq!(config.directory[1].paths, vec!["docs"]);
        assert_eq!(config.directory[1].globs, vec!["*.md"]);
    }

    #[test]
    fn test_load_config_missing_file() {
        let mock_pal = MockPal::new();
        let pal = PalHandle::new(mock_pal);
        let path = FilePath::from("nonexistent.toml");

        let result = load_config(&pal, &path);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_config_invalid_toml() {
        let mock_pal = MockPal::new();
        let invalid_content = "this is not valid toml: [[[";

        let path = FilePath::from("invalid.toml");
        mock_pal.add_file(path.clone(), invalid_content.as_bytes().to_vec());

        let pal = PalHandle::new(mock_pal);
        let result = load_config(&pal, &path);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_config_minimal() {
        let mock_pal = MockPal::new();
        let config_content = r#"
title = "Minimal Doc"
source_link_template = "https://example.com"
"#;

        let path = FilePath::from("minimal.toml");
        mock_pal.add_file(path.clone(), config_content.as_bytes().to_vec());

        let pal = PalHandle::new(mock_pal);
        let config = load_config(&pal, &path).unwrap();
        assert_eq!(config.title, "Minimal Doc");
        assert_eq!(config.directory.len(), 0);
    }
}
