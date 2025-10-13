use crate::http_types::HttpRequest;
use crate::live_service::LiveService;
use chunked_transfer::Encoder;
use hyperlit_base::err;
use hyperlit_base::result::HyperlitResult;
use hyperlit_pal::{Pal, PalBox};
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;
use tiny_http::{Header, Response, Server, StatusCode};

pub struct HyperlitServer {
    pal: PalBox,
}

impl HyperlitServer {
    pub fn new(pal: impl Pal + 'static) -> Self {
        Self { pal: Arc::new(pal) }
    }

    pub fn run(self) -> HyperlitResult<()> {
        let live_service = LiveService::new(self.pal.clone());
        let server =
            Server::http("0.0.0.0:3333").map_err(|e| err!("Could not start server: {}", e))?;
        let _server_thread = std::thread::Builder::new()
            .name("HTTP server (3333)".to_string())
            .spawn(move || {
                loop {
                    let tiny_request = server.recv();
                    let tiny_request = match tiny_request {
                        Ok(tiny_request) => tiny_request,
                        Err(e) => {
                            eprintln!("Error receiving request: {}", e);
                            continue;
                        }
                    };
                    if tiny_request.url() == "/events" {
                        std::thread::Builder::new()
                            .name("events sender".to_string())
                            .spawn(move || {
                                let mut writer = tiny_request.into_writer();

                                (write!(writer, "HTTP/1.1 200 OK\r\n")).unwrap();
                                (write!(writer, "Content-Type: text/event-stream\r\n")).unwrap();
                                (write!(writer, "Transfer-Encoding: Chunked\r\n")).unwrap();
                                (write!(writer, "Connection: keep-alive\r\n")).unwrap();
                                (write!(writer, "\r\n")).unwrap();
                                writer.flush().unwrap();
                                let mut writer = Encoder::new(writer);
                                let mut counter = 0;
                                loop {
                                    write!(writer, "data: foobar {counter}\n\n").unwrap();
                                    writer.flush().unwrap();
                                    writer.get_mut().flush().unwrap();
                                    counter += 1;
                                    std::thread::sleep(Duration::from_millis(1000));
                                }
                            })
                            .unwrap();
                        continue;
                    }
                    let request = HttpRequest {
                        url: tiny_request.url().to_string(),
                    };
                    let result = match live_service.handle_request(&request) {
                        Ok(response) => {
                            let mut tiny_response =
                                Response::new_empty(StatusCode(response.status));
                            for (key, value) in response.headers {
                                tiny_response.add_header(
                                    Header::from_bytes(key.as_bytes(), value.as_bytes()).unwrap(),
                                );
                            }
                            let tiny_response = tiny_response.with_data(response.body, None);
                            tiny_request.respond(tiny_response)
                        }
                        Err(e) => {
                            eprintln!("Error handling request: {}", e);
                            let tiny_response = Response::from_string("Internal server error");
                            tiny_request.respond(tiny_response)
                        }
                    };
                    if let Err(e) = result {
                        eprintln!("Error sending response: {}", e);
                        continue;
                    };
                }
            });

        Ok(())
    }
}
