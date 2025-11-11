use hyperlit_base::FilePath;
use serde::Serialize;

/// The structure of a book
#[derive(Debug, Clone, Serialize)]
pub struct BookStructure {
    pub title: String,
    pub chapters: Vec<ChapterStructure>,
}

impl BookStructure {
    pub fn new(title: impl Into<String>) -> BookStructure {
        BookStructure {
            title: title.into(),
            chapters: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ChapterStructure {
    pub label: String,
    pub tags: Vec<String>,
    pub file: Option<FilePath>,
    pub chapters: Vec<ChapterStructure>,
}

impl ChapterStructure {
    pub fn new(label: impl Into<String>) -> ChapterStructure {
        ChapterStructure {
            label: label.into(),
            file: None,
            tags: Vec::new(),
            chapters: Vec::new(),
        }
    }
}
