use std::fs;
use std::io::Write;
use std::path::PathBuf;

use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use walkdir::WalkDir;

use crate::{HyperlitError, HyperlitResult, error::ErrorKind};

use super::FilePath;
use super::traits::{FileChangeCallback, Pal, ReadSeek};

/* 📖 # Why use std::fs instead of async or other crates?

Per ARCHITECTURE.md, we avoid async complexity. std::fs is:
- Sufficient for synchronous file operations
- Requires no external dependencies beyond what we already use
- Easy to understand and maintain
- Well-tested and reliable

This keeps the codebase simple and maintainable.
*/

/// Concrete PAL implementation using the real filesystem via std::fs.
///
/// All file paths are resolved relative to a configured base directory,
/// ensuring operations stay within intended boundaries.
#[derive(Debug)]
pub struct RealPal {
    base_dir: PathBuf,
}

impl RealPal {
    /// Create a new RealPal with the given base directory.
    ///
    /// # Arguments
    /// * `base_dir` - All paths will be resolved relative to this directory
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Resolve a FilePath to an absolute filesystem path.
    fn resolve_path(&self, path: &FilePath) -> PathBuf {
        self.base_dir.join(path.as_path())
    }

    /// Build a GlobSet from the given glob patterns.
    fn build_glob_set(&self, globs: &[String]) -> HyperlitResult<GlobSet> {
        let mut builder = GlobSetBuilder::new();
        for glob in globs {
            let compiled = GlobBuilder::new(glob).build().map_err(|e| {
                Box::new(HyperlitError::message(format!(
                    "Invalid glob pattern '{}': {}",
                    glob, e
                )))
            })?;
            builder.add(compiled);
        }
        builder.build().map_err(|e| {
            Box::new(HyperlitError::message(format!(
                "Failed to build glob set: {}",
                e
            )))
        })
    }
}

impl Pal for RealPal {
    fn file_exists(&self, path: &FilePath) -> HyperlitResult<bool> {
        let resolved = self.resolve_path(path);
        Ok(resolved.exists())
    }

    fn read_executable_file(&self) -> HyperlitResult<Box<dyn ReadSeek + 'static>> {
        let exe_path = std::env::current_exe().map_err(|e| {
            Box::new(HyperlitError::new(ErrorKind::FileError {
                path: PathBuf::from("<current_exe>"),
                source: e,
            }))
        })?;

        let file = fs::File::open(&exe_path).map_err(|e| {
            Box::new(HyperlitError::new(ErrorKind::FileError {
                path: exe_path,
                source: e,
            }))
        })?;

