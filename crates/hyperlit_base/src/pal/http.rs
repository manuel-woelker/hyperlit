/* 📖 # Why a dedicated HTTP module in the PAL?

The HTTP abstraction allows the application to serve HTTP requests while remaining
fully testable with MockPal. This enables:

- **Testable web services**: MockPal can capture requests in-memory for assertions
- **Consistent interface**: Single API for both real and test scenarios
- **Synchronous simplicity**: No async complexity, matching the project's philosophy

This module provides raw HTTP types and abstractions for building REST APIs.
*/

use std::collections::HashMap;
use std::sync::Arc;

/// HTTP methods supported by the service.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
    Trace,
    Connect,
}

impl HttpMethod {
    /// Parse an HTTP method from a string.
    pub fn parse(method: &str) -> Option<Self> {
        match method.to_uppercase().as_str() {
            "GET" => Some(Self::Get),
            "POST" => Some(Self::Post),
            "PUT" => Some(Self::Put),
            "DELETE" => Some(Self::Delete),
            "PATCH" => Some(Self::Patch),
            "HEAD" => Some(Self::Head),
            "OPTIONS" => Some(Self::Options),
            "TRACE" => Some(Self::Trace),
            "CONNECT" => Some(Self::Connect),
            _ => None,
        }
    }

    /// Convert the method to its string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
            Self::Patch => "PATCH",
            Self::Head => "HEAD",
            Self::Options => "OPTIONS",
            Self::Trace => "TRACE",
            Self::Connect => "CONNECT",
        }
    }
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// HTTP headers collection.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HttpHeaders {
    inner: HashMap<String, String>,
}

impl HttpHeaders {
    /// Create empty headers.
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Insert a header.
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.inner.insert(key.into(), value.into());
    }

    /// Get a header value.
    pub fn get(&self, key: &str) -> Option<&String> {
        self.inner.get(key)
    }

    /// Check if a header exists.
    pub fn contains(&self, key: &str) -> bool {
        self.inner.contains_key(key)
    }

    /// Remove a header.
    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.inner.remove(key)
    }

    /// Get all headers as a reference.
    pub fn all(&self) -> &HashMap<String, String> {
        &self.inner
    }

    /// Get all headers as an owned HashMap.
    pub fn into_inner(self) -> HashMap<String, String> {
        self.inner
    }
}

impl From<HashMap<String, String>> for HttpHeaders {
    fn from(map: HashMap<String, String>) -> Self {
        Self { inner: map }
    }
}

/* 📖 # Why support both bytes and streaming in HttpBody?
SSE (Server-Sent Events) requires streaming responses where the body is generated
over time, not all at once. Most API responses are fixed-size bytes, but SSE needs
to continuously send data as events occur. Supporting both modes allows regular
endpoints to use simple byte buffers while SSE can use a streaming reader.
*/

/// HTTP request body content.
pub enum HttpBody {
    /// Fixed-size body content
    Bytes(Vec<u8>),
    /// Streaming body content
    Stream(Box<dyn std::io::Read + Send>),
}

impl HttpBody {
    /// Create an empty body.
    pub fn empty() -> Self {
        Self::Bytes(vec![])
    }

    /// Create from bytes.
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self::Bytes(bytes)
    }

    /// Create from string.
    pub fn from_string(s: impl Into<String>) -> Self {
        Self::Bytes(s.into().into_bytes())
    }

    /// Create from a streaming reader.
    pub fn from_reader<R: std::io::Read + Send + 'static>(reader: R) -> Self {
        Self::Stream(Box::new(reader))
    }

    /// Get content as bytes (only works for Bytes variant).
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Bytes(bytes) => bytes,
            Self::Stream(_) => &[],
        }
    }

    /// Get content as a string if valid UTF-8 (only works for Bytes variant).
    pub fn as_string(&self) -> Option<String> {
        match self {
            Self::Bytes(bytes) => String::from_utf8(bytes.clone()).ok(),
            Self::Stream(_) => None,
        }
    }

    /// Check if body is empty.
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Bytes(bytes) => bytes.is_empty(),
            Self::Stream(_) => false,
        }
    }

    /// Get the content length (only works for Bytes variant).
    pub fn len(&self) -> usize {
        match self {
            Self::Bytes(bytes) => bytes.len(),
            Self::Stream(_) => 0,
        }
    }

    /// Take ownership of the content (only works for Bytes variant).
    pub fn into_bytes(self) -> Vec<u8> {
        match self {
            Self::Bytes(bytes) => bytes,
            Self::Stream(_) => vec![],
        }
    }

    /// Convert into a reader suitable for tiny_http.
    pub fn into_reader(self) -> Box<dyn std::io::Read + Send> {
        match self {
            Self::Bytes(bytes) => Box::new(std::io::Cursor::new(bytes)),
            Self::Stream(reader) => reader,
        }
    }
}

