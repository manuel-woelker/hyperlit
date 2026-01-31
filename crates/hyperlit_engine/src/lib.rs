pub mod comment_parser;
pub mod config;
pub mod document;
pub mod extractor;
pub mod scanner;
pub mod store;

pub use comment_parser::CommentParser;
pub use config::{Config, DirectoryConfig, load_config};
pub use document::{ByteRange, Document, DocumentId, DocumentMetadata, DocumentSource, SourceType};
pub use extractor::{ExtractionError, ExtractionResult, extract_documents};
pub use scanner::{ScanError, ScanResult, scan_files};
pub use store::{DocumentStore, InMemoryStore, StoreHandle};
