/* 📖 # Why an API module in hyperlit_engine?

The api module provides HTTP service implementations that expose the engine's
functionality via REST endpoints. These services implement the HttpService trait
from hyperlit_base, making them compatible with both RealPal (production) and
MockPal (testing) implementations.

## CRITICAL DESIGN PRINCIPLE: Single Service for All Endpoints

**THERE IS ONLY ONE SERVICE** - Following the "single service trait" pattern (Pattern A),
the API uses **one unified service** (`ApiService`) to handle ALL endpoints. This is a
deliberate architectural decision that provides:

1. **Simplicity**: One service to register with the PAL, one handle to manage, no complex routing
2. **Consistency**: All endpoints share the same error handling (HTTP 599) and response format
3. **Resource efficiency**: Single store handle serves all endpoints
4. **Testability**: MockPal tests only need to register one service
5. **Maintainability**: Adding new endpoints doesn't change the public API

## The Unified Service

The `ApiService` struct contains both `store: StoreHandle` and `site_info: SiteInfo`,
providing everything needed for all endpoints in a single service instance. It internally
routes requests based on the path:

- `GET /api/site` - Returns site information as JSON
- `GET /api/document/{documentid}` - Returns document as JSON
- All other paths - Returns HTTP 599

## Usage

```rust
use hyperlit_engine::{ApiService, SiteInfo, InMemoryStore, StoreHandle};
use hyperlit_base::{HttpServerConfig, RealPal, Pal};

// Create the unified service
let store = StoreHandle::new(InMemoryStore::new());
let site_info = SiteInfo::new("My Documentation")
    .with_description("Project documentation")
    .with_version("1.0.0");
let service = Box::new(ApiService::new(store, site_info));

// Register ONE service with the PAL
let config = HttpServerConfig::new("127.0.0.1").with_port(8080);
let handle = pal.start_http_server(service, config)?;
// Both /api/site and /api/document/{id} are now available
```

## Internal Organization

While the public API exposes only `ApiService`, the implementation is organized into
internal modules for code clarity:
- `service.rs` - The unified `ApiService` and `SiteInfo` types
- `document.rs` - Document endpoint implementation (internal use)
- `site.rs` - Site endpoint implementation (internal use)

This structure keeps the code modular while maintaining the single public service interface.
*/

mod service;
pub mod sse;

pub use service::{ApiService, SiteInfo};
pub use sse::{SseMessage, SseRegistry};
