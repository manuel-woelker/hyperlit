use hyperlit_base::result::HyperlitResult;
use hyperlit_base::shared_string::SharedString;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug)]
pub struct SearchDocument {
    pub id: SharedString,
    pub title: SharedString,
    pub text: SharedString,
}

impl SearchDocument {
    pub fn new(
        id: impl Into<SharedString>,
        title: impl Into<SharedString>,
        text: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            text: text.into(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub search_string: SharedString,
}

impl SearchRequest {
    pub fn new(search_string: impl Into<SharedString>) -> Self {
        Self {
            search_string: search_string.into(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub hits: Vec<SearchHit>,
}

#[derive(Debug, Serialize)]
pub struct SearchHit {
    pub document_id: SharedString,
    pub document_title: SharedString,
}

pub trait SearchService {
    fn index_files(&mut self, documents: Vec<SearchDocument>) -> HyperlitResult<()>;
    fn search(&self, search_request: SearchRequest) -> HyperlitResult<SearchResponse>;
}
