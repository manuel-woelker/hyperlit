use serde::Serialize;

/// Data for a chapter, used to serve chapters as markdown
#[derive(Serialize)]
pub struct ChapterData {
    pub markdown: String,
    pub edit_url: Option<String>,
}

impl ChapterData {
    pub fn new(markdown: String) -> ChapterData {
        ChapterData {
            markdown,
            edit_url: None,
        }
    }

    pub fn markdown(&self) -> &String {
        &self.markdown
    }
}
