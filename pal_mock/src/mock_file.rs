use crate::PalMock;
use hyperlit_pal::FilePath;
use std::io::Write;

pub struct MockFile {
    path: FilePath,
    data: Vec<u8>,
    pal_mock: PalMock,
}

impl MockFile {
    pub fn new(path: &FilePath, pal_mock: PalMock) -> Self {
        Self {
            path: path.clone(),
            data: vec![],
            pal_mock,
        }
    }
}

impl Write for MockFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.data.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Drop for MockFile {
    fn drop(&mut self) {
        self.pal_mock.log_effect(format!(
            "WRITE FILE: {} -> {}",
            self.path,
            String::from_utf8_lossy(&self.data)
        ));
        self.pal_mock
            .write()
            .file_map
            .insert(self.path.clone(), self.data.clone());
    }
}
