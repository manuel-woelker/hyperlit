/* 📖 # Why provide an in-memory store implementation?

The InMemoryStore provides a simple, fast storage backend that keeps all
documents in a HashMap in memory. This is useful for:

1. **Testing**: Fast, isolated tests without file system or database setup
2. **Small datasets**: Development environments with limited documents
3. **Caching**: Temporary storage during processing pipelines
4. **Prototyping**: Quick iteration without persistence complexity

The implementation uses std::collections::HashMap for O(1) lookups by ID.
Documents are stored as values (cloned) since Document implements Clone.

While not suitable for production persistence, this implementation serves
as a reference for how to implement the DocumentStore trait and provides
immediate utility for testing and development.
*/

use std::collections::HashMap;

use hyperlit_base::HyperlitResult;

use crate::document::{Document, DocumentId};
use crate::store::traits::DocumentStore;

/// An in-memory document store backed by a HashMap.
///
/// Stores documents in memory with O(1) lookup by ID. Documents are cloned
/// on insertion, so the store owns its own copies.
///
/// This is the simplest DocumentStore implementation and is primarily
/// intended for testing and development use.
///
/// # Example
///
/// ```
/// use hyperlit_base::FilePath;
/// use hyperlit_engine::{Document, DocumentSource, SourceType, DocumentStore};
/// use hyperlit_engine::store::InMemoryStore;
/// use std::collections::HashSet;
///
/// let mut store = InMemoryStore::new();
/// let source = DocumentSource::new(
///     SourceType::CodeComment,
///     FilePath::from("src/main.rs"),
///     42,
/// );
///
/// let existing = HashSet::new();
/// let doc = Document::new(
///     "Test Document".to_string(),
///     "Content here".to_string(),
///     source,
///     None,
///     &existing,
/// );
///
/// let id = doc.id().clone();
/// store.insert(doc).unwrap();
///
/// assert!(store.contains(&id).unwrap());
/// assert_eq!(store.len().unwrap(), 1);
/// ```
#[derive(Debug, Default)]
pub struct InMemoryStore {
    documents: HashMap<DocumentId, Document>,
}

impl InMemoryStore {
    /// Create a new, empty in-memory store.
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
        }
    }

    /// Create a new store with a specific capacity.
    ///
    /// # Arguments
    /// * `capacity` - The initial capacity to allocate
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            documents: HashMap::with_capacity(capacity),
        }
    }
}

impl DocumentStore for InMemoryStore {
    fn insert(&mut self, doc: Document) -> HyperlitResult<DocumentId> {
        let id = doc.id().clone();
        self.documents.insert(id.clone(), doc);
        Ok(id)
    }

    fn get(&self, id: &DocumentId) -> HyperlitResult<Option<Document>> {
        Ok(self.documents.get(id).cloned())
    }

    fn contains(&self, id: &DocumentId) -> HyperlitResult<bool> {
        Ok(self.documents.contains_key(id))
    }

    fn list(&self) -> HyperlitResult<Vec<Document>> {
        Ok(self.documents.values().cloned().collect())
    }

    fn remove(&mut self, id: &DocumentId) -> HyperlitResult<Option<Document>> {
        Ok(self.documents.remove(id))
    }

    fn clear(&mut self) -> HyperlitResult<()> {
        self.documents.clear();
        Ok(())
    }

    fn len(&self) -> HyperlitResult<usize> {
        Ok(self.documents.len())
    }

