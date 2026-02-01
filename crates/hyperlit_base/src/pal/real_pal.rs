use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use tracing::{debug, error, info, instrument};
use walkdir::WalkDir;

use crate::{HyperlitError, HyperlitResult, err, error::ErrorKind};

use super::FilePath;
use super::http::{
    HttpBody, HttpHeaders, HttpMethod, HttpRequest, HttpResponse, HttpServerConfig,
    HttpServerHandle, HttpService, HttpStatusCode,
};
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
    #[instrument(skip(self))]
    fn build_glob_set(&self, globs: &[String]) -> HyperlitResult<GlobSet> {
        debug!("compiling {} glob patterns", globs.len());
        let mut builder = GlobSetBuilder::new();
        for (idx, glob) in globs.iter().enumerate() {
            let compiled = GlobBuilder::new(glob).build().map_err(|e| {
                debug!(index = idx, pattern = %glob, error = %e, "failed to compile glob pattern");
                err!("Invalid glob pattern '{}': {}", glob, e)
            })?;
            builder.add(compiled);
        }
        let glob_set = builder.build().map_err(|e| {
            debug!(error = %e, "failed to build glob set");
            err!("Failed to build glob set: {}", e)
        })?;
        debug!("glob set compiled successfully");
        Ok(glob_set)
    }
}

impl Pal for RealPal {
    #[instrument(skip(self), fields(path = %path))]
    fn file_exists(&self, path: &FilePath) -> HyperlitResult<bool> {
        let resolved = self.resolve_path(path);
        let exists = resolved.exists();
        debug!(exists, resolved = %resolved.display(), "checked file existence");
        Ok(exists)
    }

    #[instrument(skip(self))]
    fn read_executable_file(&self) -> HyperlitResult<Box<dyn ReadSeek + 'static>> {
        let exe_path = std::env::current_exe().map_err(|e| {
            debug!("failed to get current executable path: {}", e);
            Box::new(HyperlitError::new(ErrorKind::FileError {
                path: PathBuf::from("<current_exe>"),
                source: e,
            }))
        })?;

        debug!(path = %exe_path.display(), "opening executable file");
        let file = fs::File::open(&exe_path).map_err(|e| {
            debug!("failed to open executable: {}", e);
            Box::new(HyperlitError::new(ErrorKind::FileError {
                path: exe_path,
                source: e,
            }))
        })?;

