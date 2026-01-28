/* 📖 # Why have a dedicated document model?

The Document type represents a unit of extracted documentation that can come from:
1. Code comments with 📖 markers (e.g., `/* 📖 # Why use Arc? ... */`)
2. Standalone markdown files (e.g., `docs/design.md`)

A document model allows us to uniformly represent these different sources as a single,
queryable data structure. The model separates content (what's documented) from source
(where it came from), making it easy to track provenance and enable features like
"show me all documentation from this file".

This is a data model only - extraction logic comes later.
*/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use hyperlit_base::FilePath;

/// A documentation block extracted from source code or markdown files.
///
/// A document represents a single unit of extracted documentation, whether from:
/// - A code comment with a 📖 marker
/// - A standalone markdown file
///
/// The document has a unique ID, title, content, and source information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Document {
    id: DocumentId,
    title: String,
    content: String,
    source: DocumentSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<DocumentMetadata>,
}

/// Unique identifier for a document.
///
/// IDs are generated from the document title by slugifying it (converting to URL-friendly
/// lowercase-with-hyphens format). When multiple documents share the same title, sequential
/// numbers are appended: "title", "title-1", "title-2", etc.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DocumentId(String);

/* 📖 # Why use title-based IDs with collision handling?

Document IDs are generated from the document title by slugifying it (converting
to URL-friendly lowercase-with-hyphens format). When multiple documents share the
same title, we append sequential numbers: "title", "title-1", "title-2", etc.

Benefits of this approach:
1. **Human-readable**: IDs like "why-use-arc" are easier to understand than hashes
2. **URL-friendly**: Can be used directly in URLs without escaping
3. **Predictable**: Similar titles produce similar IDs, aiding debugging
4. **Collision handling**: Sequential numbering ensures uniqueness

The trade-off is that ID generation requires tracking existing IDs, but this is
acceptable since documents are typically created in batch during indexing.
*/

impl DocumentId {
    /// Create a DocumentId from a title, ensuring uniqueness with collision counter.
    ///
    /// If the base slug already exists in `existing_ids`, appends "-N" where N is the
    /// collision count until a unique ID is found.
    ///
    /// # Arguments
    /// * `title` - The document title to slugify
    /// * `existing_ids` - Set of already-used IDs for collision detection
    ///
    /// # Examples
    /// ```
    /// use std::collections::HashSet;
    /// use hyperlit_engine::DocumentId;
    ///
    /// let existing = HashSet::new();
    /// let id = DocumentId::from_title("Why Use Arc?", &existing);
    /// assert_eq!(id.as_str(), "why-use-arc");
    /// ```
    pub fn from_title(title: &str, existing_ids: &std::collections::HashSet<String>) -> Self {
        let base_slug = slugify(title);

        // Check for collisions and add sequential number if needed
        let mut candidate = base_slug.clone();
        let mut counter = 1;

        while existing_ids.contains(&candidate) {
            candidate = format!("{}-{}", base_slug, counter);
            counter += 1;
        }

        DocumentId(candidate)
    }

    /// Returns the ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for DocumentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Convert a string to a URL-friendly slug.
///
/// Converts to lowercase, replaces spaces/special characters with hyphens,
/// and removes consecutive hyphens.
///
/// This is an internal function used for generating document IDs. Examples:
/// - "Why Use Arc?" becomes "why-use-arc"
/// - "Hello   World" becomes "hello-world"
/// - "CamelCase" becomes "camelcase"
fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '-' || c == '_' {
                '-'
            } else {
                '\0' // Remove special characters
            }
        })
        .filter(|&c| c != '\0')
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Source location and type information for a document.
///
/// Tracks where a document came from (file path, line number), what type
/// of source it is (code comment or markdown file), and optionally the byte range
/// of the extracted content within the source file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentSource {
    source_type: SourceType,
    file_path: FilePath,
    line_number: usize,
    byte_range: Option<ByteRange>,
}

