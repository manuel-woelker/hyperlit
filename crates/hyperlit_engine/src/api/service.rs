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

use hyperlit_base::pal::http::{
    HttpMethod, HttpRequest, HttpResponse, HttpService, HttpStatusCode,
};

use crate::document::{Document, DocumentId};
use crate::search::SimpleSearch;
use crate::store::StoreHandle;

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

    /// Handle the /api/site endpoint.
    fn handle_site_request(&self) -> HttpResponse {
        let mut json = format!(
            r#"{{"title":"{}""#,
            Self::escape_json(&self.site_info.title)
        );

        if let Some(ref description) = self.site_info.description {
            json.push_str(&format!(
                r#","description":"{}""#,
                Self::escape_json(description)
            ));
        }

        if let Some(ref version) = self.site_info.version {
            json.push_str(&format!(r#","version":"{}""#, Self::escape_json(version)));
        }

        json.push('}');

        HttpResponse::ok()
            .with_content_type("application/json")
            .with_body(json)
    }

    /// Handle the /api/document/{documentid} endpoint.
    fn handle_document_request(&self, path: &str) -> HttpResponse {
        // Extract document ID from path
        let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();

        // Expected format: api/document/{documentid}
        if parts.len() != 3 || parts[0] != "api" || parts[1] != "document" {
            return self
                .failure_response("Invalid path format. Expected: /api/document/{documentid}");
        }

        let id_str = parts[2];
        if id_str.is_empty() {
            return self.failure_response("Document ID cannot be empty");
        }

        let document_id = DocumentId::from_string(id_str);

        // Retrieve document from store
        match self.store.get(&document_id) {
            Ok(Some(doc)) => self.document_to_response(&doc),
            Ok(None) => self.failure_response("Document not found"),
            Err(e) => self.failure_response(&format!("Error retrieving document: {}", e)),
        }
    }

    /// Convert a document to an HTTP response.
    fn document_to_response(&self, doc: &Document) -> HttpResponse {
        let source = doc.source();
        let mut json = format!(
            r#"{{"id":"{}","title":"{}","content":"{}","source":{{"type":"{}","file_path":"{}","line_number":{}}}"#,
            Self::escape_json(doc.id().as_str()),
            Self::escape_json(doc.title()),
            Self::escape_json(doc.content()),
            if source.is_code_comment() {
                "code_comment"
            } else {
                "markdown_file"
            },
            Self::escape_json(&source.file_path().to_string()),
            source.line_number()
        );

        // Add byte_range if present
        if let Some(range) = source.byte_range() {
            let byte_range = format!(
                r#","byte_range":{{"start":{},"end":{}}}"#,
                range.start(),
                range.end()
            );
            json.push_str(&byte_range);
        }

        // Add metadata if present
        if let Some(metadata) = doc.metadata() {
            json.push_str(r#","metadata":{"#);
            let mut first = true;
            for (key, value) in metadata.iter() {
                if !first {
                    json.push(',');
                }
                first = false;
                json.push_str(&format!(
                    "\"{}\":\"{}\"",
                    Self::escape_json(key),
                    Self::escape_json(value)
                ));
            }
            json.push('}');
        }

        json.push('}');

        HttpResponse::ok()
            .with_content_type("application/json")
            .with_body(json)
    }

    /// Handle the /api/documents endpoint.
    fn handle_documents_request(&self) -> HttpResponse {
        match self.store.list() {
            Ok(docs) => {
                let mut json = String::from("[");
                let mut first = true;

                for doc in docs {
                    if !first {
                        json.push(',');
                    }
                    first = false;
                    json.push_str(&self.document_to_json(&doc));
                }

                json.push(']');

                HttpResponse::ok()
                    .with_content_type("application/json")
                    .with_body(json)
            }
            Err(e) => self.failure_response(&format!("Error retrieving documents: {}", e)),
        }
    }

    /// Handle the /api/search endpoint.
    fn handle_search_request(&self, request: &HttpRequest) -> HttpResponse {
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
            return self.failure_response("Missing required query parameter 'q'");
        }

        match self.store.list() {
            Ok(docs) => {
                let search = SimpleSearch::new();
                let results = search.search(docs.iter(), &query);

                let mut json = String::from("{");
                json.push_str(&format!(
                    r#""query":"{}","results":"#,
                    Self::escape_json(&query)
                ));
                json.push('[');

                let mut first = true;
                for result in results {
                    if !first {
                        json.push(',');
                    }
                    first = false;

                    let match_type_str = match result.match_type {
                        crate::search::MatchType::Title => "title",
                        crate::search::MatchType::Content => "content",
                        crate::search::MatchType::Both => "both",
                    };

                    json.push_str(&format!(
                        r#"{{"document":{},"score":{},"match_type":"{}"}}"#,
                        self.document_to_json(&result.document),
                        result.score,
                        match_type_str
                    ));
                }

                json.push_str("]}}");

                HttpResponse::ok()
                    .with_content_type("application/json")
                    .with_body(json)
            }
            Err(e) => self.failure_response(&format!("Error searching documents: {}", e)),
        }
    }

    /// Convert a document to JSON string.
    fn document_to_json(&self, doc: &Document) -> String {
        let source = doc.source();
        let mut json = format!(
            r#"{{"id":"{}","title":"{}","content":"{}","source":{{"type":"{}","file_path":"{}","line_number":{}}}"#,
            Self::escape_json(doc.id().as_str()),
            Self::escape_json(doc.title()),
            Self::escape_json(doc.content()),
            if source.is_code_comment() {
                "code_comment"
            } else {
                "markdown_file"
            },
            Self::escape_json(&source.file_path().to_string()),
            source.line_number()
        );

        // Add byte_range if present
        if let Some(range) = source.byte_range() {
            let byte_range = format!(
                r#","byte_range":{{"start":{},"end":{}}}"#,
                range.start(),
                range.end()
            );
            json.push_str(&byte_range);
        }

        // Add metadata if present
        if let Some(metadata) = doc.metadata() {
            json.push_str(r#","metadata":{"#);
            let mut first = true;
            for (key, value) in metadata.iter() {
                if !first {
                    json.push(',');
                }
                first = false;
                json.push_str(&format!(
                    "\"{}\":\"{}\"",
                    Self::escape_json(key),
                    Self::escape_json(value)
                ));
            }
            json.push('}');
        }

        json.push('}');
        json
    }

    /// Handle any failure with HTTP 599.
    fn failure_response(&self, message: &str) -> HttpResponse {
        let error_json = format!(r#"{{"error":"{}"}}"#, Self::escape_json(message));
        HttpResponse::new(HttpStatusCode::NetworkConnectTimeoutError)
            .with_content_type("application/json")
            .with_body(error_json)
    }

    /// Escape special characters for JSON strings.
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

impl std::fmt::Debug for ApiService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApiService")
            .field("site_info", &self.site_info)
            .finish()
    }
}

impl HttpService for ApiService {
    fn handle_request(&self, request: HttpRequest) -> HttpResponse {
        // Only handle GET requests
        if request.method() != &HttpMethod::Get {
            return self.failure_response("Only GET requests are supported");
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
            self.failure_response("Invalid API endpoint")
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
        let response = service.handle_request(request);

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
        let response = service.handle_request(request);

        assert_eq!(response.status().as_u16(), 200);
        let body = response.body().as_string().unwrap();
        assert!(body.contains("My Document"));
        assert!(body.contains("Document content"));
    }

    #[test]
    fn test_handle_document_request_not_found() {
        let service = create_test_service();
        let request = HttpRequest::new(HttpMethod::Get, "/api/document/non-existent");
        let response = service.handle_request(request);

        assert_eq!(response.status().as_u16(), 599);
        let body = response.body().as_string().unwrap();
        assert!(body.contains("Document not found"));
    }

    #[test]
    fn test_handle_invalid_endpoint() {
        let service = create_test_service();
        let request = HttpRequest::new(HttpMethod::Get, "/api/other");
        let response = service.handle_request(request);

        assert_eq!(response.status().as_u16(), 599);
        let body = response.body().as_string().unwrap();
        assert!(body.contains("Invalid API endpoint"));
    }

    #[test]
    fn test_handle_wrong_method() {
        let service = create_test_service();
        let request = HttpRequest::new(HttpMethod::Post, "/api/site");
        let response = service.handle_request(request);

        assert_eq!(response.status().as_u16(), 599);
        let body = response.body().as_string().unwrap();
        assert!(body.contains("Only GET requests are supported"));
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
        let response = service.handle_request(request);

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
        let response = service.handle_request(request);

        assert_eq!(response.status().as_u16(), 200);
        let body = response.body().as_string().unwrap();
        assert!(body.contains("Test Site"));
    }
}
