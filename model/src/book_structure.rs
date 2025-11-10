use serde::Serialize;

/// The structure of a book
#[derive(Debug, Clone, Serialize)]
pub struct BookStructure {
    pub title: String,
    pub chapters: Vec<ChapterStructure>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChapterStructure {
    pub label: String,
    pub tags: Vec<String>,
    pub chapters: Vec<ChapterStructure>,
}

impl ChapterStructure {
    pub fn new(label: impl Into<String>) -> ChapterStructure {
        ChapterStructure {
            label: label.into(),
            tags: Vec::new(),
            chapters: Vec::new(),
        }
    }
}
