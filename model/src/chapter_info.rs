use hyperlit_base::FilePath;

/// Information about a chapter, used to serve chapters from disk
pub struct ChapterInfo {
    chapter_id: String,
    file: Option<FilePath>,
}

impl ChapterInfo {
    pub fn new(chapter_id: String, file: FilePath) -> ChapterInfo {
        ChapterInfo {
            chapter_id,
            file: Some(file),
        }
    }

    pub fn new_virtual(chapter_id: String) -> ChapterInfo {
        ChapterInfo {
            chapter_id,
            file: None,
        }
    }

    pub fn chapter_id(&self) -> &String {
        &self.chapter_id
    }

    pub fn file(&self) -> &Option<FilePath> {
        &self.file
    }
}
