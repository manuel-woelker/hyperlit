#[derive(Debug, Clone)]
pub struct BookStructure {
    pub chapters: Vec<ChapterDefinition>,
}

#[derive(Debug, Clone)]
pub struct ChapterDefinition {
    pub label: String,
    pub tags: Vec<String>,
    pub chapters: Vec<ChapterDefinition>,
}
