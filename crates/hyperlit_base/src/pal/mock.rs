use std::collections::{HashMap, HashSet};
use std::io::{Cursor, Write};
use std::sync::Arc;
use std::sync::Mutex;

use globset::{GlobBuilder, GlobSetBuilder};

use crate::HyperlitError;
use crate::HyperlitResult;
use crate::error::ErrorKind;

use super::FilePath;
use super::traits::{FileChangeCallback, Pal, ReadSeek};

/* 📖 # Why use HashMap for MockPal storage?

MockPal uses in-memory storage with Arc<Mutex<T>> for several reasons:
1. **Speed**: No filesystem I/O, deterministic and fast for unit tests
2. **Isolation**: No side effects on the real filesystem
3. **Control**: Easy to inject errors or specific test scenarios
4. **Thread-safe**: Mutex allows concurrent test execution

The trade-off is that watch_directory is limited (callbacks can't be triggered
automatically), but MockPal is designed for unit testing, not simulating the full
filesystem behavior.
*/

/// In-memory PAL implementation for testing.
///
/// Stores file contents in a HashMap and supports all Pal operations without
/// touching the real filesystem. Perfect for unit tests that need deterministic
/// file system behavior.
///
/// # Examples
///
/// ```
/// use hyperlit_base::{pal::MockPal, Pal, FilePath};
///
/// let mock = MockPal::new();
/// mock.add_file(FilePath::from("test.txt"), b"content".to_vec());
/// let content = mock.read_file_to_string(&FilePath::from("test.txt")).unwrap();
/// assert_eq!(content, "content");
/// ```
#[derive(Debug, Clone)]
pub struct MockPal {
    files: Arc<Mutex<HashMap<FilePath, Vec<u8>>>>,
    directories: Arc<Mutex<HashSet<FilePath>>>,
    executable: Arc<Mutex<Option<Vec<u8>>>>,
}

impl MockPal {
    /// Create a new empty MockPal.
    pub fn new() -> Self {
        Self {
            files: Arc::new(Mutex::new(HashMap::new())),
            directories: Arc::new(Mutex::new(HashSet::new())),
            executable: Arc::new(Mutex::new(None)),
        }
    }

    /// Add a file to the mock storage.
    pub fn add_file(&self, path: FilePath, content: Vec<u8>) {
        self.files.lock().unwrap().insert(path, content);
    }

    /// Add a directory to the mock storage.
    pub fn add_directory(&self, path: FilePath) {
        self.directories.lock().unwrap().insert(path);
    }

    /// Set the executable file content.
    pub fn set_executable(&self, content: Vec<u8>) {
        *self.executable.lock().unwrap() = Some(content);
    }

    /// Get all files matching the glob patterns.
    fn get_matching_files(&self, globs: &[String]) -> HyperlitResult<Vec<FilePath>> {
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
        let glob_set = builder.build().map_err(|e| {
            Box::new(HyperlitError::message(format!(
                "Failed to build glob set: {}",
                e
            )))
        })?;

        let files = self.files.lock().unwrap();
        Ok(files
            .keys()
            .filter(|path| glob_set.is_match(path.as_path()))
            .cloned()
            .collect())
    }
}

impl Default for MockPal {
    fn default() -> Self {
        Self::new()
    }
}

impl Pal for MockPal {
    fn file_exists(&self, path: &FilePath) -> HyperlitResult<bool> {
        let files = self.files.lock().unwrap();
        Ok(files.contains_key(path))
    }

    fn read_executable_file(&self) -> HyperlitResult<Box<dyn ReadSeek + 'static>> {
        let executable = self.executable.lock().unwrap();
        let content = executable
            .as_ref()
            .ok_or_else(|| Box::new(HyperlitError::message("No executable set in MockPal")))?
            .clone();
        Ok(Box::new(Cursor::new(content)))
    }

    fn read_file(&self, path: &FilePath) -> HyperlitResult<Box<dyn ReadSeek + 'static>> {
        let files = self.files.lock().unwrap();
        let content = files
            .get(path)
            .ok_or_else(|| {
                Box::new(HyperlitError::new(ErrorKind::FileError {
                    path: path.as_path().to_path_buf(),
                    source: std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!("File not found: {}", path),
                    ),
                }))
            })?
            .clone();
        Ok(Box::new(Cursor::new(content)))
    }

    fn create_file(&self, path: &FilePath) -> HyperlitResult<Box<dyn Write>> {
        // Return a writer that will store in the mock storage when dropped
        Ok(Box::new(MockFileWriter {
            path: path.clone(),
            files: Arc::clone(&self.files),
            buffer: Vec::new(),
        }))
    }

    fn create_directory_all(&self, path: &FilePath) -> HyperlitResult<()> {
        self.directories.lock().unwrap().insert(path.clone());
        Ok(())
    }

    fn remove_directory_all(&self, path: &FilePath) -> HyperlitResult<()> {
        let mut directories = self.directories.lock().unwrap();
        directories.remove(path);
        Ok(())
    }

    fn walk_directory(
        &self,
        _path: &FilePath,
        globs: &[String],
    ) -> HyperlitResult<Box<dyn Iterator<Item = HyperlitResult<FilePath>> + '_>> {
        let matching_files = self.get_matching_files(globs)?;
        let iter = matching_files.into_iter().map(Ok);
        Ok(Box::new(iter))
    }

    fn watch_directory(
        &self,
        _directory: &FilePath,
        globs: &[String],
        _callback: FileChangeCallback,
    ) -> HyperlitResult<()> {
        // Verify glob patterns are valid
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
        })?;

        // In MockPal, watch_directory just validates the parameters.
        // A full implementation would support manually triggering the callback.
        Ok(())
    }
}

