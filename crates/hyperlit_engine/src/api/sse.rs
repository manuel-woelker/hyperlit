use std::io::{self, Read};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/* 📖 # Why use SSE for hot-reload notifications?
Server-Sent Events provide a simple, browser-native way to push updates from server to client.
Unlike WebSockets, SSE is unidirectional (server → client), which is all we need for file change
notifications. The browser's EventSource API handles reconnection automatically, and the protocol
is simple enough to implement with tiny_http's synchronous model using the Read trait.
*/

#[derive(Debug, Clone)]
pub enum SseMessage {
    FileChanged { timestamp: u64 },
    KeepAlive,
}

impl SseMessage {
    /// Format message as SSE protocol text
    fn format(&self) -> String {
        match self {
            SseMessage::FileChanged { timestamp } => {
                format!(
                    "event: file-changed\ndata: {{\"timestamp\": {}}}\n\n",
                    timestamp
                )
            }
            SseMessage::KeepAlive => "event: ping\ndata: keep-alive\n\n".to_string(),
        }
    }
}

pub struct SseClient {
    id: String,
    sender: std::sync::mpsc::Sender<SseMessage>,
}

pub struct SseRegistry {
    clients: Arc<Mutex<Vec<SseClient>>>,
}

impl SseRegistry {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            clients: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// Register a new SSE client and return its ID and message receiver
    pub fn register(&self) -> (String, std::sync::mpsc::Receiver<SseMessage>) {
        let id = uuid::Uuid::new_v4().to_string();
        let (sender, receiver) = std::sync::mpsc::channel();

        let client = SseClient {
            id: id.clone(),
            sender,
        };

        let mut clients = self.clients.lock().unwrap();
        clients.push(client);
        tracing::debug!(client_id = %id, total_clients = clients.len(), "SSE client registered");

        (id, receiver)
    }

    /// Unregister a client by ID
    pub fn unregister(&self, id: &str) {
        let mut clients = self.clients.lock().unwrap();
        clients.retain(|c| c.id != id);
        tracing::debug!(client_id = %id, remaining_clients = clients.len(), "SSE client unregistered");
    }

    /// Broadcast a message to all connected clients
    pub fn broadcast(&self, message: SseMessage) {
        let mut clients = self.clients.lock().unwrap();

        // Remove clients where send fails (disconnected)
        clients.retain(|client| match client.sender.send(message.clone()) {
            Ok(_) => true,
            Err(_) => {
                tracing::debug!(client_id = %client.id, "Removing disconnected SSE client");
                false
            }
        });

        tracing::debug!(
            message_type = ?message,
            active_clients = clients.len(),
            "SSE message broadcast"
        );
    }

    /// Get the current number of connected clients
    pub fn client_count(&self) -> usize {
        self.clients.lock().unwrap().len()
    }

    /* 📖 # Why spawn a keep-alive thread?
    SSE connections can be closed by proxies or browsers after 30-60 seconds of inactivity.
    Sending periodic keep-alive messages prevents timeouts and helps detect dead connections.
    The thread runs independently and broadcasts a ping message every 30 seconds to all clients.
    */
    pub fn start_keepalive_thread(self: Arc<Self>) {
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(Duration::from_secs(30));
                self.broadcast(SseMessage::KeepAlive);
            }
        });
        tracing::info!("SSE keep-alive thread started");
    }
}

/* 📖 # Why implement Read for SseStream?
tiny_http requires response bodies to implement the Read trait for streaming.
This custom Read implementation bridges between the mpsc::Receiver (where messages arrive)
and the HTTP response stream (what tiny_http sends to the client). The read() method blocks
on receiver.recv(), which is fine because tiny_http spawns a thread per request.
*/
pub struct SseStream {
    receiver: std::sync::mpsc::Receiver<SseMessage>,
    buffer: Vec<u8>,
    position: usize,
}

impl SseStream {
    pub fn new(receiver: std::sync::mpsc::Receiver<SseMessage>) -> Self {
        Self {
            receiver,
            buffer: Vec::new(),
            position: 0,
        }
    }
}

impl Read for SseStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // If buffer is exhausted, wait for next message
        while self.position >= self.buffer.len() {
            match self.receiver.recv() {
                Ok(message) => {
                    self.buffer = message.format().into_bytes();
                    self.position = 0;
                }
                Err(_) => {
                    // Channel closed, end of stream
                    return Ok(0);
                }
            }
        }

        // Copy from buffer to output
        let remaining = self.buffer.len() - self.position;
        let to_copy = remaining.min(buf.len());
        buf[..to_copy].copy_from_slice(&self.buffer[self.position..self.position + to_copy]);
        self.position += to_copy;

        Ok(to_copy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_format() {
        let msg = SseMessage::FileChanged {
            timestamp: 1234567890,
        };
        let formatted = msg.format();
        assert!(formatted.contains("event: file-changed"));
        assert!(formatted.contains("data: {\"timestamp\": 1234567890}"));
        assert!(formatted.ends_with("\n\n"));
    }

    #[test]
    fn test_keepalive_format() {
        let msg = SseMessage::KeepAlive;
        let formatted = msg.format();
        assert!(formatted.contains("event: ping"));
        assert!(formatted.contains("data: keep-alive"));
    }

    #[test]
    fn test_registry_register() {
        let registry = SseRegistry::new();
        assert_eq!(registry.client_count(), 0);

        let (_id, _receiver) = registry.register();
        assert_eq!(registry.client_count(), 1);
    }

    #[test]
    fn test_registry_unregister() {
        let registry = SseRegistry::new();
        let (id, _receiver) = registry.register();
        assert_eq!(registry.client_count(), 1);

        registry.unregister(&id);
        assert_eq!(registry.client_count(), 0);
    }

    #[test]
    fn test_broadcast() {
        let registry = SseRegistry::new();
        let (_id1, receiver1) = registry.register();
        let (_id2, receiver2) = registry.register();

        registry.broadcast(SseMessage::KeepAlive);

        // Both receivers should get the message
        assert!(matches!(receiver1.try_recv(), Ok(SseMessage::KeepAlive)));
        assert!(matches!(receiver2.try_recv(), Ok(SseMessage::KeepAlive)));
    }

    #[test]
    fn test_stream_read() {
        let (sender, receiver) = std::sync::mpsc::channel();
        let mut stream = SseStream::new(receiver);

        // Send a message
        sender.send(SseMessage::KeepAlive).unwrap();

        // Read from stream
        let mut buf = [0u8; 1024];
        let n = stream.read(&mut buf).unwrap();

        assert!(n > 0);
        let text = String::from_utf8_lossy(&buf[..n]);
        assert!(text.contains("event: ping"));
    }
}