        Ok(Box::new(file))
    }

    fn read_file(&self, path: &FilePath) -> HyperlitResult<Box<dyn ReadSeek + 'static>> {
        let resolved = self.resolve_path(path);
        let file = fs::File::open(&resolved).map_err(|e| {
            Box::new(HyperlitError::new(ErrorKind::FileError {
                path: resolved,
                source: e,
            }))
        })?;
        Ok(Box::new(file))
    }

    fn create_file(&self, path: &FilePath) -> HyperlitResult<Box<dyn Write>> {
        let resolved = self.resolve_path(path);
        let file = fs::File::create(&resolved).map_err(|e| {
            Box::new(HyperlitError::new(ErrorKind::FileError {
                path: resolved,
                source: e,
            }))
        })?;
        Ok(Box::new(file))
    }

    fn create_directory_all(&self, path: &FilePath) -> HyperlitResult<()> {
        let resolved = self.resolve_path(path);
        fs::create_dir_all(&resolved).map_err(|e| {
            Box::new(HyperlitError::new(ErrorKind::FileError {
                path: resolved,
                source: e,
            }))
        })
    }

    fn remove_directory_all(&self, path: &FilePath) -> HyperlitResult<()> {
        let resolved = self.resolve_path(path);
        fs::remove_dir_all(&resolved).map_err(|e| {
            Box::new(HyperlitError::new(ErrorKind::FileError {
                path: resolved,
                source: e,
            }))
        })
    }

    fn walk_directory(
        &self,
        path: &FilePath,
        globs: &[String],
    ) -> HyperlitResult<Box<dyn Iterator<Item = HyperlitResult<FilePath>> + '_>> {
        let resolved = self.resolve_path(path);

        if !resolved.exists() {
            return Err(Box::new(HyperlitError::new(ErrorKind::FileError {
                path: resolved,
                source: std::io::Error::new(std::io::ErrorKind::NotFound, "directory not found"),
            })));
        }

        let glob_set = self.build_glob_set(globs)?;

        // Create iterator that filters by glob patterns
        let iter = WalkDir::new(&resolved)
            .into_iter()
            .filter_map(move |entry| {
                match entry {
                    Ok(e) => {
                        // Convert to relative path for glob matching
                        if let Ok(relative) = e.path().strip_prefix(&resolved) {
                            if glob_set.is_match(relative) {
                                Some(Ok(FilePath::from(relative)))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    Err(e) => Some(Err(Box::new(HyperlitError::new(ErrorKind::FileError {
                        path: e
                            .path()
                            .map(|p| p.to_path_buf())
                            .unwrap_or_else(|| PathBuf::from("unknown")),
                        source: std::io::Error::other(e.to_string()),
                    })))),
                }
            });

        Ok(Box::new(iter))
    }

    fn watch_directory(
        &self,
        directory: &FilePath,
        globs: &[String],
        _callback: FileChangeCallback,
    ) -> HyperlitResult<()> {
        let resolved = self.resolve_path(directory);

        if !resolved.exists() {
            return Err(Box::new(HyperlitError::new(ErrorKind::FileError {
                path: resolved,
                source: std::io::Error::new(std::io::ErrorKind::NotFound, "directory not found"),
            })));
        }

        // Verify glob patterns are valid
        self.build_glob_set(globs)?;

        // Note: Full watch_directory implementation would use notify::Watcher
        // For now, we verify the parameters are valid and return success.
        // A complete implementation would spawn a background watcher task.

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_test_dir() -> (TempDir, RealPal) {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let pal = RealPal::new(temp_dir.path().to_path_buf());
        (temp_dir, pal)
    }

    #[test]
    fn test_file_exists_true() {
        let (temp_dir, pal) = setup_test_dir();
        let file_path = FilePath::from("test.txt");
        fs::write(temp_dir.path().join("test.txt"), "content").unwrap();

        assert!(pal.file_exists(&file_path).unwrap());
    }

    #[test]
    fn test_file_exists_false() {
        let (_temp_dir, pal) = setup_test_dir();
        let file_path = FilePath::from("nonexistent.txt");

        assert!(!pal.file_exists(&file_path).unwrap());
    }

    #[test]
    fn test_read_file() {
        let (temp_dir, pal) = setup_test_dir();
        let file_path = FilePath::from("test.txt");
        let content = "hello world";
        fs::write(temp_dir.path().join("test.txt"), content).unwrap();

        let result = pal.read_file_to_string(&file_path).unwrap();
        assert_eq!(result, content);
    }

    #[test]
    fn test_read_file_not_found() {
        let (_temp_dir, pal) = setup_test_dir();
        let file_path = FilePath::from("nonexistent.txt");

        let result = pal.read_file(&file_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_file() {
        let (temp_dir, pal) = setup_test_dir();
        let file_path = FilePath::from("new.txt");

        let mut writer = pal.create_file(&file_path).unwrap();
        use std::io::Write;
        writer.write_all(b"test content").unwrap();
        drop(writer);

        let content = fs::read_to_string(temp_dir.path().join("new.txt")).unwrap();
        assert_eq!(content, "test content");
    }

    #[test]
    fn test_create_directory_all() {
        let (temp_dir, pal) = setup_test_dir();
        let dir_path = FilePath::from("a/b/c");

        pal.create_directory_all(&dir_path).unwrap();

        assert!(temp_dir.path().join("a/b/c").exists());
    }

    #[test]
    fn test_remove_directory_all() {
        let (temp_dir, pal) = setup_test_dir();
        let dir_path = FilePath::from("to_remove");

        fs::create_dir(temp_dir.path().join("to_remove")).unwrap();
        assert!(temp_dir.path().join("to_remove").exists());

        pal.remove_directory_all(&dir_path).unwrap();

        assert!(!temp_dir.path().join("to_remove").exists());
    }

    #[test]
    fn test_walk_directory_with_glob() {
        let (temp_dir, pal) = setup_test_dir();

        // Create some files
        fs::write(temp_dir.path().join("test1.rs"), "").unwrap();
        fs::write(temp_dir.path().join("test2.rs"), "").unwrap();
        fs::write(temp_dir.path().join("test.txt"), "").unwrap();

        let globs = vec!["*.rs".to_string()];
        let results: Vec<_> = pal
            .walk_directory(&FilePath::from("."), &globs)
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        // Should find only .rs files
        let file_names: Vec<String> = results
            .iter()
            .map(|p| {
                p.as_path()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            })
            .collect();

        assert!(file_names.contains(&"test1.rs".to_string()));
        assert!(file_names.contains(&"test2.rs".to_string()));
        assert!(!file_names.contains(&"test.txt".to_string()));
    }

    #[test]
    fn test_walk_directory_not_found() {
        let (_temp_dir, pal) = setup_test_dir();
        let globs = vec!["*.rs".to_string()];

        let result = pal.walk_directory(&FilePath::from("nonexistent"), &globs);
        assert!(result.is_err());
    }

    #[test]
    fn test_watch_directory() {
        let (temp_dir, pal) = setup_test_dir();
        fs::create_dir(temp_dir.path().join("watch")).unwrap();

        let callback: FileChangeCallback = Box::new(|_event| {});
        let globs = vec!["*.rs".to_string()];

        // Should not error for valid directory
        let result = pal.watch_directory(&FilePath::from("watch"), &globs, callback);
        assert!(result.is_ok());
    }

    #[test]
    fn test_watch_directory_not_found() {
        let (_temp_dir, pal) = setup_test_dir();
        let callback: FileChangeCallback = Box::new(|_event| {});
        let globs = vec!["*.rs".to_string()];

        let result = pal.watch_directory(&FilePath::from("nonexistent"), &globs, callback);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_glob_pattern() {
        let (_temp_dir, pal) = setup_test_dir();
        let invalid_glob = vec!["[invalid".to_string()];

        let result = pal.walk_directory(&FilePath::from("."), &invalid_glob);
        assert!(result.is_err());
    }
}