        debug!("successfully opened executable file");
        Ok(Box::new(file))
    }

    #[instrument(skip(self), fields(path = %path))]
    fn read_file(&self, path: &FilePath) -> HyperlitResult<Box<dyn ReadSeek + 'static>> {
        let resolved = self.resolve_path(path);
        debug!(resolved = %resolved.display(), "opening file for reading");
        let file = fs::File::open(&resolved).map_err(|e| {
            debug!(error = %e, "failed to open file");
            Box::new(HyperlitError::new(ErrorKind::FileError {
                path: resolved,
                source: e,
            }))
        })?;
        debug!("file opened successfully");
        Ok(Box::new(file))
    }

    #[instrument(skip(self), fields(path = %path))]
    fn create_file(&self, path: &FilePath) -> HyperlitResult<Box<dyn Write>> {
        let resolved = self.resolve_path(path);
        debug!(resolved = %resolved.display(), "creating file");
        let file = fs::File::create(&resolved).map_err(|e| {
            debug!(error = %e, "failed to create file");
            Box::new(HyperlitError::new(ErrorKind::FileError {
                path: resolved,
                source: e,
            }))
        })?;
        debug!("file created successfully");
        Ok(Box::new(file))
    }

    #[instrument(skip(self), fields(path = %path))]
    fn create_directory_all(&self, path: &FilePath) -> HyperlitResult<()> {
        let resolved = self.resolve_path(path);
        debug!(resolved = %resolved.display(), "creating directory and parents");
        fs::create_dir_all(&resolved).map_err(|e| {
            debug!(error = %e, "failed to create directory");
            Box::new(HyperlitError::new(ErrorKind::FileError {
                path: resolved,
                source: e,
            }))
        })?;
        debug!("directory created successfully");
        Ok(())
    }

    #[instrument(skip(self), fields(path = %path))]
    fn remove_directory_all(&self, path: &FilePath) -> HyperlitResult<()> {
        let resolved = self.resolve_path(path);
        debug!(resolved = %resolved.display(), "removing directory and contents");
        fs::remove_dir_all(&resolved).map_err(|e| {
            debug!(error = %e, "failed to remove directory");
            Box::new(HyperlitError::new(ErrorKind::FileError {
                path: resolved,
                source: e,
            }))
        })?;
        debug!("directory removed successfully");
        Ok(())
    }

    #[instrument(skip(self), fields(path = %path, globs = ?globs))]
    fn walk_directory(
        &self,
        path: &FilePath,
        globs: &[String],
    ) -> HyperlitResult<Box<dyn Iterator<Item = HyperlitResult<FilePath>> + '_>> {
        let resolved = self.resolve_path(path);
        debug!(resolved = %resolved.display(), "starting directory walk");

        if !resolved.exists() {
            debug!("directory not found");
            return Err(Box::new(HyperlitError::new(ErrorKind::FileError {
                path: resolved,
                source: std::io::Error::new(std::io::ErrorKind::NotFound, "directory not found"),
            })));
        }

        debug!("building glob set from {} patterns", globs.len());
        let glob_set = self.build_glob_set(globs)?;

        // Create iterator that filters by glob patterns
        debug!("creating filtered directory iterator");
        let base_path = path.clone();
        let iter = WalkDir::new(&resolved)
            .into_iter()
            .filter_map(move |entry| {
                match entry {
                    Ok(e) => {
                        // Convert to relative path for glob matching
                        if let Ok(relative) = e.path().strip_prefix(&resolved) {
                            if glob_set.is_match(relative) {
                                // Prepend the base path to get full relative path
                                let full_relative = base_path.as_path().join(relative);
                                Some(Ok(FilePath::from(full_relative.to_string_lossy().as_ref())))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    Err(e) => {
                        debug!(error = %e, "error walking directory");
                        Some(Err(Box::new(HyperlitError::new(ErrorKind::FileError {
                            path: e
                                .path()
                                .map(|p| p.to_path_buf())
                                .unwrap_or_else(|| PathBuf::from("unknown")),
                            source: std::io::Error::other(e.to_string()),
                        }))))
                    }
                }
            });

        debug!("returning directory walk iterator");
        Ok(Box::new(iter))
    }

    #[instrument(skip(self, _callback), fields(directory = %directory, globs = ?globs))]
    fn watch_directory(
        &self,
        directory: &FilePath,
        globs: &[String],
        _callback: FileChangeCallback,
    ) -> HyperlitResult<()> {
        let resolved = self.resolve_path(directory);
        debug!(resolved = %resolved.display(), "setting up directory watch");

        if !resolved.exists() {
            debug!("directory not found");
            return Err(Box::new(HyperlitError::new(ErrorKind::FileError {
                path: resolved,
                source: std::io::Error::new(std::io::ErrorKind::NotFound, "directory not found"),
            })));
        }

        // Verify glob patterns are valid
        debug!("validating {} glob patterns", globs.len());
        self.build_glob_set(globs)?;

        // Note: Full watch_directory implementation would use notify::Watcher
        // For now, we verify the parameters are valid and return success.
        // A complete implementation would spawn a background watcher task.
        debug!("directory watch setup complete (note: not fully implemented)");

        Ok(())
    }

    fn start_http_server(
        &self,
        service: Box<dyn HttpService>,
        config: HttpServerConfig,
    ) -> HyperlitResult<HttpServerHandle> {
        let addr = config.address();
        info!(address = %addr, "starting HTTP server");

        // Create tiny_http server
        let server = tiny_http::Server::http(&addr).map_err(|e| {
            error!(error = %e, "failed to create HTTP server");
            err!("Failed to start HTTP server on {}: {}", addr, e)
        })?;

        let port = server
            .server_addr()
            .to_ip()
            .map(|ip| ip.port())
            .unwrap_or(0);
        info!(port, "HTTP server listening");

        let handle = HttpServerHandle::new(port);
        let shutdown_flag = handle.shutdown_flag().clone();
        let service: Arc<dyn HttpService> = Arc::from(service);

        // Spawn server thread
        thread::spawn(move || {
            info!("HTTP server thread started");

            loop {
                // Check for shutdown signal
                if shutdown_flag.load(std::sync::atomic::Ordering::SeqCst) {
                    info!("HTTP server received shutdown signal");
                    break;
                }

                // Accept connection with timeout to allow checking shutdown flag
                match server.recv_timeout(std::time::Duration::from_millis(100)) {
                    Ok(Some(mut request)) => {
                        // Convert tiny_http request to our HttpRequest
                        let http_request = Self::convert_request(&mut request);

                        // Call the service and handle Result
                        let response = match service.handle_request(http_request) {
                            Ok(response) => response,
                            Err(e) => Self::convert_error_to_response(&e),
                        };

                        // Convert our HttpResponse to tiny_http response
                        let tiny_response = Self::convert_response(response);

                        // Send response
                        if let Err(e) = request.respond(tiny_response) {
                            error!(error = %e, "failed to send HTTP response");
                        }
                    }
                    Ok(None) => {
                        // Timeout - continue loop to check shutdown flag
                        continue;
                    }
                    Err(e) => {
                        error!(error = %e, "error receiving HTTP request");
                        // Continue loop - don't crash the server on single request error
                    }
                }
            }

            info!("HTTP server thread stopped");
        });

        Ok(handle)
    }
}

impl RealPal {
    /// Convert a tiny_http request to our HttpRequest type.
    ///
    /// Note: This takes ownership of the tiny_http request because `as_reader()`
    /// requires mutable access to read the body.
    fn convert_request(tiny_req: &mut tiny_http::Request) -> HttpRequest {
        let method = HttpMethod::parse(tiny_req.method().as_str()).unwrap_or(HttpMethod::Get);
        let url = tiny_req.url().to_string();

        // Convert headers
        let mut headers = HttpHeaders::new();
        for header in tiny_req.headers().iter() {
            headers.insert(header.field.to_string(), header.value.to_string());
        }

        // Read body - check if there's a body to read
        let body = if tiny_req.body_length().unwrap_or(0) > 0 {
            let body_reader = tiny_req.as_reader();
            let mut body_bytes = Vec::new();
            if let Err(e) = body_reader.read_to_end(&mut body_bytes) {
                debug!(error = %e, "failed to read request body");
                HttpBody::empty()
            } else {
                HttpBody::from_bytes(body_bytes)
            }
        } else {
            HttpBody::empty()
        };

        // Build request with headers already included
        let mut request = HttpRequest::new(method, url);
        // Transfer headers
        for (key, value) in headers.all().iter() {
            request.headers_mut().insert(key.clone(), value.clone());
        }
        request.with_body(body)
    }

    /// Convert our HttpResponse to a tiny_http response.
    fn convert_response(response: HttpResponse) -> tiny_http::Response<Box<dyn Read + Send>> {
        let status_code = tiny_http::StatusCode::from(response.status().as_u16());

        // Debug: Log all headers in the response
        debug!(
            status = response.status().as_u16(),
            "Converting HTTP response to tiny_http"
        );
        for (key, value) in response.headers().all().iter() {
            debug!(header_key = key, header_value = value, "Response header");
        }

        // Convert headers
        let mut tiny_headers: Vec<tiny_http::Header> = Vec::new();
        for (key, value) in response.headers().all().iter() {
            debug!(
                key = key,
                value = value,
                "Adding header to tiny_http response"
            );
            tiny_headers.push(
                tiny_http::Header::from_bytes(key.as_bytes(), value.as_bytes()).unwrap_or_else(
                    |_| {
                        error!(
                            key = key,
                            value = value,
                            "Failed to create tiny_http header"
                        );
                        tiny_http::Header::from_bytes(b"X-Invalid", b"true").unwrap()
                    },
                ),
            );
        }

        // Add Content-Length header if body is present
        let body_bytes = response.body().as_bytes().to_vec();
        if !body_bytes.is_empty() {
            tiny_headers.push(
                tiny_http::Header::from_bytes(
                    b"Content-Length",
                    body_bytes.len().to_string().as_bytes(),
                )
                .unwrap(),
            );
        }

        let body_reader: Box<dyn Read + Send> = Box::new(std::io::Cursor::new(body_bytes));

        tiny_http::Response::new(status_code, tiny_headers, body_reader, None, None)
    }

    /* 📖 # Why use HTTP 599 for all errors?

    Using HTTP 599 (Network Connect Timeout Error) for all error responses provides:

    1. **Clear distinction**: 599 is rarely used in practice, making it easy to identify
       hyperlit-specific errors vs standard HTTP errors
    2. **Consistency**: All errors look the same, simplifying client error handling
    3. **Debugging**: The full error context is included in the response body for debugging
    4. **No ambiguity**: Unlike 500 (Internal Server Error) which could mean anything,
       599 clearly signals "something went wrong in the service layer"

    The error response includes the full error context in a JSON format:
    `{"error": "full error message with context"}`
    */

    /// Convert a HyperlitError to an HTTP 599 error response.
    ///
    /// This method converts any service error into a standardized HTTP error response
    /// with status 599 and a JSON body containing the full error message.
    fn convert_error_to_response(error: &crate::HyperlitError) -> HttpResponse {
        // Build error JSON with the full error message including context
        let error_message = error.to_string();
        let json_body = format!(r#"{{"error":"{}"}}"#, Self::escape_json(&error_message));

        HttpResponse::new(HttpStatusCode::NetworkConnectTimeoutError)
            .with_content_type("application/json")
            .with_body(json_body)
    }

    /// Escape special characters for JSON string values.
    fn escape_json(s: &str) -> String {
        s.chars()
            .map(|c| match c {
                '"' => "\\\"".to_string(),
                '\\' => "\\\\".to_string(),
                '\n' => "\\n".to_string(),
                '\r' => "\\r".to_string(),
                '\t' => "\\t".to_string(),
                '\u{08}' => "\\b".to_string(),
                '\u{0C}' => "\\f".to_string(),
                c => c.to_string(),
            })
            .collect()
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
