use std::io::Read;

#[derive(Debug)]
pub struct HttpRequest {
    pub url: String,
}

pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Box<dyn Read + 'static>,
    pub streaming: bool,
}

impl HttpResponse {}

impl HttpResponse {
    pub fn ok(body: impl Read + 'static) -> Self {
        Self {
            status: 200,
            headers: vec![],
            body: Box::new(body),
            streaming: false,
        }
    }

    pub fn error(body: impl Read + 'static) -> Self {
        Self {
            status: 599,
            headers: vec![],
            body: Box::new(body),
            streaming: false,
        }
    }

    pub fn with_content_type(mut self, content_type: impl Into<String>) -> Self {
        self.headers
            .push(("Content-Type".to_string(), content_type.into()));
        self
    }
}
