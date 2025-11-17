use crate::http_types::{HttpRequest, HttpResponse};
use crate::live_service::LiveService;
use chunked_transfer::Encoder;
use hyperlit_base::error::{bail, err};
use hyperlit_base::result::HyperlitResult;
use hyperlit_base::{FilePath, log_error};
use hyperlit_pal::{Pal, PalHandle};
use std::io::Write;
use std::sync::Arc;
use tiny_http::{Header, Request, Response, Server, StatusCode};
use tracing::{debug, info};

pub struct HyperlitServer {
    pal: PalHandle,
}

impl HyperlitServer {
    pub fn new(pal: impl Pal + 'static) -> Self {
        Self {
            pal: PalHandle::new(pal),
        }
    }

    pub fn run(self) -> HyperlitResult<()> {
        let live_service = Arc::new(LiveService::new(self.pal.clone()));
        let live_service_clone = live_service.clone();
        let config = live_service.config()?;
        let pal = self.pal.clone();
        let _watcher_thread = std::thread::Builder::new()
            .name("File change watcher".to_string())
            .spawn(move || {
                info!("File change watcher started");
                let result = || -> HyperlitResult<()> {
                    let live_service_clone2 = live_service_clone.clone();
                    let live_service_clone3 = live_service_clone.clone();
                    pal.watch_directory(
                        &FilePath::from(&config.src_directory),
                        &config.src_globs,
                        Box::new(move |_event| {
                            info!("Source contents changed, triggering reload...");
                            live_service_clone.reload();
                        }),
                    )?;
                    pal.watch_directory(
                        &FilePath::from(&config.docs_directory),
                        &config.doc_globs,
                        Box::new(move |_event| {
                            info!("Doc contents changed, triggering reload...");
                            debug!("Changed files: {:?}", _event.changed_files);
                            live_service_clone2.reload();
                        }),
                    )?;
                    pal.watch_directory(
                        &FilePath::from("."),
                        &["hyperlit.toml".to_string()],
                        Box::new(move |_event| {
                            info!("Hyperlit toml changed, triggering reload...");
                            debug!("Changed files: {:?}", _event.changed_files);
                            live_service_clone3.reload();
                        }),
                    )?;
                    Ok(())
                };
                if let Err(e) = result() {
                    log_error!("Error watching files: {}", e);
                }
            })?;
        let port: u16 = 3333;
        info!("Starting HTTP server on port {}", port);
        let server =
            Server::http(("0.0.0.0", port)).map_err(|e| err!("Could not start server: {}", e))?;
        let _server_thread = std::thread::Builder::new()
            .name(format!("HTTP server ({port})"))
            .spawn(move || {
                info!("HTTP server started on port {}", port);
                loop {
                    let tiny_request = server.recv();
                    let tiny_request = match tiny_request {
                        Ok(tiny_request) => tiny_request,
                        Err(e) => {
                            log_error!("Error receiving request: {:?}", e);
                            continue;
                        }
                    };
                    let request = HttpRequest {
                        url: tiny_request.url().to_string(),
                    };
                    let request_result = live_service.handle_request(&request);
                    let result = (|| -> HyperlitResult<()> {
                        match request_result {
                            Ok(response) => {
                                if response.streaming {
                                    send_streaming_response(tiny_request, response)?;
                                } else {
                                    let mut tiny_response =
                                        Response::new_empty(StatusCode(response.status));
                                    for (key, value) in response.headers {
                                        tiny_response.add_header(
                                            Header::from_bytes(key.as_bytes(), value.as_bytes())
                                                .unwrap(),
                                        );
                                    }
                                    let tiny_response =
                                        tiny_response.with_data(response.body, None);
                                    tiny_request.respond(tiny_response)?;
                                }
                            }
                            Err(e) => {
                                log_error!("Error handling request: {:?}", e);
                                let tiny_response = Response::from_string(format!(
                                    "<pre>Internal server error:\n {:?}</pre>",
                                    e
                                ))
                                .with_status_code(500);
                                tiny_request.respond(tiny_response)?;
                            }
                        }
                        Ok(())
                    })();
                    if let Err(e) = result {
                        log_error!("Error sending response: {:?}", e);
                        continue;
                    };
                }
            });

        Ok(())
    }
}

fn send_streaming_response(tiny_request: Request, response: HttpResponse) -> HyperlitResult<()> {
    std::thread::Builder::new()
        .name("events sender".to_string())
        .spawn(move || {
            let mut writer = tiny_request.into_writer();

            (write!(writer, "HTTP/1.1 200 OK\r\n"))?;
            (write!(writer, "Content-Type: text/event-stream\r\n"))?;
            (write!(writer, "Transfer-Encoding: Chunked\r\n"))?;
            (write!(writer, "Connection: keep-alive\r\n"))?;
            (write!(writer, "\r\n"))?;
            writer.flush()?;
            let mut writer = Encoder::new(writer);
            let Some(rx) = response.events else {
                bail!("No events receiver set in streaming response");
            };
            if let Err(err) = (|| -> HyperlitResult<()> {
                loop {
                    match rx.recv() {
                        Ok(msg) => {
                            // TODO: escape newlines
                            write!(writer, "data: {msg}\n\n")?;
                            writer.flush()?;
                            writer.get_mut().flush()?;
                        }
                        Err(err) => {
                            log_error!("Error receiving streaming response: {:?}", err);
                            break;
                        }
                    }
                }
                Ok(())
            })() {
                if let Some(io_error) = err.downcast_ref::<std::io::Error>()
                    && (io_error.kind() == std::io::ErrorKind::BrokenPipe
                        || io_error.kind() == std::io::ErrorKind::ConnectionAborted)
                {
                    // Client hung up on us, do not log anything
                } else {
                    log_error!("Error sending streaming response: {:?}", err);
                }
            }
            Ok(())
        })?;
    Ok(())
}