impl Default for HttpBody {
    fn default() -> Self {
        Self::empty()
    }
}

impl Clone for HttpBody {
    fn clone(&self) -> Self {
        match self {
            Self::Bytes(bytes) => Self::Bytes(bytes.clone()),
            Self::Stream(_) => {
                // Streaming bodies cannot be cloned
                panic!("Cannot clone streaming HttpBody")
            }
        }
    }
}

impl std::fmt::Debug for HttpBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bytes(bytes) => f.debug_tuple("Bytes").field(&bytes.len()).finish(),
            Self::Stream(_) => f.debug_tuple("Stream").finish(),
        }
    }
}

impl PartialEq for HttpBody {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bytes(a), Self::Bytes(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for HttpBody {}

impl From<Vec<u8>> for HttpBody {
    fn from(v: Vec<u8>) -> Self {
        Self::from_bytes(v)
    }
}

impl From<String> for HttpBody {
    fn from(s: String) -> Self {
        Self::from_string(s)
    }
}

impl From<&str> for HttpBody {
    fn from(s: &str) -> Self {
        Self::from_string(s)
    }
}

/// HTTP request structure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequest {
    method: HttpMethod,
    path: String,
    headers: HttpHeaders,
    body: HttpBody,
}

impl HttpRequest {
    /// Create a new HTTP request.
    pub fn new(method: HttpMethod, path: impl Into<String>) -> Self {
        Self {
            method,
            path: path.into(),
            headers: HttpHeaders::new(),
            body: HttpBody::empty(),
        }
    }

    /// Get the HTTP method.
    pub fn method(&self) -> &HttpMethod {
        &self.method
    }

    /// Get the request path.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Get the request headers.
    pub fn headers(&self) -> &HttpHeaders {
        &self.headers
    }

    /// Get mutable access to headers.
    pub fn headers_mut(&mut self) -> &mut HttpHeaders {
        &mut self.headers
    }

    /// Get the request body.
    pub fn body(&self) -> &HttpBody {
        &self.body
    }

    /// Set the request body.
    pub fn with_body(mut self, body: impl Into<HttpBody>) -> Self {
        self.body = body.into();
        self
    }

    /// Set a header.
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key, value);
        self
    }
}

/// HTTP status codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpStatusCode {
    // 2xx Success
    Ok = 200,
    Created = 201,
    Accepted = 202,
    NoContent = 204,

    // 3xx Redirection
    MovedPermanently = 301,
    Found = 302,
    NotModified = 304,

    // 4xx Client Errors
    BadRequest = 400,
    Unauthorized = 401,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    Conflict = 409,
    UnprocessableEntity = 422,
    TooManyRequests = 429,

    // 5xx Server Errors
    InternalServerError = 500,
    NotImplemented = 501,
    BadGateway = 502,
    ServiceUnavailable = 503,
    GatewayTimeout = 504,
    NetworkConnectTimeoutError = 599,
}

impl HttpStatusCode {
    /// Get the numeric status code.
    pub fn as_u16(&self) -> u16 {
        *self as u16
    }

