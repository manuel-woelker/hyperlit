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
use hyperlit_base::pal::http::{HttpMethod, HttpRequest, HttpResponse, HttpService};
use serde::Serialize;

use crate::document::{Document, DocumentId};
use crate::search::{MatchType, SimpleSearch};
use crate::store::StoreHandle;

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

/// HTTP service providing unified access to all API endpoints.
///
/// This single service handles all API endpoints:
/// - `GET /api/site` - Returns site information as JSON
/// - `GET /api/documents` - Returns list of all documents as JSON
/// - `GET /api/search?q={query}` - Returns search results as JSON
/// - `GET /api/document/{documentid}` - Returns document as JSON
///
/// All endpoints return HTTP 200 on success and HTTP 599 on any failure.
#[derive(Clone)]
pub struct ApiService {
    store: StoreHandle,
    site_info: SiteInfo,
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
        Self { store, site_info }
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
            Err(Box::new(hyperlit_base::HyperlitError::message(
                "Invalid API endpoint",
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