// Manual implementations of Serialize/Deserialize for DocumentSource
// because FilePath doesn't derive these traits directly.
impl Serialize for DocumentSource {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let num_fields = if self.byte_range.is_some() { 4 } else { 3 };
        let mut state = serializer.serialize_struct("DocumentSource", num_fields)?;
        state.serialize_field("source_type", &self.source_type)?;
        state.serialize_field("file_path", &self.file_path.to_string())?;
        state.serialize_field("line_number", &self.line_number)?;
        if let Some(byte_range) = self.byte_range {
            state.serialize_field("byte_range", &byte_range)?;
        }
        state.end()
    }
}

impl<'de> Deserialize<'de> for DocumentSource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            SourceType,
            FilePath,
            LineNumber,
            ByteRange,
        }

        struct DocumentSourceVisitor;

        impl<'de> Visitor<'de> for DocumentSourceVisitor {
            type Value = DocumentSource;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct DocumentSource")
            }

            fn visit_map<V>(self, mut map: V) -> Result<DocumentSource, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut source_type = None;
                let mut file_path = None;
                let mut line_number = None;
                let mut byte_range = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::SourceType => {
                            if source_type.is_some() {
                                return Err(de::Error::duplicate_field("source_type"));
                            }
                            source_type = Some(map.next_value()?);
                        }
                        Field::FilePath => {
                            if file_path.is_some() {
                                return Err(de::Error::duplicate_field("file_path"));
                            }
                            let path_str: String = map.next_value()?;
                            file_path = Some(FilePath::from(path_str));
                        }
                        Field::LineNumber => {
                            if line_number.is_some() {
                                return Err(de::Error::duplicate_field("line_number"));
                            }
                            line_number = Some(map.next_value()?);
                        }
                        Field::ByteRange => {
                            if byte_range.is_some() {
                                return Err(de::Error::duplicate_field("byte_range"));
                            }
                            byte_range = Some(map.next_value()?);
                        }
                    }
                }
                let source_type =
                    source_type.ok_or_else(|| de::Error::missing_field("source_type"))?;
                let file_path = file_path.ok_or_else(|| de::Error::missing_field("file_path"))?;
                let line_number =
                    line_number.ok_or_else(|| de::Error::missing_field("line_number"))?;
                Ok(DocumentSource {
                    source_type,
                    file_path,
                    line_number,
                    byte_range,
                })
            }
        }

        deserializer.deserialize_struct(
            "DocumentSource",
            &["source_type", "file_path", "line_number", "byte_range"],
            DocumentSourceVisitor,
        )
    }
}

/* 📖 # Why separate DocumentSource from Document?

Separating source location information from document content provides:

1. **Clear separation of concerns**: Content (title, markdown) vs. provenance (file, line)
2. **Easier testing**: Can create test documents without real file paths
3. **Flexibility**: Can change source representation without affecting document core
4. **Query support**: Can filter/group by source properties independently

This pattern follows the single responsibility principle - Document handles content,
DocumentSource handles location metadata.
*/

impl DocumentSource {
    /// Create a new DocumentSource.
    ///
    /// # Arguments
    /// * `source_type` - Whether this is a code comment or markdown file
    /// * `file_path` - Relative path to the file
    /// * `line_number` - Line number where the document appears (1-indexed for code, 1 for files)
    ///
    /// For a DocumentSource with byte range, use `with_byte_range()` after construction.
    pub fn new(source_type: SourceType, file_path: FilePath, line_number: usize) -> Self {
        Self {
            source_type,
            file_path,
            line_number,
            byte_range: None,
        }
    }

    /// Set the byte range for this source.
    ///
    /// The byte range indicates where the extracted documentation content is located
    /// within the source file, excluding any document markers.
    ///
    /// # Examples
    /// ```
    /// use hyperlit_base::FilePath;
    /// use hyperlit_engine::{DocumentSource, SourceType, ByteRange};
    ///
    /// let source = DocumentSource::new(
    ///     SourceType::CodeComment,
    ///     FilePath::from("src/main.rs"),
    ///     42,
    /// ).with_byte_range(ByteRange::new(100, 250));
    ///
    /// assert_eq!(source.byte_range(), Some(&ByteRange::new(100, 250)));
    /// ```
    pub fn with_byte_range(mut self, byte_range: ByteRange) -> Self {
        self.byte_range = Some(byte_range);
        self
    }

    /// Returns the type of source.
    pub fn source_type(&self) -> SourceType {
        self.source_type
    }

