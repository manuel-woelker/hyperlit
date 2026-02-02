/* 📖 # Why create a DocumentStore trait?

The DocumentStore trait abstracts how documents are persisted and retrieved.
This allows the engine to work with different storage backends:

1. **In-memory store**: For testing and small datasets
2. **File-based store**: For simple persistence (JSON, TOML, etc.)
3. **Database store**: For large-scale deployments (SQLite, PostgreSQL, etc.)

By defining a trait, we can:
- Write code that doesn't care about storage implementation
- Test with fast in-memory stores
- Swap storage backends without changing business logic
- Mock storage for unit tests

The trait design follows the PAL pattern used elsewhere in the codebase,
but focuses specifically on document persistence operations.
*/

use std::sync::Arc;

use parking_lot::RwLock;

use hyperlit_base::HyperlitResult;

use crate::document::{Document, DocumentId};

/// Trait for document storage implementations.
///
/// Provides basic CRUD operations for documents, keyed by their unique ID.
/// All operations return `HyperlitResult` for consistent error handling.
pub trait DocumentStore: Send + Sync + 'static {
    /// Store a document, keyed by its ID.
    ///
    /// If a document with the same ID already exists, it will be replaced.
    /// The document's ID is extracted from the document itself.
    ///
    /// # Arguments
    /// * `doc` - The document to store
    ///
    /// # Returns
    /// The ID of the stored document
    fn insert(&mut self, doc: Document) -> HyperlitResult<DocumentId>;

    /// Retrieve a document by its ID.
    ///
    /// # Arguments
    /// * `id` - The document ID to look up
    ///
    /// # Returns
    /// * `Ok(Some(doc))` - If the document exists
    /// * `Ok(None)` - If no document with that ID exists
    fn get(&self, id: &DocumentId) -> HyperlitResult<Option<Document>>;

    /// Check if a document with the given ID exists.
    ///
    /// # Arguments
    /// * `id` - The document ID to check
    fn contains(&self, id: &DocumentId) -> HyperlitResult<bool>;

    /// List all documents in the store.
    ///
    /// # Returns
    /// A vector of all stored documents. Order is not guaranteed.
    fn list(&self) -> HyperlitResult<Vec<Document>>;

    /// Remove a document by its ID.
    ///
    /// # Arguments
    /// * `id` - The document ID to remove
    ///
    /// # Returns
    /// * `Ok(Some(doc))` - The removed document if it existed
    /// * `Ok(None)` - If no document with that ID existed
    fn remove(&mut self, id: &DocumentId) -> HyperlitResult<Option<Document>>;

    /// Clear all documents from the store.
    fn clear(&mut self) -> HyperlitResult<()>;

    /// Get the number of documents in the store.
    fn len(&self) -> HyperlitResult<usize>;

    /// Returns true if the store contains no documents.
    fn is_empty(&self) -> HyperlitResult<bool>;
}

/// A thread-safe handle to a document store.
///
/// StoreHandle provides cheap cloning (via Arc) and interior mutability (via RwLock),
/// allowing the store to be shared across async tasks and threads.
///
/// This follows the same pattern as `PalHandle` in hyperlit_base.
#[derive(Clone)]
pub struct StoreHandle(Arc<RwLock<dyn DocumentStore>>);

impl StoreHandle {
    /// Create a new StoreHandle wrapping the given store implementation.
    pub fn new<S: DocumentStore>(store: S) -> Self {
        Self(Arc::new(RwLock::new(store)))
    }

    /// Store a document.
    ///
    /// See [`DocumentStore::insert`] for details.
    pub fn insert(&self, doc: Document) -> HyperlitResult<DocumentId> {
        self.0.write().insert(doc)
    }

    /// Retrieve a document by ID.
    ///
    /// See [`DocumentStore::get`] for details.
    pub fn get(&self, id: &DocumentId) -> HyperlitResult<Option<Document>> {
        self.0.read().get(id)
    }

    /// Check if a document exists.
    ///
    /// See [`DocumentStore::contains`] for details.
    pub fn contains(&self, id: &DocumentId) -> HyperlitResult<bool> {
        self.0.read().contains(id)
    }

    /// List all documents.
    ///
    /// See [`DocumentStore::list`] for details.
    pub fn list(&self) -> HyperlitResult<Vec<Document>> {
        self.0.read().list()
    }

    /// Remove a document by ID.
    ///
    /// See [`DocumentStore::remove`] for details.
    pub fn remove(&self, id: &DocumentId) -> HyperlitResult<Option<Document>> {
        self.0.write().remove(id)
    }

    /// Clear all documents.
    ///
    /// See [`DocumentStore::clear`] for details.
    pub fn clear(&self) -> HyperlitResult<()> {
        self.0.write().clear()
    }

    /// Get the number of documents.
    ///
    /// See [`DocumentStore::len`] for details.
    pub fn len(&self) -> HyperlitResult<usize> {
        self.0.read().len()
    }

    /// Check if the store is empty.
    ///
    /// See [`DocumentStore::is_empty`] for details.
    pub fn is_empty(&self) -> HyperlitResult<bool> {
        self.0.read().is_empty()
    }
}
