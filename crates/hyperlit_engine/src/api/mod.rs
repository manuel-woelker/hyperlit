/* 📖 # Why an API module in hyperlit_engine?

The api module provides HTTP service implementations that expose the engine's
functionality via REST endpoints. These services implement the HttpService trait
from hyperlit_base, making them compatible with both RealPal (production) and
MockPal (testing) implementations.

Current services:
- DocumentService: Serves documents at /api/document/{documentid}

These services use StoreHandle for thread-safe document access and return
JSON responses for easy integration with web clients.
*/

mod document;

pub use document::DocumentService;