    /// Returns the file path.
    pub fn file_path(&self) -> &FilePath {
        &self.file_path
    }

    /// Returns the line number (1-indexed for code, 1 for markdown files).
    pub fn line_number(&self) -> usize {
        self.line_number
    }

    /// Returns the byte range if present.
    ///
    /// The byte range indicates where the extracted documentation content is located
    /// within the source file, excluding any document markers.
    pub fn byte_range(&self) -> Option<&ByteRange> {
        self.byte_range.as_ref()
    }

    /// Returns true if this document comes from a code comment.
    pub fn is_code_comment(&self) -> bool {
        matches!(self.source_type, SourceType::CodeComment)
    }

    /// Returns true if this document comes from a markdown file.
    pub fn is_markdown_file(&self) -> bool {
        matches!(self.source_type, SourceType::MarkdownFile)
    }
}

/// Type of document source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceType {
    /// Document extracted from a code comment with a 📖 marker.
    CodeComment,
    /// Document from a standalone markdown file.
    MarkdownFile,
}

/* 📖 # Why track byte ranges for extracted content?

When a document is extracted from a code comment, we store the byte range of the
actual content (excluding the document marker). This allows:

1. **Source verification**: Validate extracted content matches source bytes
2. **Update tracking**: When source changes, check if affected byte range changed
3. **Link generation**: Create precise links to the documentation in source
4. **Debugging**: Correlate extracted content back to exact source locations

For markdown files, the byte range typically covers the entire file. For code comments,
it excludes the `/* 📖 # ... */` markers and includes only the actual documentation content.
*/

/// Byte range indicating where content was extracted from.
///
/// Represents the start and end byte positions of the extracted documentation content
/// within the source file. The range excludes any document markers (e.g., `/* 📖 #`)
/// and includes only the actual documentation content.
///
/// # Examples
///
/// For a code comment:
/// ```text
/// /* 📖 # Why use Arc?
/// Arc provides thread-safe...
/// */
/// ```
/// The byte range would start after the `/* 📖 # Why use Arc?` marker
/// and end before the closing `*/`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ByteRange {
    /// Start byte position (inclusive)
    start: usize,
    /// End byte position (exclusive)
    end: usize,
}

impl ByteRange {
    /// Create a new byte range.
    ///
    /// # Arguments
    /// * `start` - Start byte position (inclusive)
    /// * `end` - End byte position (exclusive)
    ///
    /// # Examples
    /// ```
    /// use hyperlit_engine::ByteRange;
    ///
    /// let range = ByteRange::new(10, 50);
    /// assert_eq!(range.start(), 10);
    /// assert_eq!(range.end(), 50);
    /// assert_eq!(range.len(), 40);
    /// ```
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Get the start byte position (inclusive).
    pub fn start(&self) -> usize {
        self.start
    }

    /// Get the end byte position (exclusive).
    pub fn end(&self) -> usize {
        self.end
    }

    /// Get the length of the byte range.
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Returns true if the range is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Metadata extracted from a document (simple key-value pairs).
///
/// Stores additional metadata like author, date, tags, etc. Metadata is stored as simple
/// string key-value pairs for flexibility. Future versions may add YAML frontmatter parsing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentMetadata {
    fields: HashMap<String, String>,
}

impl DocumentMetadata {
    /// Create metadata from a HashMap of fields.
    pub fn new(fields: HashMap<String, String>) -> Self {
        Self { fields }
    }

    /// Create empty metadata.
    pub fn empty() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

    /// Get a metadata field by key.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.fields.get(key).map(|s| s.as_str())
    }

    /// Iterate over metadata fields.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.fields.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    /// Returns true if metadata has no fields.
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}

