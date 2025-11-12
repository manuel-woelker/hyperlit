use hyperlit_base::FilePath;
use hyperlit_base::id::new_id;
use hyperlit_base::id_generator::IdGenerator;
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
    pub id: String,
    pub tags: Vec<String>,
    pub file: Option<FilePath>,
    pub chapters: Vec<ChapterStructure>,
}

impl ChapterStructure {
    pub fn new(label: impl Into<String>) -> ChapterStructure {
        ChapterStructure {
            label: label.into(),
            id: new_id(),
            file: None,
            tags: Vec::new(),
            chapters: Vec::new(),
        }
    }

    pub fn new_with_gen_id(label: impl Into<String>, id_gen: &mut IdGenerator) -> ChapterStructure {
        let label = label.into();
        let id_gen = id_gen.id_from(&label);
        ChapterStructure {
            label,
            id: id_gen,
            file: None,
            tags: Vec::new(),
            chapters: Vec::new(),
        }
    }
}