    /// Get the standard reason phrase.
    pub fn reason_phrase(&self) -> &'static str {
        match self {
            Self::Ok => "OK",
            Self::Created => "Created",
            Self::Accepted => "Accepted",
            Self::NoContent => "No Content",
            Self::MovedPermanently => "Moved Permanently",
            Self::Found => "Found",
            Self::NotModified => "Not Modified",
            Self::BadRequest => "Bad Request",
            Self::Unauthorized => "Unauthorized",
            Self::Forbidden => "Forbidden",
            Self::NotFound => "Not Found",
            Self::MethodNotAllowed => "Method Not Allowed",
            Self::Conflict => "Conflict",
            Self::UnprocessableEntity => "Unprocessable Entity",
            Self::TooManyRequests => "Too Many Requests",
            Self::InternalServerError => "Internal Server Error",
            Self::NotImplemented => "Not Implemented",
            Self::BadGateway => "Bad Gateway",
            Self::ServiceUnavailable => "Service Unavailable",
            Self::GatewayTimeout => "Gateway Timeout",
            Self::NetworkConnectTimeoutError => "Network Connect Timeout Error",
        }
    }
}

impl From<u16> for HttpStatusCode {
    fn from(code: u16) -> Self {
        match code {
            200 => Self::Ok,
            201 => Self::Created,
            202 => Self::Accepted,
            204 => Self::NoContent,
            301 => Self::MovedPermanently,
            302 => Self::Found,
            304 => Self::NotModified,
            400 => Self::BadRequest,
            401 => Self::Unauthorized,
            403 => Self::Forbidden,
            404 => Self::NotFound,
            405 => Self::MethodNotAllowed,
            409 => Self::Conflict,
            422 => Self::UnprocessableEntity,
            429 => Self::TooManyRequests,
            500 => Self::InternalServerError,
            501 => Self::NotImplemented,
            502 => Self::BadGateway,
            503 => Self::ServiceUnavailable,
            504 => Self::GatewayTimeout,
            599 => Self::NetworkConnectTimeoutError,
            _ => Self::InternalServerError, // Default for unknown codes
        }
    }
}

/// HTTP response structure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    status: HttpStatusCode,
    headers: HttpHeaders,
    body: HttpBody,
}

impl HttpResponse {
    /// Create a new response with the given status.
    pub fn new(status: HttpStatusCode) -> Self {
        Self {
            status,
            headers: HttpHeaders::new(),
            body: HttpBody::empty(),
        }
    }

    /// Create a 200 OK response.
    pub fn ok() -> Self {
        Self::new(HttpStatusCode::Ok)
    }

    /// Create a 201 Created response.
    pub fn created() -> Self {
        Self::new(HttpStatusCode::Created)
    }

    /// Create a 204 No Content response.
    pub fn no_content() -> Self {
        Self::new(HttpStatusCode::NoContent)
    }

    /// Create a 400 Bad Request response.
    pub fn bad_request() -> Self {
        Self::new(HttpStatusCode::BadRequest)
    }

    /// Create a 404 Not Found response.
    pub fn not_found() -> Self {
        Self::new(HttpStatusCode::NotFound)
    }

    /// Create a 500 Internal Server Error response.
    pub fn internal_error() -> Self {
        Self::new(HttpStatusCode::InternalServerError)
    }

    /// Get the status code.
    pub fn status(&self) -> HttpStatusCode {
        self.status
    }

    /// Get the headers.
    pub fn headers(&self) -> &HttpHeaders {
        &self.headers
    }

    /// Get mutable access to headers.
    pub fn headers_mut(&mut self) -> &mut HttpHeaders {
        &mut self.headers
    }

    /// Get the body.
    pub fn body(&self) -> &HttpBody {
        &self.body
    }

    /// Take ownership of the body.
    pub fn into_body(self) -> HttpBody {
        self.body
    }

    /// Set the response body.
    pub fn with_body(mut self, body: impl Into<HttpBody>) -> Self {
        self.body = body.into();
        self
    }

    /// Set a header.
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key, value);
        self
    }

    /// Set the Content-Type header.
    pub fn with_content_type(self, content_type: impl Into<String>) -> Self {
        self.with_header("Content-Type", content_type)
    }

    /// Set the status code.
    pub fn with_status(mut self, status: HttpStatusCode) -> Self {
        self.status = status;
        self
    }

    /// Create a JSON response.
    pub fn json(body: impl Into<String>) -> Self {
        Self::ok()
            .with_content_type("application/json")
            .with_body(body.into())
    }

    /// Create a plain text response.
    pub fn text(body: impl Into<String>) -> Self {
        let body_str: String = body.into();
        Self::ok()
            .with_content_type("text/plain")
            .with_body(body_str)
    }
}

