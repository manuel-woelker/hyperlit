use hyperlit_pal::FilePath;

pub struct TextEdit {
    pub path: FilePath,
    pub offset: usize,
    pub expected_text: String,
    pub new_text: String,
}

impl TextEdit {
    pub fn new(
        path: impl Into<FilePath>,
        offset: usize,
        expected_text: impl Into<String>,
        new_text: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            offset,
            expected_text: expected_text.into(),
            new_text: new_text.into(),
        }
    }
}
