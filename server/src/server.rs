use crate::http_types::{HttpRequest, HttpResponse};
use crate::live_service::LiveService;
use chunked_transfer::Encoder;
use hyperlit_base::error::{bail, err};
use hyperlit_base::log_error;
use hyperlit_base::logging::init_logging;
use hyperlit_base::result::HyperlitResult;
use hyperlit_pal::{Pal, PalHandle};
use std::convert::Infallible;
use std::io::Write;
use std::time::Duration;
use tiny_http::{Header, Request, Response, Server, StatusCode};
use tracing::info;

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
        init_logging();
        let live_service = LiveService::new(self.pal.clone());
        let port: u16 = 3333;
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
                    if tiny_request.url() == "/events" {
                        std::thread::Builder::new()
                            .name("events sender".to_string())
                            .spawn(move || {
                                let result = || -> HyperlitResult<Infallible> {
                                    let mut writer = tiny_request.into_writer();

                                    (write!(writer, "HTTP/1.1 200 OK\r\n"))?;
                                    (write!(writer, "Content-Type: text/event-stream\r\n"))?;
                                    (write!(writer, "Transfer-Encoding: Chunked\r\n"))?;
                                    (write!(writer, "Connection: keep-alive\r\n"))?;
                                    (write!(writer, "\r\n"))?;
                                    writer.flush()?;
                                    let mut writer = Encoder::new(writer);
                                    let mut counter = 0;
                                    loop {
                                        write!(writer, "data: foobar {counter}\n\n")?;
                                        writer.flush()?;
                                        writer.get_mut().flush()?;
                                        counter += 1;
                                        std::thread::sleep(Duration::from_millis(1000));
                                    }
                                }();
                                let Err(error) = result;
                                if let Some(io_error) = error.downcast_ref::<std::io::Error>()
                                    && (io_error.kind() == std::io::ErrorKind::BrokenPipe
                                        || io_error.kind() == std::io::ErrorKind::ConnectionAborted)
                                {
                                    // Client hung up on us, do not log anything
                                    return;
                                }
                                log_error!("Error sending events: {:?}", error);
                            })
                            .unwrap();
                        continue;
                    }
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
                                ));
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
                log_error!("Error sending streaming response: {:?}", err);
            }
            Ok(())
        })?;
    Ok(())
}
