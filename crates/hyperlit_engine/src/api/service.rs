/* 📖 # Why a single unified API service?

The ApiService provides a single HTTP service that handles ALL API endpoints.
This design follows the "single service trait" pattern (Pattern A) specified in the
requirements, providing several benefits:

1. **Simplicity**: One service to register, one handle to manage, no routing complexity
2. **Consistency**: All endpoints share the same error handling and response format
3. **Extensibility**: New endpoints are added internally without changing the service interface
4. **Resource sharing**: Single store handle serves all endpoints
5. **Testing**: MockPal tests only need to register one service

The service internally routes requests to the appropriate handler based on the path:
- `/api/site` -> Site info endpoint
- `/api/documents` -> List all documents
- `/api/search?q={query}` -> Search documents
- `/api/document/{documentid}` -> Document retrieval endpoint
- All other paths -> HTTP 599 error

This pattern is used instead of separate services per endpoint because it maintains
the simple single-handler API while supporting multiple endpoints internally.
*/

/* 📖 # Why use serde for JSON serialization?

Manual JSON string construction using format!() is error-prone and difficult to maintain:
- Requires manual escaping of special characters
- Easy to create malformed JSON (missing commas, brackets)
- Type changes require updating multiple format strings
- No compile-time validation of JSON structure

Using serde with derive(Serialize) provides:
1. **Type safety**: Structs define the schema, compiler catches mismatches
2. **Automatic escaping**: serde_json handles all escaping correctly
3. **Maintainability**: Change the struct, serialization updates automatically
4. **Performance**: Optimized serialization, no string concatenation overhead

All API responses should use this pattern: define a struct, derive Serialize,
use serde_json::to_string() for conversion.
*/

/* 📖 # Why a common JSON serialization helper?

Centralizing JSON serialization through a single helper method provides several benefits:

1. **Consistent error handling**: All serialization errors are handled the same way
2. **DRY principle**: No repeated match statements for serde_json::to_string()
3. **Maintainability**: Change error handling in one place, affects all endpoints
4. **Type safety**: The generic method ensures only serializable types are accepted
5. **Testability**: Easier to test serialization logic independently

This pattern should be used for all API endpoints that return JSON responses.
The helper returns Result to allow callers to handle errors appropriately or
convert them to HTTP error responses using failure_response().
*/

use hyperlit_base::HyperlitResult;
use hyperlit_base::pal::http::{HttpBody, HttpMethod, HttpRequest, HttpResponse, HttpService};
use serde::Serialize;
use std::io::{Cursor, Read};
use tracing::{debug, error, info, warn};

use crate::document::{Document, DocumentId};
use crate::search::{MatchType, SimpleSearch};
use crate::store::StoreHandle;

/* 📖 # Why serve static files from an embedded zip?

The hyperlit binary contains UI assets appended as a zip file at the end.
This allows distribution as a single self-contained executable.

The zip structure at the end of the binary:
[Binary Code][Zip Local Files][Zip Central Directory][End of Central Directory Record]

We use the `zip` crate's ZipArchive to read the embedded assets, which provides:
- Robust zip parsing without manual byte manipulation
- Automatic handling of different compression methods
- Well-tested and maintained code
*/

/// Reads embedded zip assets from the end of the current binary.
#[derive(Clone)]
pub struct EmbeddedAssetService {
    /// The binary content containing the embedded zip
    binary_content: Vec<u8>,
}

