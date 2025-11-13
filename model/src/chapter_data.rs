use serde::Serialize;

/// Data for a chapter, used to serve chapters as markdown
#[derive(Serialize)]
pub struct ChapterData {
    markdown: String,
}

impl ChapterData {
    pub fn new(markdown: String) -> ChapterData {
        ChapterData { markdown }
    }

    pub fn markdown(&self) -> &String {
        &self.markdown
    }
}
