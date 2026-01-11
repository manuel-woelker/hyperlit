use hyperlit_base::FilePath;
use hyperlit_base::shared_string::SharedString;

#[derive(Debug)]
pub struct DocumentData {
    pub id: SharedString,
    pub title: SharedString,
    pub file: FilePath,
}
