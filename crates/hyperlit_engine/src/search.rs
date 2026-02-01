/* 📖 # Why implement a simple in-memory search?

For the initial search implementation, we use a simple in-memory text search
rather than a full-text search engine like Tantivy. This provides several benefits:

1. **Simplicity**: No external dependencies or index management
2. **Speed**: For small to medium documentation sets, linear search is fast enough
3. **Predictability**: Simple substring matching is easy to understand and debug
4. **Zero setup**: Works immediately without index building or configuration

The trade-off is that this won't scale to very large documentation sets (10k+ documents),
but for typical projects it's sufficient. We can always upgrade to Tantivy later.

Search ranking prioritizes:
1. Title matches (higher priority)
2. Content matches (lower priority)
Both are case-insensitive substring searches.
*/

use crate::document::Document;

/// A search result containing the document and match information.
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// The matching document
    pub document: Document,
    /// The relevance score (higher = better match)
    pub score: usize,
    /// Where the match was found
    pub match_type: MatchType,
}

/// Type of search match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatchType {
    /// Match found in the document title
    Title,
    /// Match found in the document content
    Content,
    /// Match found in both title and content
    Both,
}

/// Simple in-memory search engine for documents.
///
/// Performs case-insensitive substring matching on document titles and content.
/// Results are ranked with title matches prioritized over content matches.
#[derive(Debug, Clone)]
pub struct SimpleSearch;

impl SimpleSearch {
    /// Create a new simple search instance.
    pub fn new() -> Self {
        Self
    }

    /// Search documents for the given query.
    ///
    /// Performs case-insensitive substring matching. Results are ranked by:
    /// 1. Title matches (score = 100)
    /// 2. Content matches (score = 10)
    /// 3. Both title and content (score = 110)
    ///
    /// # Arguments
    /// * `documents` - Iterator of documents to search
    /// * `query` - The search query string
    ///
    /// # Returns
    /// A vector of search results sorted by score (highest first)
    pub fn search<'a>(
        &self,
        documents: impl Iterator<Item = &'a Document>,
        query: &str,
    ) -> Vec<SearchResult> {
        if query.is_empty() {
            return Vec::new();
        }

        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for doc in documents {
            let title_lower = doc.title().to_lowercase();
            let content_lower = doc.content().to_lowercase();

            let title_match = title_lower.contains(&query_lower);
            let content_match = content_lower.contains(&query_lower);

            if title_match || content_match {
                let (score, match_type) = match (title_match, content_match) {
                    (true, true) => (110, MatchType::Both),
                    (true, false) => (100, MatchType::Title),
                    (false, true) => (10, MatchType::Content),
                    (false, false) => unreachable!(),
                };

                results.push(SearchResult {
                    document: doc.clone(),
                    score,
                    match_type,
                });
            }
        }

        // Sort by score descending (highest score first)
        results.sort_by(|a, b| b.score.cmp(&a.score));

        results
    }
}

impl Default for SimpleSearch {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{DocumentSource, SourceType};
    use hyperlit_base::FilePath;
    use std::collections::HashSet;

    fn create_test_doc(title: &str, content: &str) -> Document {
        let source = DocumentSource::new(SourceType::CodeComment, FilePath::from("test.rs"), 1);
        Document::new(
            title.to_string(),
            content.to_string(),
            source,
            None,
            &HashSet::new(),
        )
    }

    #[test]
    fn test_empty_query_returns_no_results() {
        let search = SimpleSearch::new();
        let docs = vec![create_test_doc("Test", "Content")];
        let results = search.search(docs.iter(), "");
        assert!(results.is_empty());
    }

    #[test]
    fn test_title_match() {
        let search = SimpleSearch::new();
        let docs = vec![create_test_doc("Why Use Arc?", "Some content about Mutex")];
        let results = search.search(docs.iter(), "arc");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].match_type, MatchType::Title);
        assert_eq!(results[0].score, 100);
    }

    #[test]
    fn test_content_match() {
        let search = SimpleSearch::new();
        let docs = vec![create_test_doc(
            "Some Title",
            "This is about Mutex and locks",
        )];
        let results = search.search(docs.iter(), "mutex");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].match_type, MatchType::Content);
        assert_eq!(results[0].score, 10);
    }

    #[test]
    fn test_title_priority_over_content() {
        let search = SimpleSearch::new();
        let docs = vec![
            create_test_doc("Mutex Guide", "Content about threading"),
            create_test_doc("Threading", "Content about Mutex usage"),
        ];
        let results = search.search(docs.iter(), "mutex");

        assert_eq!(results.len(), 2);
        // Title match should come first
        assert_eq!(results[0].match_type, MatchType::Title);
        assert_eq!(results[1].match_type, MatchType::Content);
    }

    #[test]
    fn test_both_match() {
        let search = SimpleSearch::new();
        let docs = vec![create_test_doc(
            "Arc Pattern",
            "This explains the Arc pattern",
        )];
        let results = search.search(docs.iter(), "arc");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].match_type, MatchType::Both);
        assert_eq!(results[0].score, 110);
    }

    #[test]
    fn test_case_insensitive() {
        let search = SimpleSearch::new();
        let docs = vec![create_test_doc("UPPERCASE", "MiXeD CaSe CoNtEnT")];

        let results = search.search(docs.iter(), "upper");
        assert_eq!(results.len(), 1);

        let results = search.search(docs.iter(), "mixed");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_no_matches() {
        let search = SimpleSearch::new();
        let docs = vec![create_test_doc("Rust Guide", "Content about ownership")];
        let results = search.search(docs.iter(), "python");

        assert!(results.is_empty());
    }
}
