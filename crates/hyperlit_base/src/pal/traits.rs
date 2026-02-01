use std::io::{Read, Seek, Write};
use std::sync::Arc;

use crate::HyperlitResult;

use super::file_path::FilePath;
use super::http::{HttpServerConfig, HttpServerHandle, HttpService};

/* 📖 # What is the Platform Abstraction Layer (PAL)?

The PAL provides a trait-based abstraction over filesystem operations, enabling:
- Testable code: MockPal allows deterministic unit tests without filesystem access
- Flexibility: Switch between real filesystem and in-memory implementations
- Consistency: All filesystem operations use the same error handling

This follows the Dependency Inversion Principle—code depends on abstractions (Pal trait),
not concrete implementations (RealPal or MockPal).
*/

/// Trait combining Read + Seek for file operations.
///
/// This trait enables returning opaque file handles that support both reading
/// and seeking, useful for different implementations (real files, in-memory buffers, etc.)
pub trait ReadSeek: Read + Seek {}
impl<T: Read + Seek> ReadSeek for T {}

/// File change event returned when watching directories.
#[derive(Debug, Clone)]
pub struct FileChangeEvent {
    /// List of file paths that changed.
    pub changed_files: Vec<FilePath>,
}

/// Callback invoked when watched files change.
pub type FileChangeCallback = Box<dyn Fn(FileChangeEvent) + Send + Sync>;

/* 📖 # Why is Pal a trait instead of a struct?

Using a trait enables two key benefits:
1. **Testability**: MockPal implements Pal for fast, deterministic tests without filesystem side effects
2. **Flexibility**: Code depends on the abstraction, not the concrete implementation

This is the Dependency Inversion Principle applied to filesystem operations.
*/

/// Platform Abstraction Layer (PAL) trait providing filesystem operations.
///
/// Implement this trait to provide custom filesystem behavior. Two implementations
/// are provided:
/// - `RealPal`: Uses the real filesystem via `std::fs`
/// - `MockPal`: In-memory implementation for testing
pub trait Pal: std::fmt::Debug + Send + Sync + 'static {
    /// Check if a file exists at the given path.
    fn file_exists(&self, path: &FilePath) -> HyperlitResult<bool>;

    /// Read the executable file (current program binary).
    fn read_executable_file(&self) -> HyperlitResult<Box<dyn ReadSeek + 'static>>;

    /// Open a file for reading.
    fn read_file(&self, path: &FilePath) -> HyperlitResult<Box<dyn ReadSeek + 'static>>;

    /// Read entire file contents as a UTF-8 string.
    ///
    /// This is a convenience method with a default implementation. It reads the file,
    /// validates UTF-8, and returns the string or an error.
    fn read_file_to_string(&self, path: &FilePath) -> HyperlitResult<String> {
        use std::io::Read;
        let mut reader = self.read_file(path)?;
        let mut contents = Vec::new();
        reader.read_to_end(&mut contents).map_err(|e| {
            Box::new(crate::HyperlitError::new(
                crate::error::ErrorKind::FileError {
                    path: path.as_path().to_path_buf(),
                    source: e,
                },
            ))
        })?;
        String::from_utf8(contents).map_err(|_e| crate::err!("File is not valid UTF-8: {}", path))
    }

    /// Create a new file, overwriting if it exists.
    fn create_file(&self, path: &FilePath) -> HyperlitResult<Box<dyn Write>>;

    /// Create a directory and all parent directories.
    fn create_directory_all(&self, path: &FilePath) -> HyperlitResult<()>;

    /// Remove a directory and all its contents.
    fn remove_directory_all(&self, path: &FilePath) -> HyperlitResult<()>;

    /// Walk a directory tree, yielding paths matching the given glob patterns.
    ///
    /// # Arguments
    /// * `path` - Directory to walk
    /// * `globs` - Glob patterns to match (e.g., `["*.rs", "!*.bak"]`)
    ///
    /// Returns an iterator of FilePath results that match any of the patterns.
    fn walk_directory(
        &self,
        path: &FilePath,
        globs: &[String],
    ) -> HyperlitResult<Box<dyn Iterator<Item = HyperlitResult<FilePath>> + '_>>;

    /// Watch a directory for file changes.
    ///
    /// # Arguments
    /// * `directory` - Directory to watch
    /// * `globs` - Glob patterns to match (e.g., `["*.rs"]`)
    /// * `callback` - Function called when files change
    ///
    /// Returns immediately; the callback will be invoked asynchronously when changes occur.
    fn watch_directory(
        &self,
        directory: &FilePath,
        globs: &[String],
        callback: FileChangeCallback,
    ) -> HyperlitResult<()>;

    /// Start an HTTP server with the given service.
    ///
    /// # Arguments
    /// * `service` - The HTTP service that will handle incoming requests
    /// * `config` - Server configuration (host, port, etc.)
    ///
    /// Returns a handle to the running server. The server will start immediately
    /// and listen for connections. When the handle is dropped (or shutdown() is called),
    /// the server will stop accepting new connections and shut down gracefully.
    fn start_http_server(
        &self,
        service: Box<dyn HttpService>,
        config: HttpServerConfig,
    ) -> HyperlitResult<HttpServerHandle>;
}

/* 📖 # Why use Arc<dyn Pal> with PalHandle?

Arc enables cheap cloning of the entire PAL implementation, allowing it to be
shared across multiple parts of the application (thread-safe via dyn Pal bounds).
PalHandle wraps this for ergonomic Deref access and Clone support.
This pattern avoids lifetime parameters and enables flexible PAL passing through
the codebase.
*/

/// Handle to a PAL implementation, enabling shared ownership.
///
/// Internally wraps `Arc<dyn Pal>` for cheap cloning and thread-safe sharing.
/// Can be cloned and passed around freely without lifetime concerns.
///
/// # Examples
///
/// ```no_run
/// use hyperlit_base::{RealPal, PalHandle};
///
/// let pal = PalHandle::new(RealPal::new(".".into()));
/// let pal_clone = pal.clone(); // Cheap clone, shares the same implementation
/// ```
#[derive(Debug, Clone)]
pub struct PalHandle(Arc<dyn Pal>);

impl PalHandle {
    /// Create a new PalHandle from a Pal implementation.
    pub fn new(pal: impl Pal + 'static) -> Self {
        Self(Arc::new(pal))
    }
}

impl std::ops::Deref for PalHandle {
    type Target = dyn Pal;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_change_event_creation() {
        let paths = vec![FilePath::from("test1.rs"), FilePath::from("test2.rs")];
        let event = FileChangeEvent {
            changed_files: paths.clone(),
        };
        assert_eq!(event.changed_files.len(), 2);
        assert_eq!(event.changed_files[0], FilePath::from("test1.rs"));
    }

    #[test]
    fn test_pal_handle_clone() {
        use crate::pal::mock::MockPal;
        let pal = PalHandle::new(MockPal::new());
        let _pal_clone = pal.clone();
        // Should not panic, clone works
    }
}
