use crate::http_types::{HttpRequest, HttpResponse};
use hyperlit_base::result::HyperlitResult;
use hyperlit_engine::engine::HyperlitEngine;
use hyperlit_pal::{FilePath, PalHandle};
use std::io::{Cursor, Read, Write};
use std::thread::Builder;
use std::time::Duration;

pub struct LiveService {
    pal: PalHandle,
    engine: HyperlitEngine,
}

pub struct LiveServiceInner {}

impl LiveService {
    pub fn new(pal: PalHandle) -> LiveService {
        let engine = HyperlitEngine::new_handle(pal.clone());
        engine.init();
        LiveService { pal, engine }
    }

    pub fn handle_request(&self, request: &HttpRequest) -> HyperlitResult<HttpResponse> {
        let response = match request.url.as_str() {
            "/" => {
                let file = self
                    .pal
                    .read_file(&FilePath::from("ui/live_service.html"))?;
                HttpResponse::ok(file).with_content_type("text/html")
            }
            "/book.html" => {
                let book_html = self.engine.render_book_html()?;
                HttpResponse::ok(Cursor::new(book_html)).with_content_type("text/html")
            }
            "/events" => {
                let mut response = HttpResponse::ok(Events {});
                response
                    .headers
                    .push(("Content-Type".to_string(), "text/event-stream".to_string()));
                let sender = response.set_streaming();
                Builder::new()
                    .name("Event pusher".to_string())
                    .spawn(move || {
                        let mut counter = 0;
                        loop {
                            let result = sender.send(format!("event {counter}"));
                            if result.is_err() {
                                // Other end hung up on us, do not log anything
                                return;
                            }
                            counter += 1;
                            std::thread::sleep(Duration::from_millis(1000));
                        }
                    })?;
                response
            }
            path => {
                let file_path = FilePath::from("ui").join(path);
                let file_content = self.pal.read_file(&file_path)?;
                let content_type = match file_path.extension() {
                    Some("css") => "text/css",
                    Some("js") => "application/javascript",
                    _ => "application/unknown",
                };
                HttpResponse::ok(file_content).with_content_type(content_type)
            }
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
