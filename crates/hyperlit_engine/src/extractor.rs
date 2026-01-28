/* 📖 # Why have a dedicated extraction module?

The extractor provides the critical functionality to convert raw files into Documents.
This separation from the scanner (which just finds files) allows:

1. **Independent testing**: Can test extraction logic without filesystem operations
2. **Composition**: Scanning and extraction can be combined in different ways
3. **Error recovery**: Fail-tolerant extraction means one bad file doesn't stop processing
4. **Modularity**: Extraction logic can be extended later with new file types

The extractor handles markdown files specifically, parsing YAML frontmatter,
extracting titles, and creating Document instances with proper metadata.
*/

use std::collections::HashSet;
use tracing::{instrument, warn};

use hyperlit_base::{FilePath, HyperlitError, HyperlitResult, PalHandle};

use crate::{ByteRange, Document, DocumentMetadata, DocumentSource, SourceType};

/// Results from extracting documents from markdown files.
///
/// This struct enables fail-tolerant extraction: if some files fail to extract,
/// the operation continues and reports both successful documents and encountered errors.
#[derive(Debug)]
pub struct ExtractionResult {
    /// Successfully extracted documents
    pub documents: Vec<Document>,
    /// Errors encountered during extraction (non-fatal)
    pub errors: Vec<ExtractionError>,
}

/// Error encountered while extracting a specific file.
#[derive(Debug)]
pub struct ExtractionError {
    /// File path that failed to extract
    pub file_path: FilePath,
    /// The error that occurred
    pub error: Box<HyperlitError>,
}

/// Extract documents from a list of file paths.
///
/// This function reads each file, determines its type, and extracts documentation.
/// For markdown files, it parses YAML frontmatter (if present) and extracts the title.
///
/// Extraction is fail-tolerant: if a file fails to extract, the error is collected
/// and extraction continues with remaining files.
#[instrument(skip(pal, files), fields(file_count = files.len()))]
pub fn extract_documents(pal: &PalHandle, files: &[FilePath]) -> HyperlitResult<ExtractionResult> {
    let mut documents = Vec::new();
    let mut errors = Vec::new();
    let mut existing_ids = HashSet::new();

    for file_path in files {
        match extract_markdown_document(pal, file_path, &existing_ids) {
            Ok(doc) => {
                existing_ids.insert(doc.id().as_str().to_string());
                documents.push(doc);
            }
            Err(e) => {
                warn!("failed to extract {}: {}", file_path, e);
                errors.push(ExtractionError {
                    file_path: file_path.clone(),
                    error: e,
                });
            }
        }
    }

    Ok(ExtractionResult { documents, errors })
}

/// Extract a single markdown document from a file.
///
/// This function:
/// 1. Reads the file content
/// 2. Parses YAML frontmatter (if present)
/// 3. Extracts the title (from frontmatter or first # heading)
/// 4. Creates a Document with appropriate metadata
fn extract_markdown_document(
    pal: &PalHandle,
    file_path: &FilePath,
    existing_ids: &HashSet<String>,
) -> HyperlitResult<Document> {
    // Read file content
    let content = pal.read_file_to_string(file_path)?;

    // Parse frontmatter
    let (metadata, content_without_frontmatter, frontmatter_end_byte) =
        parse_frontmatter(&content)?;

    // Extract title
    let title = extract_title(content_without_frontmatter, &metadata, file_path)?;

    // Calculate byte range (excluding frontmatter)
    let byte_range = ByteRange::new(frontmatter_end_byte, content.len());

    // Create document source
    let source = DocumentSource::new(SourceType::MarkdownFile, file_path.clone(), 1)
        .with_byte_range(byte_range);

    // Create document
    let doc = Document::new(
        title,
        content_without_frontmatter.to_string(),
        source,
        metadata,
        existing_ids,
    );

    Ok(doc)
}