impl Document {
    /// Create a new document with a generated ID based on the title.
    ///
    /// The document ID is generated by slugifying the title. If the slug already exists
    /// in `existing_ids`, a collision number is appended.
    ///
    /// # Arguments
    /// * `title` - The document title
    /// * `content` - The markdown content
    /// * `source` - Source location information
    /// * `metadata` - Optional metadata (author, date, tags, etc.)
    /// * `existing_ids` - Set of already-used IDs for collision detection
    ///
    /// # Examples
    /// ```
    /// use std::collections::HashSet;
    /// use hyperlit_base::FilePath;
    /// use hyperlit_engine::{Document, DocumentSource, SourceType};
    ///
    /// let source = DocumentSource::new(
    ///     SourceType::CodeComment,
    ///     FilePath::from("src/main.rs"),
    ///     42,
    /// );
    ///
    /// let mut existing = HashSet::new();
    /// let doc = Document::new(
    ///     "Why use Arc?".to_string(),
    ///     "Arc provides thread-safe...".to_string(),
    ///     source,
    ///     None,
    ///     &existing,
    /// );
    ///
    /// assert_eq!(doc.id().as_str(), "why-use-arc");
    /// assert_eq!(doc.title(), "Why use Arc?");
    /// ```
    pub fn new(
        title: String,
        content: String,
        source: DocumentSource,
        metadata: Option<DocumentMetadata>,
        existing_ids: &std::collections::HashSet<String>,
    ) -> Self {
        let id = DocumentId::from_title(&title, existing_ids);
        Self {
            id,
            title,
            content,
            source,
            metadata,
        }
    }

    /// Returns the document ID.
    pub fn id(&self) -> &DocumentId {
        &self.id
    }

    /// Returns the document title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the markdown content.
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Returns source location and type information.
    pub fn source(&self) -> &DocumentSource {
        &self.source
    }

    /// Returns metadata if present.
    pub fn metadata(&self) -> Option<&DocumentMetadata> {
        self.metadata.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_document_id_from_title() {
        let existing = HashSet::new();

        let id1 = DocumentId::from_title("Why Use Arc?", &existing);
        assert_eq!(id1.as_str(), "why-use-arc");

        let id2 = DocumentId::from_title("Testing 123", &existing);
        assert_eq!(id2.as_str(), "testing-123");
    }

    #[test]
    fn test_document_id_collision_handling() {
        let mut existing = HashSet::new();

        // First document with this title
        let id1 = DocumentId::from_title("Why Use Arc?", &existing);
        assert_eq!(id1.as_str(), "why-use-arc");
        existing.insert(id1.as_str().to_string());

        // Second document with same title - should get -1 suffix
        let id2 = DocumentId::from_title("Why Use Arc?", &existing);
        assert_eq!(id2.as_str(), "why-use-arc-1");
        existing.insert(id2.as_str().to_string());

        // Third document with same title - should get -2 suffix
        let id3 = DocumentId::from_title("Why Use Arc?", &existing);
        assert_eq!(id3.as_str(), "why-use-arc-2");
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Why Use Arc?"), "why-use-arc");
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("Test   Multiple   Spaces"), "test-multiple-spaces");
        assert_eq!(slugify("Special@#$Chars"), "specialchars");
        assert_eq!(slugify("CamelCase"), "camelcase");
    }

    #[test]
    fn test_document_creation() {
        let source = DocumentSource::new(SourceType::CodeComment, FilePath::from("test.rs"), 10);

        let existing = HashSet::new();
        let doc = Document::new(
            "Test Document".to_string(),
            "Test content".to_string(),
            source,
            None,
            &existing,
        );

        assert_eq!(doc.title(), "Test Document");
        assert_eq!(doc.content(), "Test content");
        assert_eq!(doc.source().line_number(), 10);
        assert_eq!(doc.id().as_str(), "test-document");
        assert!(doc.metadata().is_none());
    }

    #[test]
    fn test_source_type_checks() {
        let code_source =
            DocumentSource::new(SourceType::CodeComment, FilePath::from("test.rs"), 10);
        assert!(code_source.is_code_comment());
        assert!(!code_source.is_markdown_file());

        let md_source = DocumentSource::new(SourceType::MarkdownFile, FilePath::from("doc.md"), 1);
        assert!(!md_source.is_code_comment());
        assert!(md_source.is_markdown_file());
    }

    #[test]
    fn test_metadata_operations() {
        let mut fields = HashMap::new();
        fields.insert("author".to_string(), "Fred Brooks".to_string());
        fields.insert("date".to_string(), "2025-11-14".to_string());

        let metadata = DocumentMetadata::new(fields);

        assert_eq!(metadata.get("author"), Some("Fred Brooks"));
        assert_eq!(metadata.get("date"), Some("2025-11-14"));
        assert_eq!(metadata.get("nonexistent"), None);
        assert!(!metadata.is_empty());
    }