    fn is_empty(&self) -> HyperlitResult<bool> {
        Ok(self.documents.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{DocumentSource, SourceType};
    use hyperlit_base::FilePath;
    use std::collections::HashSet;

    fn create_test_document(title: &str, existing: &HashSet<String>) -> Document {
        let source =
            DocumentSource::new(SourceType::CodeComment, FilePath::from("src/test.rs"), 10);

        Document::new(
            title.to_string(),
            format!("Content for {}", title),
            source,
            None,
            existing,
        )
    }

    #[test]
    fn test_store_new() {
        let store = InMemoryStore::new();
        assert!(store.is_empty().unwrap());
        assert_eq!(store.len().unwrap(), 0);
    }

    #[test]
    fn test_store_insert_and_get() {
        let mut store = InMemoryStore::new();
        let existing = HashSet::new();
        let doc = create_test_document("Test Doc", &existing);
        let id = doc.id().clone();

        let stored_id = store.insert(doc).unwrap();
        assert_eq!(stored_id, id);

        let retrieved = store.get(&id).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().title(), "Test Doc");
    }

    #[test]
    fn test_store_contains() {
        let mut store = InMemoryStore::new();
        let existing = HashSet::new();
        let doc = create_test_document("Test Doc", &existing);
        let id = doc.id().clone();

        assert!(!store.contains(&id).unwrap());
        store.insert(doc).unwrap();
        assert!(store.contains(&id).unwrap());
    }

    #[test]
    fn test_store_get_nonexistent() {
        let store = InMemoryStore::new();
        let existing = HashSet::new();
        let doc = create_test_document("Test Doc", &existing);
        let id = doc.id().clone();

        let retrieved = store.get(&id).unwrap();
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_store_list() {
        let mut store = InMemoryStore::new();
        let existing = HashSet::new();

        let doc1 = create_test_document("First Doc", &existing);
        let id1 = doc1.id().clone();
        store.insert(doc1).unwrap();

        let mut existing = existing;
        existing.insert(id1.as_str().to_string());
        let doc2 = create_test_document("Second Doc", &existing);
        store.insert(doc2).unwrap();

        let list = store.list().unwrap();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_store_remove() {
        let mut store = InMemoryStore::new();
        let existing = HashSet::new();
        let doc = create_test_document("Test Doc", &existing);
        let id = doc.id().clone();

        store.insert(doc).unwrap();
        assert_eq!(store.len().unwrap(), 1);

        let removed = store.remove(&id).unwrap();
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().title(), "Test Doc");
        assert_eq!(store.len().unwrap(), 0);
    }

    #[test]
    fn test_store_remove_nonexistent() {
        let mut store = InMemoryStore::new();
        let existing = HashSet::new();
        let doc = create_test_document("Test Doc", &existing);
        let id = doc.id().clone();

        let removed = store.remove(&id).unwrap();
        assert!(removed.is_none());
    }

    #[test]
    fn test_store_clear() {
        let mut store = InMemoryStore::new();
        let existing = HashSet::new();

        let doc1 = create_test_document("First Doc", &existing);
        let id1 = doc1.id().clone();
        store.insert(doc1).unwrap();

        let mut existing = existing;
        existing.insert(id1.as_str().to_string());
        let doc2 = create_test_document("Second Doc", &existing);
        store.insert(doc2).unwrap();

        assert_eq!(store.len().unwrap(), 2);
        store.clear().unwrap();
        assert!(store.is_empty().unwrap());
    }

    #[test]
    fn test_store_update_existing() {
        let mut store = InMemoryStore::new();
        let existing = HashSet::new();
        let doc = create_test_document("Test Doc", &existing);
        let id = doc.id().clone();

        store.insert(doc).unwrap();

        // Create a new document with same title (will get same ID due to empty existing set)
        // But for this test, we manually verify replacement
        let existing2 = HashSet::new();
        let doc2 = create_test_document("Test Doc Updated", &existing2);
        let id2 = doc2.id().clone();

        // Different IDs since the titles are different
        assert_ne!(id, id2);

        store.insert(doc2.clone()).unwrap();
        assert_eq!(store.len().unwrap(), 2);

        // Get the second document back
        let retrieved = store.get(&id2).unwrap();
        assert_eq!(retrieved.unwrap().title(), "Test Doc Updated");
    }

    #[test]
    fn test_store_with_capacity() {
        let store = InMemoryStore::with_capacity(100);
        assert!(store.is_empty().unwrap());
        assert_eq!(store.len().unwrap(), 0);
    }

    #[test]
    fn test_store_handle_basic_operations() {
        use crate::store::StoreHandle;

        let handle = StoreHandle::new(InMemoryStore::new());
        let existing = HashSet::new();
        let doc = create_test_document("Handle Test", &existing);
        let id = doc.id().clone();

        handle.insert(doc).unwrap();
        assert!(handle.contains(&id).unwrap());
        assert_eq!(handle.len().unwrap(), 1);

        let retrieved = handle.get(&id).unwrap();
        assert!(retrieved.is_some());

        handle.clear().unwrap();
        assert!(handle.is_empty().unwrap());
    }

    #[test]
    fn test_store_handle_clone() {
        use crate::store::StoreHandle;

        let handle1 = StoreHandle::new(InMemoryStore::new());
        let existing = HashSet::new();
        let doc = create_test_document("Clone Test", &existing);
        let id = doc.id().clone();

        handle1.insert(doc).unwrap();

        let handle2 = handle1.clone();
        assert!(handle2.contains(&id).unwrap());
        assert_eq!(handle2.len().unwrap(), 1);
    }
}
