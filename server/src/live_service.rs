use crate::http_types::{HttpRequest, HttpResponse};
use hyperlit_base::result::HyperlitResult;
use hyperlit_engine::engine::HyperlitEngine;
use hyperlit_model::book::Book;
use hyperlit_model::value::Value;
use hyperlit_pal::{FilePath, PalBox};
use std::io::{Cursor, Read, Write};

pub struct LiveService {
    pal: PalBox,
    engine: HyperlitEngine,
}

pub struct LiveServiceInner {}

impl LiveService {
    pub fn new(pal: PalBox) -> LiveService {
        let book = Book::new(Value::String("Moby Dick".to_string()));
        LiveService {
            pal,
            engine: HyperlitEngine::new(book),
        }
    }

    pub fn handle_request(&self, request: &HttpRequest) -> HyperlitResult<HttpResponse> {
        let response = match request.url.as_str() {
            "/" => {
                let file = self
                    .pal
                    .read_file(&FilePath::from("server/src/assets/live_service.html"))?;
                HttpResponse::ok(file).with_content_type("text/html")
            }
            "/book.html" => {
                let book_html = self.engine.render_book_html()?;
                HttpResponse::ok(Cursor::new(book_html)).with_content_type("text/html")
            }
            "/live_service.js" => {
                let file = self
                    .pal
                    .read_file(&FilePath::from("server/src/assets/live_service.js"))?;
                HttpResponse::ok(file).with_content_type("application/javascript")
            }
            "/events" => {
                let mut response = HttpResponse::ok(Events {});
                response
                    .headers
                    .push(("Content-Type".to_string(), "text/event-stream".to_string()));
                response
            }
            _ => HttpResponse::error("File not found".as_bytes()),
        };
        Ok(response)
    }
}

struct Events {}

impl Read for Events {
    fn read(&mut self, mut buf: &mut [u8]) -> std::io::Result<usize> {
        std::thread::sleep(std::time::Duration::from_secs(1));
        let data = "data: foo\n\n";
        buf.write(data.as_bytes())
    }
}