/// Configuration for the HTTP server.
#[derive(Debug, Clone)]
pub struct HttpServerConfig {
    /// Host address to bind to.
    pub host: String,
    /// Port to listen on. If None, the OS will assign an available port.
    pub port: Option<u16>,
    /// Server name used in responses.
    pub server_name: String,
}

impl HttpServerConfig {
    /// Create a new configuration with the given host.
    pub fn new(host: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            port: None,
            server_name: "hyperlit-server".to_string(),
        }
    }

    /// Set the port.
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// Set the server name.
    pub fn with_server_name(mut self, name: impl Into<String>) -> Self {
        self.server_name = name.into();
        self
    }

    /// Get the address string (host:port or host for OS-assigned port).
    pub fn address(&self) -> String {
        match self.port {
            Some(port) => format!("{}:{}", self.host, port),
            None => format!("{}:0", self.host),
        }
    }
}

impl Default for HttpServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: None,
            server_name: "hyperlit-server".to_string(),
        }
    }
}

/* 📖 # Why a single HttpService trait?

The HttpService trait follows the simple "single handler" pattern (Pattern A).
The service receives raw HttpRequest objects and returns HttpResponse objects.
This gives the application full control over routing and request handling.

Benefits:
- **Simple**: No complex route registration API
- **Flexible**: Application can implement any routing logic
- **Testable**: Easy to test with MockPal's simulate_request() method
*/

/// Trait for handling HTTP requests.
///
/// Implement this trait to create an HTTP service. The service receives raw
/// HTTP requests and returns responses.
pub trait HttpService: std::fmt::Debug + Send + Sync + 'static {
    /// Handle an HTTP request and return a response.
    ///
    /// This method is called for every incoming request. The implementation
    /// should inspect the request and return an appropriate response.
    ///
    /// Errors are returned as `HyperlitResult::Err` and will be converted to
    /// HTTP error responses by the PAL implementation. All errors result in
    /// HTTP 599 status to make them easily distinguishable from successful responses.
    fn handle_request(&self, request: HttpRequest) -> crate::HyperlitResult<HttpResponse>;
}

/// Handle to a running HTTP server.
///
/// This handle allows control over the server lifecycle. When dropped, the
/// server will shut down gracefully (stop accepting new connections and
/// wait for existing ones to complete).
#[derive(Debug, Clone)]
pub struct HttpServerHandle {
    port: u16,
    shutdown: Arc<std::sync::atomic::AtomicBool>,
}