impl EmbeddedAssetService {
    /// Create a new EmbeddedAssetService by reading the current binary.
    pub fn new() -> Option<Self> {
        debug!("Initializing EmbeddedAssetService");

        // Read the current executable
        let exe_path = match std::env::current_exe() {
            Ok(path) => {
                debug!(exe_path = ?path, "Found current executable");
                path
            }
            Err(e) => {
                error!(error = %e, "Failed to get current executable path");
                return None;
            }
        };

        let mut file = match std::fs::File::open(&exe_path) {
            Ok(f) => f,
            Err(e) => {
                error!(exe_path = ?exe_path, error = %e, "Failed to open executable");
                return None;
            }
        };

        let mut content = Vec::new();
        if let Err(e) = file.read_to_end(&mut content) {
            error!(error = %e, "Failed to read executable content");
            return None;
        }

        debug!(binary_size = content.len(), "Read binary content");

        // 📖 # Why verify a zip archive exists?
        // We need to ensure the binary actually contains an embedded zip.
        // The zip crate can read from the end of the file to find the EOCD,
        // but we first verify the EOCD signature exists to avoid creating
        // a service that will fail on every request.
        let eocd_sig: [u8; 4] = [0x50, 0x4B, 0x05, 0x06];
        match Self::find_signature_backwards(&content, &eocd_sig) {
            Some(offset) => {
                debug!(offset = offset, "Found EOCD signature");
            }
            None => {
                warn!("No embedded zip found - EOCD signature not present");
                return None;
            }
        }

        info!("EmbeddedAssetService initialized successfully");
        Some(Self {
            binary_content: content,
        })
    }

    /// Find a signature by searching backwards from the end of data.
    fn find_signature_backwards(data: &[u8], signature: &[u8]) -> Option<usize> {
        if data.len() < signature.len() {
            return None;
        }

        // Search from the end backwards
        (0..=data.len() - signature.len())
            .rev()
            .find(|&i| &data[i..i + signature.len()] == signature)
    }

    /// Extract a file from the embedded zip by path.
    fn extract_file(&self, path: &str) -> Option<Vec<u8>> {
        // Normalize path: remove leading slash
        let normalized_path = path.strip_prefix('/').unwrap_or(path);

        // If path is empty (root), serve index.html
        let lookup_path = if normalized_path.is_empty() {
            "index.html"
        } else {
            normalized_path
        };

        debug!(
            requested_path = path,
            lookup_path = lookup_path,
            "Extracting file from embedded zip"
        );

        // Use zip crate to read the embedded zip
        // ZipArchive reads from the end of the file to find the EOCD and
        // uses the central directory offsets to locate file data anywhere
        // in the binary (not just at the offset where the central directory starts)
        let cursor = Cursor::new(&self.binary_content);

        let mut archive = match zip::ZipArchive::new(cursor) {
            Ok(archive) => {
                debug!(file_count = archive.len(), "Opened zip archive");
                archive
            }
            Err(e) => {
                error!(error = %e, "Failed to open zip archive from binary");
                return None;
            }
        };

        // List all files in the archive for debugging
        debug!("Files in embedded zip:");
        for i in 0..archive.len() {
            if let Ok(file) = archive.by_index(i) {
                debug!(
                    name = file.name(),
                    compressed = file.compressed_size(),
                    uncompressed = file.size(),
                    "  - Zip entry"
                );
            }
        }

        // Find and extract the file
        let mut file = match archive.by_name(lookup_path) {
            Ok(file) => {
                debug!(name = file.name(), size = file.size(), compression = ?file.compression(), "Found file in zip");
                file
            }
            Err(e) => {
                warn!(lookup_path = lookup_path, error = %e, "File not found in embedded zip");
                return None;
            }
        };

        let mut content = Vec::new();
        if let Err(e) = file.read_to_end(&mut content) {
            error!(lookup_path = lookup_path, error = %e, "Failed to read file content from zip");
            return None;
        }

        debug!(
            lookup_path = lookup_path,
            content_size = content.len(),
            "Successfully extracted file"
        );
        Some(content)
    }

    /// Serve a static file from the embedded zip.
    pub fn serve_file(&self, path: &str) -> Option<HttpResponse> {
        debug!(path = path, "Serving static file request");

        let content = match self.extract_file(path) {
            Some(content) => content,
            None => {
                debug!(path = path, "File not found in embedded zip");
                return None;
            }
        };

        let content_type = Self::guess_content_type(path);
        info!(
            path = path,
            content_type = content_type,
            content_size = content.len(),
            "Serving static file"
        );

        Some(
            HttpResponse::ok()
                .with_content_type(content_type)
                .with_body(HttpBody::from_bytes(content)),
        )
    }

