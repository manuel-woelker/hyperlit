use hyperlit_base::result::HyperlitResult;
use serde::Serialize;
use std::io::{Cursor, Read};
use std::sync::mpsc::{Receiver, Sender};

#[derive(Debug)]
pub struct HttpRequest {
    pub url: String,
}

pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Box<dyn Read + 'static>,
    pub streaming: bool,
    pub events: Option<Receiver<String>>,
}

impl HttpResponse {}

impl HttpResponse {
    pub fn ok(body: impl Read + 'static) -> Self {
        Self {
            status: 200,
            headers: vec![],
            body: Box::new(body),
            streaming: false,
            events: None,
        }
    }

    pub fn ok_buffer(body: impl Into<Vec<u8>>) -> Self {
        Self {
            status: 200,
            headers: vec![],
            body: Box::new(Cursor::new(body.into())),
            streaming: false,
            events: None,
        }
    }

    pub fn json(body: &impl Serialize) -> HyperlitResult<Self> {
        let json = serde_json::to_string_pretty(body)?;
        Ok(Self::ok_buffer(json).with_content_type("application/json"))
    }

    pub fn error(body: impl Read + 'static) -> Self {
        Self {
            status: 599,
            headers: vec![],
            body: Box::new(body),
            streaming: false,
            events: None,
        }
    }

    pub fn with_content_type(mut self, content_type: impl Into<String>) -> Self {
        self.headers
            .push(("Content-Type".to_string(), content_type.into()));
        self
    }

    pub fn set_streaming(&mut self) -> Sender<String> {
        self.streaming = true;
        let (tx, rx) = std::sync::mpsc::channel();
        self.events = Some(rx);
        tx
    }
}
