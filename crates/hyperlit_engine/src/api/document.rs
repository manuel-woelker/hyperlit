/* 📖 # Why a dedicated document API service?

The DocumentService provides an HTTP endpoint for retrieving documents by ID.
This enables:

1. **RESTful access**: Clients can fetch documents via GET /api/document/{documentid}
2. **JSON serialization**: Documents are returned as JSON for easy consumption
3. **Integration with storage**: Uses StoreHandle for thread-safe document retrieval
4. **Error handling**: Returns 599 for any failure to simplify error handling

This service implements the HttpService trait from hyperlit_base, making it compatible
with both RealPal and MockPal implementations.
*/

use hyperlit_base::pal::http::{
    HttpMethod, HttpRequest, HttpResponse, HttpService, HttpStatusCode,
};

use crate::document::{Document, DocumentId};
use crate::store::StoreHandle;

/// HTTP service for serving documents at /api/document/{documentid}.
///
/// This service handles GET requests to `/api/document/{documentid}` and returns
/// the document as JSON. If the document is not found or any error occurs,
/// returns HTTP 599.
#[derive(Clone)]
pub struct DocumentService {
    store: StoreHandle,
}

impl std::fmt::Debug for DocumentService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DocumentService").finish()
    }
}

impl DocumentService {
    /// Create a new DocumentService with the given store handle.
    ///
    /// # Arguments
    /// * `store` - The document store handle for retrieving documents
    ///
    /// # Examples
    /// ```
    /// use hyperlit_engine::{DocumentService, InMemoryStore, StoreHandle};
    ///
    /// let store = StoreHandle::new(InMemoryStore::new());
    /// let service = DocumentService::new(store);
    /// ```
    pub fn new(store: StoreHandle) -> Self {
        Self { store }
    }

    /// Parse the document ID from the request path.
    ///
    /// Expects paths in the format `/api/document/{documentid}`.
    /// Returns None if the path doesn't match the expected format.
    fn extract_document_id(&self, path: &str) -> Option<DocumentId> {
        // Remove any query parameters
        let path = path.split('?').next().unwrap_or(path);

        // Must start with leading slash
        if !path.starts_with('/') {
            return None;
        }

        // Parse the path
        let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();

        // Expected format: api/document/{documentid}
        if parts.len() == 3 && parts[0] == "api" && parts[1] == "document" {
            let id_str = parts[2];
            if !id_str.is_empty() {
                return Some(DocumentId::from_string(id_str));
            }
        }

        None
    }

    /// Serialize a document to JSON.
    ///
    /// Returns a JSON object with id, title, content, and source information.
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

    /// Handle successful document retrieval.
    fn success_response(&self, doc: &Document) -> HttpResponse {
        let json = self.document_to_json(doc);
        HttpResponse::ok()
            .with_content_type("application/json")
            .with_body(json)
    }

