use crate::http_types::{HttpRequest, HttpResponse};
use hyperlit_base::FilePath;
use hyperlit_base::result::HyperlitResult;
use hyperlit_core::config::HyperlitConfig;
use hyperlit_engine::engine::HyperlitEngine;
use hyperlit_pal::PalHandle;
use std::io::{Cursor, Read, Write};
use std::sync::RwLock;
use std::sync::mpsc::Sender;

pub struct LiveService {
    pal: PalHandle,
    engine: HyperlitEngine,
    senders: RwLock<Vec<Sender<String>>>,
}

pub struct LiveServiceInner {}

impl LiveService {
    pub fn new(pal: PalHandle) -> LiveService {
        let engine = HyperlitEngine::new_handle(pal.clone());
        engine.init();
        LiveService {
            pal,
            engine,
            senders: RwLock::new(Vec::new()),
        }
    }

    pub fn config(&self) -> HyperlitResult<HyperlitConfig> {
        self.engine.config()
    }

    pub fn reload(&self) {
        self.engine.init();
        let mut senders = self.senders.write().unwrap();
        senders.retain_mut(|sender| sender.send("reload".to_string()).is_ok());
    }

    pub fn handle_request(&self, request: &HttpRequest) -> HyperlitResult<HttpResponse> {
        let response = match request.url.as_str() {
            "/" => {
                let file = self
                    .pal
                    .read_file(&FilePath::from("ui/live_service.html"))?;
                HttpResponse::ok(file).with_content_type("text/html")
            }
            "/api/structure.json" => {
                let structure = self.engine.get_book_structure()?;
                HttpResponse::json(&structure)?
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
                self.senders.write().unwrap().push(sender);
                response
            }
            path => {
                if let Some(chapter_id) = extract_chapter_id(path) {
                    return Ok(HttpResponse::ok(Cursor::new(
                        self.engine.get_chapter_markdown(chapter_id)?,
                    )));
                }
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

fn extract_chapter_id(path: &str) -> Option<&str> {
    const PREFIX: &str = "/api/chapter/";
    const SUFFIX: &str = ".md";
    if path.starts_with(PREFIX) && path.ends_with(SUFFIX) {
        let start = PREFIX.len();
        let end = path.len() - SUFFIX.len();
        if start < end {
            return Some(&path[start..end]);
        }
    }
    None
}