    /// Guess the MIME type based on file extension.
    fn guess_content_type(path: &str) -> &'static str {
        let path_lower = path.to_lowercase();
        if path_lower.ends_with(".html") || path_lower.ends_with(".htm") {
            "text/html"
        } else if path_lower.ends_with(".css") {
            "text/css"
        } else if path_lower.ends_with(".js") || path_lower.ends_with(".mjs") {
            "application/javascript"
        } else if path_lower.ends_with(".json") {
            "application/json"
        } else if path_lower.ends_with(".png") {
            "image/png"
        } else if path_lower.ends_with(".jpg") || path_lower.ends_with(".jpeg") {
            "image/jpeg"
        } else if path_lower.ends_with(".gif") {
            "image/gif"
        } else if path_lower.ends_with(".svg") {
            "image/svg+xml"
        } else if path_lower.ends_with(".ico") {
            "image/x-icon"
        } else if path_lower.ends_with(".woff") {
            "font/woff"
        } else if path_lower.ends_with(".woff2") {
            "font/woff2"
        } else if path_lower.ends_with(".ttf") {
            "font/ttf"
        } else if path_lower.ends_with(".otf") {
            "font/otf"
        } else if path_lower.ends_with(".wasm") {
            "application/wasm"
        } else if path_lower.ends_with(".xml") {
            "application/xml"
        } else if path_lower.ends_with(".txt") {
            "text/plain"
        } else {
            "application/octet-stream"
        }
    }
}

/// API response structure for site information.
#[derive(Serialize)]
struct SiteInfoResponse {
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
}

/// API response structure for document source information.
#[derive(Serialize)]
struct SourceResponse {
    #[serde(rename = "type")]
    source_type: String,
    file_path: String,
    line_number: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    byte_range: Option<ByteRangeResponse>,
}

/// Byte range information for API responses.
#[derive(Serialize)]
struct ByteRangeResponse {
    start: usize,
    end: usize,
}

/// API response structure for a document.
#[derive(Serialize)]
struct DocumentResponse {
    id: String,
    title: String,
    content: String,
    source: SourceResponse,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<std::collections::HashMap<String, String>>,
}

/// API response structure for search results.
#[derive(Serialize)]
struct SearchResultResponse {
    document: DocumentResponse,
    score: usize,
    match_type: String,
}

/// API response structure for search endpoint.
#[derive(Serialize)]
struct SearchResponse {
    query: String,
    results: Vec<SearchResultResponse>,
}

/// Information about the documentation site.
///
/// Contains metadata about the site including title, description, and version.
/// This struct is part of the unified ApiService and provides the data for
/// the `/api/site` endpoint.
#[derive(Debug, Clone)]
pub struct SiteInfo {
    /// The site title (e.g., "My Project Documentation")
    pub title: String,
    /// Optional site description
    pub description: Option<String>,
    /// Optional site version
    pub version: Option<String>,
}

impl SiteInfo {
    /// Create site info with just a title.
    ///
    /// # Examples
    /// ```
    /// use hyperlit_engine::api::SiteInfo;
    ///
    /// let info = SiteInfo::new("My Docs");
    /// assert_eq!(info.title, "My Docs");
    /// ```
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: None,
            version: None,
        }
    }

    /// Set the site description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the site version.
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }
}

/// HTTP service providing unified access to all API endpoints and static files.
///
/// This single service handles:
/// - API endpoints:
///   - `GET /api/site` - Returns site information as JSON
///   - `GET /api/documents` - Returns list of all documents as JSON
///   - `GET /api/search?q={query}` - Returns search results as JSON
///   - `GET /api/document/{documentid}` - Returns document as JSON
/// - Static files: Serves UI assets from embedded zip for all other paths
///   - Falls back to index.html for SPA routing
///
/// All API endpoints return HTTP 200 on success and HTTP 599 on any failure.
/// Static files return HTTP 200 with appropriate content types.
#[derive(Clone)]
pub struct ApiService {
    store: StoreHandle,
    site_info: SiteInfo,
    asset_service: Option<EmbeddedAssetService>,
}