    /// Handle any failure with HTTP 599.
    fn failure_response(&self, message: &str) -> HttpResponse {
        let error_json = format!(r#"{{"error":"{}"}}"#, Self::escape_json(message));
        HttpResponse::new(HttpStatusCode::from(599))
            .with_content_type("application/json")
            .with_body(error_json)
    }
}

impl HttpService for DocumentService {
    fn handle_request(&self, request: HttpRequest) -> HttpResponse {
        // Only handle GET requests
        if request.method() != &HttpMethod::Get {
            return self.failure_response("Only GET requests are supported");
        }

        // Extract document ID from path
        let document_id = match self.extract_document_id(request.path()) {
            Some(id) => id,
            None => {
                return self
                    .failure_response("Invalid path format. Expected: /api/document/{documentid}");
            }
        };

        // Retrieve document from store
        let doc_result = self.store.get(&document_id);

        match doc_result {
            Ok(Some(doc)) => self.success_response(&doc),
            Ok(None) => self.failure_response("Document not found"),
            Err(e) => self.failure_response(&format!("Error retrieving document: {}", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{DocumentSource, SourceType};
    use crate::store::InMemoryStore;
    use hyperlit_base::FilePath;

    fn create_test_document(title: &str, content: &str) -> Document {
        let source =
            DocumentSource::new(SourceType::CodeComment, FilePath::from("src/test.rs"), 42);
        // We can't easily create a Document with a specific ID since Document::new generates IDs
        // So we'll use the store to insert and then retrieve
        Document::new(
            title.to_string(),
            content.to_string(),
            source,
            None,
            &std::collections::HashSet::new(),
        )
    }

    #[test]
    fn test_extract_document_id_valid() {
        let store = StoreHandle::new(InMemoryStore::new());
        let service = DocumentService::new(store);

        assert_eq!(
            service.extract_document_id("/api/document/my-doc"),
            Some(DocumentId::from_string("my-doc"))
        );
        assert_eq!(
            service.extract_document_id("/api/document/why-use-arc"),
            Some(DocumentId::from_string("why-use-arc"))
        );
    }

    #[test]
    fn test_extract_document_id_invalid() {
        let store = StoreHandle::new(InMemoryStore::new());
        let service = DocumentService::new(store);

        assert!(service.extract_document_id("/api/document/").is_none());
        assert!(service.extract_document_id("/api/other/my-doc").is_none());
        assert!(service.extract_document_id("/wrong/path").is_none());
        assert!(service.extract_document_id("api/document/my-doc").is_none());
    }

    #[test]
    fn test_document_to_json() {
        let store = StoreHandle::new(InMemoryStore::new());
        let service = DocumentService::new(store);

        let doc = create_test_document("Test Title", "Test content");
        let json = service.document_to_json(&doc);

        // Verify JSON contains expected fields
        assert!(json.contains("\"id\":\""));
        assert!(json.contains("\"title\":\"Test Title\""));
        assert!(json.contains("\"content\":\"Test content\""));
        assert!(json.contains("\"source\":"));
        assert!(json.contains("\"type\":\"code_comment\""));
        assert!(json.contains("\"file_path\":\"src/test.rs\""));
        assert!(json.contains("\"line_number\":42"));
    }

    #[test]
    fn test_document_to_json_escaping() {
        let store = StoreHandle::new(InMemoryStore::new());
        let service = DocumentService::new(store);

        let doc = create_test_document("Title with \"quotes\"", "Line1\nLine2\\");
        let json = service.document_to_json(&doc);

        // Verify proper escaping
        assert!(json.contains("\\\"quotes\\\""));
        assert!(json.contains("\\n"));
        assert!(json.contains("\\\\"));
    }

    #[test]
    fn test_handle_request_success() {
        let store = StoreHandle::new(InMemoryStore::new());
        let service = DocumentService::new(store.clone());

        // Insert a document
        let doc = create_test_document("My Document", "Document content");
        let id = doc.id().clone();
        store.insert(doc).unwrap();

        // Create request
        let request = HttpRequest::new(HttpMethod::Get, format!("/api/document/{}", id.as_str()));
        let response = service.handle_request(request);

        // Verify response
        assert_eq!(response.status().as_u16(), 200);
        assert_eq!(
            response.headers().get("Content-Type"),
            Some(&"application/json".to_string())
        );

        let body = response.body().as_string().unwrap();
        assert!(body.contains("My Document"));
        assert!(body.contains("Document content"));
    }

    #[test]
    fn test_handle_request_not_found() {
        let store = StoreHandle::new(InMemoryStore::new());
        let service = DocumentService::new(store);

        // Request non-existent document
        let request = HttpRequest::new(HttpMethod::Get, "/api/document/non-existent-doc");
        let response = service.handle_request(request);

        // Should return 599
        assert_eq!(response.status().as_u16(), 599);
        let body = response.body().as_string().unwrap();
        assert!(body.contains("error"));
    }

    #[test]
    fn test_handle_request_invalid_path() {
        let store = StoreHandle::new(InMemoryStore::new());
        let service = DocumentService::new(store);

        let request = HttpRequest::new(HttpMethod::Get, "/invalid/path");
        let response = service.handle_request(request);

        assert_eq!(response.status().as_u16(), 599);
        let body = response.body().as_string().unwrap();
        assert!(body.contains("Invalid path format"));
    }

    #[test]
    fn test_handle_request_wrong_method() {
        let store = StoreHandle::new(InMemoryStore::new());
        let service = DocumentService::new(store);

        let request = HttpRequest::new(HttpMethod::Post, "/api/document/test");
        let response = service.handle_request(request);

        assert_eq!(response.status().as_u16(), 599);
        let body = response.body().as_string().unwrap();
        assert!(body.contains("Only GET requests are supported"));
    }

    #[test]
    fn test_escape_json() {
        assert_eq!(DocumentService::escape_json("test"), "test");
        assert_eq!(
            DocumentService::escape_json("with\"quotes\""),
            "with\\\"quotes\\\""
        );
        assert_eq!(
            DocumentService::escape_json("path\\to\\file"),
            "path\\\\to\\\\file"
        );
        assert_eq!(
            DocumentService::escape_json("line1\nline2"),
            "line1\\nline2"
        );
        assert_eq!(DocumentService::escape_json("tab\there"), "tab\\there");
    }
}
