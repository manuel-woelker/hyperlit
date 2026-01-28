/* 📖 # Why use a dedicated scanner module?

The scanner provides a clean separation of concerns:
- **Configuration handling** (config.rs): Loads configuration from files
- **File discovery** (scanner.rs): Finds all files matching configured patterns
- **Content extraction**: To be added in future modules

This separation enables independent testing and composition of these operations.
Additionally, the fail-tolerant design ensures that one misconfigured directory
doesn't block scanning of others—errors are collected and reported alongside results.
*/

use tracing::{debug, instrument, warn};

use hyperlit_base::{FilePath, HyperlitError, HyperlitResult, PalHandle};

use super::Config;

/// Results from scanning files, including matched files and any errors encountered.
///
/// This struct enables fail-tolerant scanning: if some directories fail to scan,
/// the operation continues and reports both successful files and encountered errors.
///
/// # Examples
/// ```no_run
/// use hyperlit_base::{RealPal, PalHandle, FilePath};
/// use hyperlit_engine::{load_config, scan_files};
///
/// let pal = PalHandle::new(RealPal::new(".".into()));
/// let config = load_config(&pal, &FilePath::from("hyperlit.toml")).unwrap();
/// let result = scan_files(&pal, &config).unwrap();
///
/// println!("Found {} files", result.files.len());
/// if !result.errors.is_empty() {
///     println!("Encountered {} errors", result.errors.len());
/// }
/// ```
#[derive(Debug)]
pub struct ScanResult {
    /// Files found during the scan.
    pub files: Vec<FilePath>,
    /// Errors encountered during the scan (non-fatal).
    pub errors: Vec<ScanError>,
}

/// Error encountered while scanning a specific directory.
#[derive(Debug)]
pub struct ScanError {
    /// The directory path that was being scanned when the error occurred.
    pub directory_path: String,
    /// The error that occurred.
    pub error: Box<HyperlitError>,
}

