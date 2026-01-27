/* 📖 # Why have hyperlit_base as a core library?
hyperlit_base provides the foundational error handling and types used across all crates.
This ensures consistency in error handling and prevents circular dependencies between crates.
*/

pub mod error;
mod error_tests;
pub mod tracing;

// Re-export commonly used types for convenience
pub use error::{HyperlitError, HyperlitResult, ResultExt};