    #[test]
    fn test_metadata_empty() {
        let metadata = DocumentMetadata::empty();
        assert!(metadata.is_empty());
        assert_eq!(metadata.get("any"), None);
    }

    #[test]
    fn test_document_equality() {
        let source = DocumentSource::new(SourceType::CodeComment, FilePath::from("test.rs"), 10);

        let existing = HashSet::new();
        let doc1 = Document::new(
            "Title".to_string(),
            "Content".to_string(),
            source.clone(),
            None,
            &existing,
        );

        let doc2 = Document::new(
            "Title".to_string(),
            "Content".to_string(),
            source,
            None,
            &existing,
        );

        assert_eq!(doc1, doc2);
    }

    #[test]
    fn test_document_id_collision_in_creation() {
        let mut existing = HashSet::new();
        let source1 = DocumentSource::new(SourceType::CodeComment, FilePath::from("test1.rs"), 10);
        let source2 = DocumentSource::new(SourceType::CodeComment, FilePath::from("test2.rs"), 20);

        // Create first document
        let doc1 = Document::new(
            "Same Title".to_string(),
            "Content 1".to_string(),
            source1,
            None,
            &existing,
        );
        assert_eq!(doc1.id().as_str(), "same-title");
        existing.insert(doc1.id().as_str().to_string());

        // Create second document with same title
        let doc2 = Document::new(
            "Same Title".to_string(),
            "Content 2".to_string(),
            source2,
            None,
            &existing,
        );
        assert_eq!(doc2.id().as_str(), "same-title-1");
    }

    #[test]
    fn test_document_with_metadata() {
        let source = DocumentSource::new(
            SourceType::MarkdownFile,
            FilePath::from("docs/design.md"),
            1,
        );

        let mut fields = HashMap::new();
        fields.insert("author".to_string(), "Alice".to_string());
        let metadata = DocumentMetadata::new(fields);

        let existing = HashSet::new();
        let doc = Document::new(
            "Design Decision".to_string(),
            "We chose this because...".to_string(),
            source,
            Some(metadata),
            &existing,
        );

        assert!(doc.metadata().is_some());
        assert_eq!(doc.metadata().unwrap().get("author"), Some("Alice"));
    }

    #[test]
    fn test_document_id_display() {
        let id = DocumentId::from_title("Test Title", &HashSet::new());
        assert_eq!(id.to_string(), "test-title");
    }

    #[test]
    fn test_source_display() {
        let source =
            DocumentSource::new(SourceType::CodeComment, FilePath::from("src/main.rs"), 42);
        assert_eq!(source.file_path().to_string(), "src/main.rs");
        assert_eq!(source.line_number(), 42);
    }

    #[test]
    fn test_slugify_edge_cases() {
        assert_eq!(slugify(""), "");
        assert_eq!(slugify("   "), "");
        assert_eq!(slugify("___"), "");
        assert_eq!(slugify("a-b-c"), "a-b-c");
        assert_eq!(slugify("a--b"), "a-b");
        assert_eq!(slugify("---"), "");
    }

    #[test]
    fn test_metadata_iteration() {
        let mut fields = HashMap::new();
        fields.insert("key1".to_string(), "value1".to_string());
        fields.insert("key2".to_string(), "value2".to_string());

        let metadata = DocumentMetadata::new(fields);
        let pairs: Vec<_> = metadata.iter().collect();

        assert_eq!(pairs.len(), 2);
        // Order doesn't matter for HashMaps, just verify both are present
        let has_key1 = pairs.iter().any(|(k, _)| *k == "key1");
        let has_key2 = pairs.iter().any(|(k, _)| *k == "key2");
        assert!(has_key1);
        assert!(has_key2);
    }

    #[test]
    fn test_byte_range_creation() {
        let range = ByteRange::new(10, 50);
        assert_eq!(range.start(), 10);
        assert_eq!(range.end(), 50);
        assert_eq!(range.len(), 40);
        assert!(!range.is_empty());
    }

