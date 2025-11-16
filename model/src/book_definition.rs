use serde::Deserialize;

/// The definition of a book
#[derive(Debug, Clone, Deserialize)]
pub struct BookDefinition {
    pub title: String,
    pub chapters: Vec<ChapterDefinition>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChapterDefinition {
    pub label: String,
    pub tags: Vec<String>,
    pub directories: Option<Vec<String>>,
    pub label_template: Option<String>,
    pub chapters: Vec<ChapterDefinition>,
}
