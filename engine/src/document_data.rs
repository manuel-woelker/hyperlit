use hyperlit_base::FilePath;

#[derive(Debug)]
pub struct DocumentData {
    pub id: String,
    pub title: String,
    pub file_reference: FilePath,
}