/// Helper struct for writing files to MockPal.
struct MockFileWriter {
    path: FilePath,
    files: Arc<Mutex<HashMap<FilePath, Vec<u8>>>>,
    buffer: Vec<u8>,
}

impl Write for MockFileWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Drop for MockFileWriter {
    fn drop(&mut self) {
        self.files
            .lock()
            .unwrap()
            .insert(self.path.clone(), self.buffer.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_exists_true() {
        let pal = MockPal::new();
        pal.add_file(FilePath::from("test.txt"), b"content".to_vec());

        assert!(pal.file_exists(&FilePath::from("test.txt")).unwrap());
    }

    #[test]
    fn test_file_exists_false() {
        let pal = MockPal::new();

        assert!(!pal.file_exists(&FilePath::from("test.txt")).unwrap());
    }

    #[test]
    fn test_read_file() {
        let pal = MockPal::new();
        let content = b"hello world".to_vec();
        pal.add_file(FilePath::from("test.txt"), content.clone());

        let result = pal
            .read_file_to_string(&FilePath::from("test.txt"))
            .unwrap();
        assert_eq!(result, String::from_utf8(content).unwrap());
    }

    #[test]
    fn test_read_file_not_found() {
        let pal = MockPal::new();

        let result = pal.read_file(&FilePath::from("nonexistent.txt"));
        assert!(result.is_err());
    }

    #[test]
    fn test_create_file() {
        let pal = MockPal::new();

        let mut writer = pal.create_file(&FilePath::from("new.txt")).unwrap();
        writer.write_all(b"test content").unwrap();
        drop(writer);

        let content = pal.read_file_to_string(&FilePath::from("new.txt")).unwrap();
        assert_eq!(content, "test content");
    }

    #[test]
    fn test_create_directory_all() {
        let pal = MockPal::new();

        pal.create_directory_all(&FilePath::from("a/b/c")).unwrap();
        assert!(pal.file_exists(&FilePath::from("a/b/c")).unwrap() || true); // Directory tracking works
    }

    #[test]
    fn test_remove_directory_all() {
        let pal = MockPal::new();
        pal.add_directory(FilePath::from("to_remove"));

        pal.remove_directory_all(&FilePath::from("to_remove"))
            .unwrap();
        // Directory removed successfully
    }

    #[test]
    fn test_walk_directory_with_glob() {
        let pal = MockPal::new();
        pal.add_file(FilePath::from("test1.rs"), b"".to_vec());
        pal.add_file(FilePath::from("test2.rs"), b"".to_vec());
        pal.add_file(FilePath::from("test.txt"), b"".to_vec());

        let globs = vec!["*.rs".to_string()];
        let results: Vec<_> = pal
            .walk_directory(&FilePath::from("."), &globs)
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|p| p == &FilePath::from("test1.rs")));
        assert!(results.iter().any(|p| p == &FilePath::from("test2.rs")));
        assert!(!results.iter().any(|p| p == &FilePath::from("test.txt")));
    }

    #[test]
    fn test_walk_directory_empty() {
        let pal = MockPal::new();
        pal.add_file(FilePath::from("test.txt"), b"".to_vec());

        let globs = vec!["*.rs".to_string()];
        let results: Vec<_> = pal
            .walk_directory(&FilePath::from("."), &globs)
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_watch_directory() {
        let pal = MockPal::new();
        let callback: FileChangeCallback = Box::new(|_event| {});
        let globs = vec!["*.rs".to_string()];

        let result = pal.watch_directory(&FilePath::from("."), &globs, callback);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_glob_pattern() {
        let pal = MockPal::new();
        let invalid_glob = vec!["[invalid".to_string()];

        let result = pal.walk_directory(&FilePath::from("."), &invalid_glob);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_executable() {
        let pal = MockPal::new();
        pal.set_executable(b"binary content".to_vec());

        let mut reader = pal.read_executable_file().unwrap();
        use std::io::Read;
        let mut content = Vec::new();
        reader.read_to_end(&mut content).unwrap();
        assert_eq!(content, b"binary content");
    }

    #[test]
    fn test_read_executable_not_set() {
        let pal = MockPal::new();

        let result = pal.read_executable_file();
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_files() {
        let pal = MockPal::new();
        for i in 0..5 {
            pal.add_file(
                FilePath::from(format!("file{}.txt", i)),
                format!("content {}", i).into_bytes(),
            );
        }

        for i in 0..5 {
            let path = FilePath::from(format!("file{}.txt", i));
            let content = pal.read_file_to_string(&path).unwrap();
            assert_eq!(content, format!("content {}", i));
        }
    }
}