impl ApiService {
    /// Create a new ApiService with the given store and site info.
    ///
    /// # Arguments
    /// * `store` - The document store for retrieving documents
    /// * `site_info` - Site metadata for the /api/site endpoint
    ///
    /// # Examples
    /// ```
    /// use hyperlit_engine::{ApiService, SiteInfo, InMemoryStore, StoreHandle};
    ///
    /// let store = StoreHandle::new(InMemoryStore::new());
    /// let site_info = SiteInfo::new("My Documentation");
    /// let service = ApiService::new(store, site_info);
    /// ```
    pub fn new(store: StoreHandle, site_info: SiteInfo) -> Self {
        let asset_service = EmbeddedAssetService::new();
        Self {
            store,
            site_info,
            asset_service,
        }
    }

    /// Serialize data to JSON and wrap in an HTTP 200 response.
    ///
    /// This helper centralizes JSON serialization logic for all API endpoints.
    /// If serialization fails, it returns a HyperlitError that will be converted
    /// to an HTTP 599 error response by the PAL implementation.
    ///
    /// # Type Parameters
    /// * `T` - Any type that implements Serialize
    fn serialize_json_response<T: Serialize>(data: &T) -> HyperlitResult<HttpResponse> {
        serde_json::to_string(data)
            .map(|json| {
                HttpResponse::ok()
                    .with_content_type("application/json")
                    .with_body(json)
            })
            .map_err(|e| {
                Box::new(hyperlit_base::HyperlitError::message(format!(
                    "JSON serialization error: {}",
                    e
                )))
            })
    }

    /// Handle the /api/site endpoint.
    fn handle_site_request(&self) -> HyperlitResult<HttpResponse> {
        let response = SiteInfoResponse {
            title: self.site_info.title.clone(),
            description: self.site_info.description.clone(),
            version: self.site_info.version.clone(),
        };

        Self::serialize_json_response(&response)
    }

    /// Handle the /api/document/{documentid} endpoint.
    fn handle_document_request(&self, path: &str) -> HyperlitResult<HttpResponse> {
        // Extract document ID from path
        let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();

        // Expected format: api/document/{documentid}
        if parts.len() != 3 || parts[0] != "api" || parts[1] != "document" {
            return Err(Box::new(hyperlit_base::HyperlitError::message(
                "Invalid path format. Expected: /api/document/{documentid}",
            )));
        }

        let id_str = parts[2];
        if id_str.is_empty() {
            return Err(Box::new(hyperlit_base::HyperlitError::message(
                "Document ID cannot be empty",
            )));
        }

        let document_id = DocumentId::from_string(id_str);

        // Retrieve document from store
        match self.store.get(&document_id) {
            Ok(Some(doc)) => self.document_to_response(&doc),
            Ok(None) => Err(Box::new(hyperlit_base::HyperlitError::message(
                "Document not found",
            ))),
            Err(e) => Err(Box::new(hyperlit_base::HyperlitError::message(format!(
                "Error retrieving document: {}",
                e
            )))),
        }
    }

    /// Convert a Document to a DocumentResponse.
    fn document_to_response_struct(doc: &Document) -> DocumentResponse {
        let source = doc.source();
        let source_type = if source.is_code_comment() {
            "code_comment".to_string()
        } else {
            "markdown_file".to_string()
        };

        let byte_range = source.byte_range().map(|range| ByteRangeResponse {
            start: range.start(),
            end: range.end(),
        });

        let metadata = doc.metadata().map(|m| {
            let mut map = std::collections::HashMap::new();
            for (key, value) in m.iter() {
                map.insert(key.to_string(), value.to_string());
            }
            map
        });

        DocumentResponse {
            id: doc.id().as_str().to_string(),
            title: doc.title().to_string(),
            content: doc.content().to_string(),
            source: SourceResponse {
                source_type,
                file_path: source.file_path().to_string(),
                line_number: source.line_number(),
                byte_range,
            },
            metadata,
        }
    }

    /// Convert a document to an HTTP response.
    fn document_to_response(&self, doc: &Document) -> HyperlitResult<HttpResponse> {
        let response = Self::document_to_response_struct(doc);
        Self::serialize_json_response(&response)
    }