    #[test]
    fn test_byte_range_empty() {
        let range = ByteRange::new(10, 10);
        assert_eq!(range.len(), 0);
        assert!(range.is_empty());
    }

    #[test]
    fn test_byte_range_equality() {
        let range1 = ByteRange::new(10, 50);
        let range2 = ByteRange::new(10, 50);
        let range3 = ByteRange::new(10, 60);

        assert_eq!(range1, range2);
        assert_ne!(range1, range3);
    }

    #[test]
    fn test_byte_range_saturating_subtraction() {
        // Test that len() uses saturating subtraction (end >= start is guaranteed)
        let range = ByteRange::new(100, 50);
        assert_eq!(range.len(), 0); // 50.saturating_sub(100) = 0
    }

    #[test]
    fn test_document_source_without_byte_range() {
        let source =
            DocumentSource::new(SourceType::CodeComment, FilePath::from("src/main.rs"), 42);

        assert_eq!(source.source_type(), SourceType::CodeComment);
        assert_eq!(source.file_path().to_string(), "src/main.rs");
        assert_eq!(source.line_number(), 42);
        assert!(source.byte_range().is_none());
    }

    #[test]
    fn test_document_source_with_byte_range() {
        let range = ByteRange::new(100, 250);
        let source =
            DocumentSource::new(SourceType::CodeComment, FilePath::from("src/main.rs"), 42)
                .with_byte_range(range);

        assert_eq!(source.source_type(), SourceType::CodeComment);
        assert_eq!(source.file_path().to_string(), "src/main.rs");
        assert_eq!(source.line_number(), 42);
        assert_eq!(source.byte_range(), Some(&ByteRange::new(100, 250)));
    }

    #[test]
    fn test_document_source_byte_range_fluent() {
        let source = DocumentSource::new(
            SourceType::MarkdownFile,
            FilePath::from("docs/design.md"),
            1,
        )
        .with_byte_range(ByteRange::new(0, 1000));

        assert!(source.is_markdown_file());
        assert_eq!(source.byte_range().unwrap().len(), 1000);
    }

    #[test]
    fn test_document_with_source_byte_range() {
        let range = ByteRange::new(50, 200);
        let source = DocumentSource::new(SourceType::CodeComment, FilePath::from("test.rs"), 10)
            .with_byte_range(range);

        let existing = HashSet::new();
        let doc = Document::new(
            "Test Doc".to_string(),
            "Content".to_string(),
            source,
            None,
            &existing,
        );

        assert_eq!(doc.source().byte_range().unwrap().start(), 50);
        assert_eq!(doc.source().byte_range().unwrap().end(), 200);
    }

    #[test]
    fn test_byte_range_serialization() {
        let range = ByteRange::new(100, 250);
        let json = serde_json::to_string(&range).unwrap();
        let deserialized: ByteRange = serde_json::from_str(&json).unwrap();

        assert_eq!(range, deserialized);
    }

    #[test]
    fn test_document_source_serialization_without_byte_range() {
        let source =
            DocumentSource::new(SourceType::CodeComment, FilePath::from("src/main.rs"), 42);

        let json = serde_json::to_string(&source).unwrap();
        assert!(!json.contains("byte_range")); // Should be skipped when None

        let deserialized: DocumentSource = serde_json::from_str(&json).unwrap();
        assert_eq!(source, deserialized);
    }

    #[test]
    fn test_document_source_serialization_with_byte_range() {
        let source =
            DocumentSource::new(SourceType::CodeComment, FilePath::from("src/main.rs"), 42)
                .with_byte_range(ByteRange::new(100, 250));

        let json = serde_json::to_string(&source).unwrap();
        assert!(json.contains("byte_range")); // Should be included when Some

        let deserialized: DocumentSource = serde_json::from_str(&json).unwrap();
        assert_eq!(source, deserialized);
        assert_eq!(deserialized.byte_range(), Some(&ByteRange::new(100, 250)));
    }

    #[test]
    fn test_byte_range_copy_semantics() {
        let range1 = ByteRange::new(10, 50);
        let range2 = range1; // Copy trait

        assert_eq!(range1, range2);
        assert_eq!(range1.start(), range2.start());
    }
}
