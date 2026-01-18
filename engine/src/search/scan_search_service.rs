use crate::search::search_service::{
    SearchDocument, SearchHit, SearchRequest, SearchResponse, SearchService,
};
use hyperlit_base::result::HyperlitResult;

pub struct ScanSearchService {
    documents: Vec<SearchDocument>,
}

impl ScanSearchService {
    pub fn new() -> HyperlitResult<Self> {
        Ok(Self {
            documents: Vec::new(),
        })
    }
}

impl SearchService for ScanSearchService {
    fn index_files(&mut self, mut documents: Vec<SearchDocument>) -> HyperlitResult<()> {
        for document in &mut documents {
            document.text = document.text.to_lowercase().into();
        }
        self.documents.extend(documents);
        Ok(())
    }

    fn search(&self, search_request: SearchRequest) -> HyperlitResult<SearchResponse> {
        let search_string = search_request.search_string.as_str().to_lowercase();
        let mut hits = vec![];
        for document in &self.documents {
            if document.text.contains(&search_string) {
                hits.push(SearchHit {
                    document_id: document.id.clone(),
                    document_title: document.title.clone(),
                });
            }
        }
        Ok(SearchResponse { hits })
    }
}

#[cfg(test)]
mod tests {
    use crate::search::scan_search_service::ScanSearchService;
    use crate::search::search_service::SearchService;
    use crate::search::search_service::{SearchDocument, SearchRequest};
    use expect_test::expect;
    use hyperlit_base::result::HyperlitResult;

    #[test]
    fn test_index() -> HyperlitResult<()> {
        let mut search_service = ScanSearchService::new()?;
        search_service.index_files(vec![
            SearchDocument::new("one", "Title 1", "this is the first document"),
            SearchDocument::new("two", "Title 2", "this is the second document"),
        ])?;
        expect![[r#"
            SearchResponse {
                hits: [
                    SearchHit {
                        document_id: "one",
                        document_title: "Title 1",
                    },
                ],
            }
        "#]]
        .assert_debug_eq(&search_service.search(SearchRequest::new("first"))?);
        expect![[r#"
            SearchResponse {
                hits: [
                    SearchHit {
                        document_id: "one",
                        document_title: "Title 1",
                    },
                    SearchHit {
                        document_id: "two",
                        document_title: "Title 2",
                    },
                ],
            }
        "#]]
        .assert_debug_eq(&search_service.search(SearchRequest::new("DOCUMENT"))?);
        expect![[r#"
            SearchResponse {
                hits: [],
            }
        "#]]
        .assert_debug_eq(&search_service.search(SearchRequest::new("three"))?);
        expect![[r#"
            SearchResponse {
                hits: [],
            }
        "#]]
        .assert_debug_eq(&search_service.search(SearchRequest::new("title"))?);
        Ok(())
    }
}