    /// Handle the /api/documents endpoint.
    fn handle_documents_request(&self) -> HyperlitResult<HttpResponse> {
        match self.store.list() {
            Ok(docs) => {
                let responses: Vec<DocumentResponse> =
                    docs.iter().map(Self::document_to_response_struct).collect();

                Self::serialize_json_response(&responses)
            }
            Err(e) => Err(Box::new(hyperlit_base::HyperlitError::message(format!(
                "Error retrieving documents: {}",
                e
            )))),
        }
    }

    /// Handle the /api/search endpoint.
    fn handle_search_request(&self, request: &HttpRequest) -> HyperlitResult<HttpResponse> {
        // Parse query parameter from URL
        let query = request
            .path()
            .split('?')
            .nth(1)
            .and_then(|params| {
                params.split('&').find_map(|param| {
                    let (key, value) = param.split_once('=')?;
                    if key == "q" {
                        Some(value.to_string())
                    } else {
                        None
                    }
                })
            })
            .unwrap_or_default();

        if query.is_empty() {
            return Err(Box::new(hyperlit_base::HyperlitError::message(
                "Missing required query parameter 'q'",
            )));
        }

        match self.store.list() {
            Ok(docs) => {
                let search = SimpleSearch::new();
                let results = search.search(docs.iter(), &query);

                let response_results: Vec<SearchResultResponse> = results
                    .into_iter()
                    .map(|result| SearchResultResponse {
                        document: Self::document_to_response_struct(&result.document),
                        score: result.score,
                        match_type: match result.match_type {
                            MatchType::Title => "title".to_string(),
                            MatchType::Content => "content".to_string(),
                            MatchType::Both => "both".to_string(),
                        },
                    })
                    .collect();

                let response = SearchResponse {
                    query,
                    results: response_results,
                };

                Self::serialize_json_response(&response)
            }
            Err(e) => Err(Box::new(hyperlit_base::HyperlitError::message(format!(
                "Error searching documents: {}",
                e
            )))),
        }
    }
}

impl std::fmt::Debug for ApiService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApiService")
            .field("site_info", &self.site_info)
            .finish()
    }
}

