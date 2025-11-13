use crate::http_types::{HttpRequest, HttpResponse};
use hyperlit_base::FilePath;
use hyperlit_base::result::{Context, HyperlitResult};
use hyperlit_core::config::HyperlitConfig;
use hyperlit_engine::engine::HyperlitEngine;
use hyperlit_pal::PalHandle;
use std::io::{Cursor, Read, Write};
use std::sync::RwLock;
use std::sync::mpsc::Sender;
use zip::ZipArchive;

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
            "/api/structure.json" => {
                let structure = self.engine.get_book_structure()?;
                HttpResponse::json(&structure)?
            }
            "/book.html" => {
                let book_html = self.engine.render_book_html()?;
                HttpResponse::ok_buffer(book_html).with_content_type("text/html")
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
                let path = path.strip_prefix("/").unwrap_or(path);
                // ignore everything after the first "?"
                let mut path = path.split_once('?').unwrap_or((path, "")).0;
                if path.is_empty() {
                    path = "index.html";
                }
                self.serve_asset(path)?
            }
        };
        Ok(response)
    }

    fn serve_asset(&self, filename: &str) -> HyperlitResult<HttpResponse> {
        //        let executable_file = self.pal.read_file(&FilePath::from("../target/ui.zip"))?;
        /*        let executable_file = self
        .pal
        .read_file(&FilePath::from("../target/release/hyperlit.exe"))?;*/
        let executable_file = self.pal.read_executable_file()?;
        let mut zip = ZipArchive::new(executable_file)?;
        let mut file = zip
            .by_name(filename)
            .with_context(|| format!("Unable to open file '{}'", filename))?;
        /*                let file = self
        .pal
        .read_file(&FilePath::from("ui/live_service.html"))?;*/
        let mut file_content = Vec::new();
        file.read_to_end(&mut file_content)?;
        let file_path = FilePath::from(filename);
        let content_type = match file_path.extension() {
            Some("css") => "text/css",
            Some("js") => "application/javascript",
            Some("html") => "text/html",
            Some("png") => "image/png",
            _ => "application/unknown",
        };

        Ok(HttpResponse::ok_buffer(file_content).with_content_type(content_type))
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