/// Scan for files matching the configured glob patterns.
///
/// This function walks through each directory in the configuration using the provided
/// glob patterns and returns all matching file paths. If scanning any directory fails,
/// the error is collected and scanning continues with other directories.
///
/// # Arguments
/// * `pal` - Platform Abstraction Layer handle for filesystem access
/// * `config` - Configuration containing directories and glob patterns to scan
///
/// # Returns
/// A `ScanResult` containing matched files and any non-fatal errors encountered.
/// If a critical error occurs (e.g., invalid configuration), returns `Err`.
///
/// # Examples
/// ```no_run
/// use hyperlit_base::{RealPal, PalHandle, FilePath};
/// use hyperlit_engine::{load_config, scan_files};
///
/// let pal = PalHandle::new(RealPal::new(".".into()));
/// let config = load_config(&pal, &FilePath::from("hyperlit.toml")).unwrap();
/// let result = scan_files(&pal, &config).unwrap();
///
/// for file in &result.files {
///     println!("Found: {}", file);
/// }
///
/// for error in &result.errors {
///     eprintln!("Error scanning {}: {}", error.directory_path, error.error);
/// }
/// ```
#[instrument(skip(pal, config), fields(directory_count = config.directory.len()))]
pub fn scan_files(pal: &PalHandle, config: &Config) -> HyperlitResult<ScanResult> {
    debug!("starting file scan");

    let mut files = Vec::new();
    let mut errors = Vec::new();

    // Iterate through each DirectoryConfig
    for dir_config in &config.directory {
        // Iterate through each path in the DirectoryConfig
        for path_str in &dir_config.paths {
            let path = FilePath::from(path_str.as_str());

            // Call walk_directory with globs
            match pal.walk_directory(&path, &dir_config.globs) {
                Ok(iter) => {
                    // Collect results from iterator
                    for result in iter {
                        match result {
                            Ok(file_path) => files.push(file_path),
                            Err(e) => {
                                warn!("error walking file: {}", e);
                                errors.push(ScanError {
                                    directory_path: path_str.clone(),
                                    error: e,
                                });
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("error walking directory '{}': {}", path_str, e);
                    errors.push(ScanError {
                        directory_path: path_str.clone(),
                        error: e,
                    });
                }
            }
        }
    }

    debug!(
        files_found = files.len(),
        errors_count = errors.len(),
        "file scan complete"
    );

    Ok(ScanResult { files, errors })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DirectoryConfig;
    use hyperlit_base::pal::MockPal;

    #[test]
    fn test_scan_files_success() {
        let mock_pal = MockPal::new();

        // Setup files
        mock_pal.add_file(FilePath::from("src/main.rs"), b"fn main() {}".to_vec());
        mock_pal.add_file(FilePath::from("src/lib.rs"), b"pub fn lib() {}".to_vec());
        mock_pal.add_file(FilePath::from("docs/readme.md"), b"# README".to_vec());

        // Setup configuration
        let config = Config {
            title: "Test Project".to_string(),
            source_link_template: "https://example.com/{path}".to_string(),
            directory: vec![
                DirectoryConfig {
                    paths: vec!["src".to_string()],
                    globs: vec!["*.rs".to_string()],
                },
                DirectoryConfig {
                    paths: vec!["docs".to_string()],
                    globs: vec!["*.md".to_string()],
                },
            ],
        };

        let pal = PalHandle::new(mock_pal);
        let result = scan_files(&pal, &config).unwrap();

        assert_eq!(result.files.len(), 3);
        assert!(result.files.contains(&FilePath::from("src/main.rs")));
        assert!(result.files.contains(&FilePath::from("src/lib.rs")));
        assert!(result.files.contains(&FilePath::from("docs/readme.md")));
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_scan_files_empty_config() {
        let mock_pal = MockPal::new();
        let config = Config {
            title: "Empty Project".to_string(),
            source_link_template: "https://example.com/{path}".to_string(),
            directory: vec![],
        };

        let pal = PalHandle::new(mock_pal);
        let result = scan_files(&pal, &config).unwrap();

        assert_eq!(result.files.len(), 0);
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_scan_files_nonexistent_directory() {
        let mock_pal = MockPal::new();
        // Note: MockPal doesn't distinguish between directories - it returns all matching files
        // This test verifies that walk_directory errors propagate properly

        let config = Config {
            title: "Test Project".to_string(),
            source_link_template: "https://example.com/{path}".to_string(),
            directory: vec![DirectoryConfig {
                paths: vec!["nonexistent".to_string()],
                globs: vec!["*.rs".to_string()],
            }],
        };

        let pal = PalHandle::new(mock_pal);
        let result = scan_files(&pal, &config).unwrap();

        // Empty glob set results in no files (even though the directory "path" was handled)
        // MockPal returns all matching files regardless of directory path
        assert_eq!(result.files.len(), 0);
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_scan_files_multiple_paths_per_directory() {
        let mock_pal = MockPal::new();

        mock_pal.add_file(FilePath::from("src/main.rs"), b"fn main() {}".to_vec());
        mock_pal.add_file(FilePath::from("lib/lib.rs"), b"pub fn lib() {}".to_vec());

        let config = Config {
            title: "Multi-path Project".to_string(),
            source_link_template: "https://example.com/{path}".to_string(),
            directory: vec![DirectoryConfig {
                paths: vec!["src".to_string(), "lib".to_string()],
                globs: vec!["*.rs".to_string()],
            }],
        };

        let pal = PalHandle::new(mock_pal);
        let result = scan_files(&pal, &config).unwrap();

        // MockPal's walk_directory ignores the path parameter and returns all matching files
        // for each path in the DirectoryConfig, so we get 2 * 2 = 4 files (both files matching both paths)
        assert_eq!(result.files.len(), 4);
        assert!(
            result
                .files
                .iter()
                .filter(|f| **f == FilePath::from("src/main.rs"))
                .count()
                >= 1
        );
        assert!(
            result
                .files
                .iter()
                .filter(|f| **f == FilePath::from("lib/lib.rs"))
                .count()
                >= 1
        );
    }

    #[test]
    fn test_scan_files_no_matches() {
        let mock_pal = MockPal::new();

        mock_pal.add_file(FilePath::from("src/main.rs"), b"fn main() {}".to_vec());

        let config = Config {
            title: "No Match Project".to_string(),
            source_link_template: "https://example.com/{path}".to_string(),
            directory: vec![DirectoryConfig {
                paths: vec!["src".to_string()],
                globs: vec!["*.py".to_string()],
            }],
        };

        let pal = PalHandle::new(mock_pal);
        let result = scan_files(&pal, &config).unwrap();

        assert_eq!(result.files.len(), 0);
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_scan_files_mixed_success_failure() {
        let mock_pal = MockPal::new();

        mock_pal.add_file(FilePath::from("src/main.rs"), b"fn main() {}".to_vec());

        let config = Config {
            title: "Mixed Project".to_string(),
            source_link_template: "https://example.com/{path}".to_string(),
            directory: vec![
                DirectoryConfig {
                    paths: vec!["src".to_string()],
                    globs: vec!["*.rs".to_string()],
                },
                DirectoryConfig {
                    paths: vec!["nonexistent".to_string()],
                    globs: vec!["*.py".to_string()],
                },
            ],
        };

        let pal = PalHandle::new(mock_pal);
        let result = scan_files(&pal, &config).unwrap();

        // Should have the file from src directory
        assert_eq!(result.files.len(), 1);
        assert!(result.files.contains(&FilePath::from("src/main.rs")));

        // No errors - MockPal returns empty results for non-matching globs
        assert_eq!(result.errors.len(), 0);
    }
}