impl HttpService for ApiService {
    fn handle_request(&self, request: HttpRequest) -> HyperlitResult<HttpResponse> {
        // Only handle GET requests
        if request.method() != &HttpMethod::Get {
            return Err(Box::new(hyperlit_base::HyperlitError::message(
                "Only GET requests are supported",
            )));
        }

        // Remove query parameters from path
        let path = request.path().split('?').next().unwrap_or(request.path());

        // Route to appropriate handler based on path
        if path == "/api/site" {
            self.handle_site_request()
        } else if path == "/api/documents" {
            self.handle_documents_request()
        } else if path.starts_with("/api/search") {
            self.handle_search_request(&request)
        } else if path.starts_with("/api/document/") {
            self.handle_document_request(path)
        } else {
            // 📖 # Why serve static files from embedded zip?
            // For non-API routes, serve UI assets from the embedded zip.
            // This enables the single-binary distribution where the UI is
            // appended to the Rust binary as a zip file.
            // If no embedded assets are available, fall back to error.
            // If file not found, serve index.html for SPA routing.
            debug!(path = path, "Handling static file request");

            if let Some(ref asset_service) = self.asset_service {
                // Try to serve the requested file
                if let Some(response) = asset_service.serve_file(path) {
                    debug!(path = path, "Served requested file from embedded assets");
                    return Ok(response);
                }

                // File not found - serve index.html for SPA routing
                // This allows React Router to handle the route
                debug!(
                    path = path,
                    "Requested file not found, falling back to index.html for SPA routing"
                );
                if let Some(response) = asset_service.serve_file("/index.html") {
                    debug!("Served index.html for SPA routing");
                    return Ok(response);
                }

                error!("Failed to serve index.html - not found in embedded assets");
            } else {
                warn!(
                    path = path,
                    "No embedded asset service available - binary may not contain embedded UI assets"
                );
            }

            Err(Box::new(hyperlit_base::HyperlitError::message(
                "Invalid API endpoint or no embedded UI assets available",
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{DocumentSource, SourceType};
    use crate::store::InMemoryStore;
    use hyperlit_base::FilePath;

    fn create_test_service() -> ApiService {
        let store = StoreHandle::new(InMemoryStore::new());
        let site_info = SiteInfo::new("Test Site")
            .with_description("A test site")
            .with_version("1.0.0");
        ApiService::new(store, site_info)
    }

    fn create_test_document(title: &str, content: &str) -> Document {
        let source =
            DocumentSource::new(SourceType::CodeComment, FilePath::from("src/test.rs"), 42);
        Document::new(
            title.to_string(),
            content.to_string(),
            source,
            None,
            &std::collections::HashSet::new(),
        )
    }

    #[test]
    fn test_handle_site_request_success() {
        let service = create_test_service();
        let request = HttpRequest::new(HttpMethod::Get, "/api/site");
        let response = service.handle_request(request).unwrap();

        assert_eq!(response.status().as_u16(), 200);
        assert_eq!(
            response.headers().get("Content-Type"),
            Some(&"application/json".to_string())
        );

        let body = response.body().as_string().unwrap();
        assert!(body.contains("Test Site"));
        assert!(body.contains("A test site"));
        assert!(body.contains("1.0.0"));
    }

    #[test]
    fn test_handle_document_request_success() {
        let store = StoreHandle::new(InMemoryStore::new());
        let site_info = SiteInfo::new("Test Site");
        let service = ApiService::new(store.clone(), site_info);

        // Insert a document
        let doc = create_test_document("My Document", "Document content");
        let id = doc.id().clone();
        store.insert(doc).unwrap();

        // Request the document
        let request = HttpRequest::new(HttpMethod::Get, format!("/api/document/{}", id.as_str()));
        let response = service.handle_request(request).unwrap();

        assert_eq!(response.status().as_u16(), 200);
        let body = response.body().as_string().unwrap();
        assert!(body.contains("My Document"));
        assert!(body.contains("Document content"));
    }

    #[test]
    fn test_handle_document_request_not_found() {
        let service = create_test_service();
        let request = HttpRequest::new(HttpMethod::Get, "/api/document/non-existent");
        let result = service.handle_request(request);

        // Service returns Err for not found, which RealPal converts to HTTP 599
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Document not found"));
    }

    #[test]
    fn test_handle_invalid_endpoint() {
        let service = create_test_service();
        let request = HttpRequest::new(HttpMethod::Get, "/api/other");
        let result = service.handle_request(request);

        // Service returns Err for invalid endpoint
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Invalid API endpoint"));
    }

    #[test]
    fn test_handle_wrong_method() {
        let service = create_test_service();
        let request = HttpRequest::new(HttpMethod::Post, "/api/site");
        let result = service.handle_request(request);

        // Service returns Err for wrong method
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Only GET requests are supported"));
    }

    #[test]
    fn test_document_with_byte_range() {
        let store = StoreHandle::new(InMemoryStore::new());
        let site_info = SiteInfo::new("Test Site");
        let service = ApiService::new(store.clone(), site_info);

        // Create document with byte range
        let source =
            DocumentSource::new(SourceType::CodeComment, FilePath::from("src/test.rs"), 42)
                .with_byte_range(crate::document::ByteRange::new(100, 250));

        let doc = Document::new(
            "Test Doc".to_string(),
            "Content".to_string(),
            source,
            None,
            &std::collections::HashSet::new(),
        );
        let id = doc.id().clone();
        store.insert(doc).unwrap();

        let request = HttpRequest::new(HttpMethod::Get, format!("/api/document/{}", id.as_str()));
        let response = service.handle_request(request).unwrap();

        assert_eq!(response.status().as_u16(), 200);
        let body = response.body().as_string().unwrap();
        assert!(body.contains("\"byte_range\""));
        assert!(body.contains("\"start\":100"));
        assert!(body.contains("\"end\":250"));
    }

    #[test]
    fn test_site_endpoint_with_query_params() {
        let service = create_test_service();
        let request = HttpRequest::new(HttpMethod::Get, "/api/site?format=json");
        let response = service.handle_request(request).unwrap();

        assert_eq!(response.status().as_u16(), 200);
        let body = response.body().as_string().unwrap();
        assert!(body.contains("Test Site"));
    }
}