/// Parse YAML frontmatter from markdown content.
///
/// Returns (metadata, content_without_frontmatter, frontmatter_end_byte).
/// If no frontmatter exists, returns empty metadata and full content.
fn parse_frontmatter(content: &str) -> HyperlitResult<(Option<DocumentMetadata>, &str, usize)> {
    // Check if content starts with YAML frontmatter (---)
    if !content.starts_with("---") {
        return Ok((None, content, 0));
    }

    // Find the closing --- delimiter
    let lines: Vec<&str> = content.lines().collect();

    // Must have at least opening ---, content line, closing ---
    if lines.len() < 2 {
        return Ok((None, content, 0));
    }

    // Look for closing --- on second or later line
    let mut closing_index = None;
    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.trim() == "---" {
            closing_index = Some(i);
            break;
        }
    }

    let closing_idx = match closing_index {
        Some(idx) => idx,
        None => {
            // No closing --- found, treat as no frontmatter
            return Ok((None, content, 0));
        }
    };

    // Extract frontmatter YAML content (between the --- delimiters)
    let frontmatter_str = lines[1..closing_idx].join("\n");

    // Parse YAML
    let metadata = if frontmatter_str.trim().is_empty() {
        None
    } else {
        match serde_yaml::from_str::<serde_yaml::Value>(&frontmatter_str) {
            Ok(value) => {
                if let serde_yaml::Value::Mapping(mapping) = value {
                    let mut fields = std::collections::HashMap::new();

                    for (key, val) in mapping {
                        if let serde_yaml::Value::String(k) = key {
                            let v = match val {
                                serde_yaml::Value::String(s) => s,
                                serde_yaml::Value::Number(n) => n.to_string(),
                                serde_yaml::Value::Bool(b) => b.to_string(),
                                _ => continue,
                            };
                            fields.insert(k, v);
                        }
                    }

                    if fields.is_empty() {
                        None
                    } else {
                        Some(DocumentMetadata::new(fields))
                    }
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    };

    // Calculate frontmatter end byte position
    // We need to account for newlines between lines
    let mut frontmatter_end = 0;
    for (i, line) in lines.iter().enumerate().take(closing_idx + 1) {
        frontmatter_end += line.len();
        if i < closing_idx {
            frontmatter_end += 1; // newline character
        }
    }

    // Account for newline after closing ---
    if closing_idx + 1 < lines.len() {
        frontmatter_end += 1;
    }

    // Get content after frontmatter
    let content_without_frontmatter = if closing_idx + 1 < lines.len() {
        lines[closing_idx + 1..].join("\n")
    } else {
        String::new()
    };

    Ok((
        metadata,
        Box::leak(content_without_frontmatter.into_boxed_str()),
        frontmatter_end,
    ))
}

/// Extract title from markdown content or metadata.
///
/// Priority:
/// 1. Metadata "title" field
/// 2. First # heading in content
/// 3. Filename (without extension)
fn extract_title(
    content: &str,
    metadata: &Option<DocumentMetadata>,
    file_path: &FilePath,
) -> HyperlitResult<String> {
    // Try metadata first
    if let Some(meta) = metadata
        && let Some(title) = meta.get("title")
    {
        return Ok(title.to_string());
    }

    // Try first # heading
    if let Some(title) = extract_first_heading(content) {
        return Ok(title);
    }

    // Fallback to filename
    let filename = file_path
        .as_path()
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled");

    Ok(filename.to_string())
}

/// Extract the first # heading from markdown content.
///
/// Returns the heading text without the # prefix.
/// Handles cases like "# Title #tag" by removing trailing tags.
fn extract_first_heading(content: &str) -> Option<String> {
    // Use regex to find first H1 heading: ^#\s+(.+)$
    let re = regex::Regex::new(r"^#\s+(.+)$").ok()?;

    for line in content.lines() {
        if let Some(captures) = re.captures(line)
            && let Some(heading) = captures.get(1)
        {
            let title = heading.as_str();

            // Remove trailing # tags (e.g., "# Amdahl's Law #law" → "Amdahl's Law")
            let title = title.split('#').next().unwrap_or(title);

            return Some(title.trim().to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyperlit_base::pal::MockPal;

    #[test]
    fn test_extract_simple_markdown() {
        let mock_pal = MockPal::new();
        let content = "# Test Title\n\nThis is content.";
        mock_pal.add_file(FilePath::from("test.md"), content.as_bytes().to_vec());

        let pal = hyperlit_base::PalHandle::new(mock_pal);
        let files = vec![FilePath::from("test.md")];
        let result = extract_documents(&pal, &files).unwrap();

        assert_eq!(result.documents.len(), 1);
        assert_eq!(result.documents[0].title(), "Test Title");
        assert_eq!(
            result.documents[0].content(),
            "# Test Title\n\nThis is content."
        );
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_extract_markdown_with_frontmatter() {
        let mock_pal = MockPal::new();
        let content = "---\ntitle: \"Brooks' Law\"\nauthor: \"Fred Brooks\"\ndate: 2025-11-14\n---\n\n# Brooks's Law\n\nContent here.";
        mock_pal.add_file(FilePath::from("brooks.md"), content.as_bytes().to_vec());

        let pal = hyperlit_base::PalHandle::new(mock_pal);
        let files = vec![FilePath::from("brooks.md")];
        let result = extract_documents(&pal, &files).unwrap();

        assert_eq!(result.documents.len(), 1);
        let doc = &result.documents[0];

        // Title from frontmatter, not heading
        assert_eq!(doc.title(), "Brooks' Law");

        // Content excludes frontmatter
        assert!(!doc.content().contains("---"));
        assert!(doc.content().contains("# Brooks's Law"));

        // Metadata extracted
        let meta = doc.metadata().unwrap();
        assert_eq!(meta.get("author"), Some("Fred Brooks"));
        assert_eq!(meta.get("date"), Some("2025-11-14"));
    }

    #[test]
    fn test_extract_markdown_no_heading_uses_filename() {
        let mock_pal = MockPal::new();
        let content = "Just some content without a heading.";
        mock_pal.add_file(FilePath::from("no-title.md"), content.as_bytes().to_vec());

        let pal = hyperlit_base::PalHandle::new(mock_pal);
        let files = vec![FilePath::from("no-title.md")];
        let result = extract_documents(&pal, &files).unwrap();

        assert_eq!(result.documents.len(), 1);
        assert_eq!(result.documents[0].title(), "no-title");
    }

    #[test]
    fn test_extract_heading_with_trailing_tag() {
        let content = "# Amdahl's Law #law\n\nContent here.";
        let title = extract_first_heading(content);
        assert_eq!(title, Some("Amdahl's Law".to_string()));
    }

    #[test]
    fn test_extract_unicode_title() {
        let content = "# Title including umlauts like \"ö\", \"ä\", \"ü\"\n\nContent.";
        let title = extract_first_heading(content);
        assert!(title.unwrap().contains("ö"));
    }

    #[test]
    fn test_extract_multiple_files() {
        let mock_pal = MockPal::new();

        // Add multiple markdown files
        mock_pal.add_file(
            FilePath::from("file1.md"),
            b"# File One\n\nContent 1.".to_vec(),
        );
        mock_pal.add_file(
            FilePath::from("file2.md"),
            b"# File Two\n\nContent 2.".to_vec(),
        );
        mock_pal.add_file(
            FilePath::from("file3.md"),
            b"# File Three\n\nContent 3.".to_vec(),
        );

        let pal = hyperlit_base::PalHandle::new(mock_pal);
        let files = vec![
            FilePath::from("file1.md"),
            FilePath::from("file2.md"),
            FilePath::from("file3.md"),
        ];
        let result = extract_documents(&pal, &files).unwrap();

        assert_eq!(result.documents.len(), 3);
        assert_eq!(result.errors.len(), 0);

        // Check ID collision handling
        let ids: Vec<_> = result.documents.iter().map(|d| d.id().as_str()).collect();
        assert_eq!(ids.len(), 3);
        // All IDs should be unique
        let unique: HashSet<_> = ids.iter().collect();
        assert_eq!(unique.len(), 3);
    }

    #[test]
    fn test_extract_non_utf8_file() {
        let mock_pal = MockPal::new();

        // Add invalid UTF-8 content
        mock_pal.add_file(FilePath::from("invalid.md"), vec![0xFF, 0xFE, 0xFD]);

        let pal = hyperlit_base::PalHandle::new(mock_pal);
        let files = vec![FilePath::from("invalid.md")];
        let result = extract_documents(&pal, &files).unwrap();

        // Should have error, no documents
        assert_eq!(result.documents.len(), 0);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].file_path, FilePath::from("invalid.md"));
    }

    #[test]
    fn test_extract_fail_tolerant() {
        let mock_pal = MockPal::new();

        // Add one valid file, one invalid
        mock_pal.add_file(FilePath::from("valid.md"), b"# Valid\n\nContent.".to_vec());
        mock_pal.add_file(FilePath::from("invalid.md"), vec![0xFF, 0xFE, 0xFD]);

        let pal = hyperlit_base::PalHandle::new(mock_pal);
        let files = vec![FilePath::from("valid.md"), FilePath::from("invalid.md")];
        let result = extract_documents(&pal, &files).unwrap();

        // Should extract valid file despite invalid one
        assert_eq!(result.documents.len(), 1);
        assert_eq!(result.documents[0].title(), "Valid");
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_byte_range_without_frontmatter() {
        let mock_pal = MockPal::new();
        let content = "# Test\n\nContent.";
        mock_pal.add_file(FilePath::from("test.md"), content.as_bytes().to_vec());

        let pal = hyperlit_base::PalHandle::new(mock_pal);
        let files = vec![FilePath::from("test.md")];
        let result = extract_documents(&pal, &files).unwrap();

        let doc = &result.documents[0];
        let range = doc.source().byte_range().unwrap();

        // Should cover entire file
        assert_eq!(range.start(), 0);
        assert_eq!(range.end(), content.len());
    }

    #[test]
    fn test_byte_range_with_frontmatter() {
        let mock_pal = MockPal::new();
        let content = "---\ntitle: Test\n---\n\n# Content\n\nBody.";
        mock_pal.add_file(FilePath::from("test.md"), content.as_bytes().to_vec());

        let pal = hyperlit_base::PalHandle::new(mock_pal);
        let files = vec![FilePath::from("test.md")];
        let result = extract_documents(&pal, &files).unwrap();

        let doc = &result.documents[0];
        let range = doc.source().byte_range().unwrap();

        // Should start after frontmatter
        assert!(range.start() > 0);
        assert_eq!(range.end(), content.len());

        // Content should not include frontmatter
        assert!(!doc.content().contains("---"));
    }

    #[test]
    fn test_parse_frontmatter_empty() {
        let content = "# No frontmatter\n\nContent.";
        let (metadata, content_without, frontmatter_end) = parse_frontmatter(content).unwrap();

        assert!(metadata.is_none());
        assert_eq!(content_without, content);
        assert_eq!(frontmatter_end, 0);
    }

    #[test]
    fn test_parse_frontmatter_with_data() {
        let content = "---\nkey: value\n---\n\nContent.";
        let (metadata, content_without, _) = parse_frontmatter(content).unwrap();

        assert!(metadata.is_some());
        let meta = metadata.unwrap();
        assert_eq!(meta.get("key"), Some("value"));
        assert!(!content_without.contains("---"));
        assert!(content_without.contains("Content."));
    }

    #[test]
    fn test_extract_title_from_metadata() {
        let mut fields = std::collections::HashMap::new();
        fields.insert("title".to_string(), "From Metadata".to_string());
        let metadata = Some(DocumentMetadata::new(fields));

        let title = extract_title(
            "# From Heading\n\nContent",
            &metadata,
            &FilePath::from("file.md"),
        )
        .unwrap();
        assert_eq!(title, "From Metadata");
    }

    #[test]
    fn test_extract_title_from_heading() {
        let metadata = None;
        let title = extract_title(
            "# From Heading\n\nContent",
            &metadata,
            &FilePath::from("file.md"),
        )
        .unwrap();
        assert_eq!(title, "From Heading");
    }

    #[test]
    fn test_extract_title_from_filename() {
        let metadata = None;
        let title = extract_title(
            "No heading here\n\nContent",
            &metadata,
            &FilePath::from("my-file.md"),
        )
        .unwrap();
        assert_eq!(title, "my-file");
    }

    #[test]
    fn test_extract_first_heading_not_found() {
        let content = "Just some text\n\nNo headings here";
        let title = extract_first_heading(content);
        assert!(title.is_none());
    }

    #[test]
    fn test_extract_first_heading_ignores_h2() {
        let content = "## Not H1\n\n# This is H1\n\nContent";
        let title = extract_first_heading(content);
        assert_eq!(title, Some("This is H1".to_string()));
    }

    #[test]
    fn test_extract_document_id_collision() {
        let mock_pal = MockPal::new();

        // Two files with same title
        mock_pal.add_file(
            FilePath::from("file1.md"),
            b"# Same Title\n\nContent 1.".to_vec(),
        );
        mock_pal.add_file(
            FilePath::from("file2.md"),
            b"# Same Title\n\nContent 2.".to_vec(),
        );

        let pal = hyperlit_base::PalHandle::new(mock_pal);
        let files = vec![FilePath::from("file1.md"), FilePath::from("file2.md")];
        let result = extract_documents(&pal, &files).unwrap();

        assert_eq!(result.documents.len(), 2);
        let id1 = result.documents[0].id().as_str();
        let id2 = result.documents[1].id().as_str();

        // IDs should be different
        assert_ne!(id1, id2);
        assert_eq!(id1, "same-title");
        assert_eq!(id2, "same-title-1");
    }
}