impl HttpServerHandle {
    /// Create a new handle for the given port.
    pub fn new(port: u16) -> Self {
        Self {
            port,
            shutdown: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Get the port the server is listening on.
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Get the full address (host:port) the server is listening on.
    pub fn address(&self, host: &str) -> String {
        format!("{}:{}", host, self.port)
    }

    /// Signal the server to shut down.
    ///
    /// The server will stop accepting new connections immediately. Existing
    /// connections will be allowed to complete.
    pub fn shutdown(&self) {
        self.shutdown
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }

    /// Check if the server has been signaled to shut down.
    pub fn is_shutdown(&self) -> bool {
        self.shutdown.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Access the shutdown flag (for internal use by implementations).
    pub fn shutdown_flag(&self) -> &Arc<std::sync::atomic::AtomicBool> {
        &self.shutdown
    }
}

impl Drop for HttpServerHandle {
    fn drop(&mut self) {
        // Signal shutdown when the last handle is dropped
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_method_from_str() {
        assert_eq!(HttpMethod::parse("GET"), Some(HttpMethod::Get));
        assert_eq!(HttpMethod::parse("POST"), Some(HttpMethod::Post));
        assert_eq!(HttpMethod::parse("put"), Some(HttpMethod::Put)); // Case insensitive
        assert_eq!(HttpMethod::parse("INVALID"), None);
    }

    #[test]
    fn test_http_method_display() {
        assert_eq!(format!("{}", HttpMethod::Get), "GET");
        assert_eq!(format!("{}", HttpMethod::Post), "POST");
    }

    #[test]
    fn test_http_headers() {
        let mut headers = HttpHeaders::new();
        headers.insert("Content-Type", "application/json");
        headers.insert("Authorization", "Bearer token123");

        assert_eq!(
            headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
        assert!(headers.contains("Authorization"));
        assert!(!headers.contains("X-Custom"));

        headers.remove("Authorization");
        assert!(!headers.contains("Authorization"));
    }

    #[test]
    fn test_http_body() {
        let body = HttpBody::from_string("Hello, World!");
        assert_eq!(body.as_string(), Some("Hello, World!".to_string()));
        assert_eq!(body.len(), 13);

        let empty = HttpBody::empty();
        assert!(empty.is_empty());
    }

    #[test]
    fn test_http_request() {
        let request = HttpRequest::new(HttpMethod::Get, "/api/test")
            .with_header("Accept", "application/json")
            .with_body("{\"key\": \"value\"}");

        assert_eq!(request.method(), &HttpMethod::Get);
        assert_eq!(request.path(), "/api/test");
        assert_eq!(
            request.headers().get("Accept"),
            Some(&"application/json".to_string())
        );
        assert_eq!(
            request.body().as_string(),
            Some("{\"key\": \"value\"}".to_string())
        );
    }

    #[test]
    fn test_http_response_helpers() {
        let ok = HttpResponse::ok();
        assert_eq!(ok.status(), HttpStatusCode::Ok);

        let not_found = HttpResponse::not_found();
        assert_eq!(not_found.status(), HttpStatusCode::NotFound);

        let json = HttpResponse::json("{\"data\": []}");
        assert_eq!(json.status(), HttpStatusCode::Ok);
        assert_eq!(
            json.headers().get("Content-Type"),
            Some(&"application/json".to_string())
        );

        let text = HttpResponse::text("Hello");
        assert_eq!(text.body().as_string(), Some("Hello".to_string()));
    }

    #[test]
    fn test_http_status_code_from_u16() {
        assert_eq!(HttpStatusCode::from(200), HttpStatusCode::Ok);
        assert_eq!(HttpStatusCode::from(404), HttpStatusCode::NotFound);
        assert_eq!(
            HttpStatusCode::from(500),
            HttpStatusCode::InternalServerError
        );
        assert_eq!(
            HttpStatusCode::from(999),
            HttpStatusCode::InternalServerError
        ); // Unknown defaults to 500
    }

    #[test]
    fn test_http_server_config() {
        let config = HttpServerConfig::new("127.0.0.1")
            .with_port(8080)
            .with_server_name("test-server");

        assert_eq!(config.address(), "127.0.0.1:8080");
        assert_eq!(config.server_name, "test-server");

        let default = HttpServerConfig::default();
        assert_eq!(default.address(), "127.0.0.1:0");
    }

    #[test]
    fn test_http_server_handle() {
        let handle = HttpServerHandle::new(8080);
        assert_eq!(handle.port(), 8080);
        assert_eq!(handle.address("127.0.0.1"), "127.0.0.1:8080");

        // Test shutdown flag
        assert!(!handle.is_shutdown());
        handle.shutdown();
        assert!(handle.is_shutdown());
    }

    #[test]
    fn test_http_service_trait() {
        // Simple test implementation
        #[derive(Debug)]
        struct TestService;
        impl HttpService for TestService {
            fn handle_request(&self, request: HttpRequest) -> crate::HyperlitResult<HttpResponse> {
                if request.path() == "/test" {
                    Ok(HttpResponse::text("OK"))
                } else {
                    Ok(HttpResponse::not_found())
                }
            }
        }

        let service = TestService;
        let req = HttpRequest::new(HttpMethod::Get, "/test");
        let resp = service.handle_request(req).unwrap();
        assert_eq!(resp.status(), HttpStatusCode::Ok);
        assert_eq!(resp.body().as_string(), Some("OK".to_string()));

        let req2 = HttpRequest::new(HttpMethod::Get, "/other");
        let resp2 = service.handle_request(req2).unwrap();
        assert_eq!(resp2.status(), HttpStatusCode::NotFound);
    }
}
