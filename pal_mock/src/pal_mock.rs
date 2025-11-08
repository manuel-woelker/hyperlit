use crate::mock_file::MockFile;
use expect_test::Expect;
use hyperlit_base::error::err;
use hyperlit_base::result::HyperlitResult;
use hyperlit_pal::{FileChangeCallback, FilePath, Pal};
use std::collections::HashMap;
use std::fmt::Debug;
use std::io::{Cursor, Read, Write};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub mod mock_file;

#[derive(Clone, Default)]
pub struct PalMock {
    inner: Arc<RwLock<PalMockInner>>,
}

#[derive(Default)]
struct PalMockInner {
    effects_string: String,
    file_map: HashMap<FilePath, Vec<u8>>,
}

impl PalMock {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(PalMockInner {
                effects_string: String::new(),
                file_map: HashMap::new(),
            })),
        }
    }

    fn read(&self) -> RwLockReadGuard<'_, PalMockInner> {
        self.inner
            .read()
            .expect("Unable to acquire read lock for mock PAL")
    }

    fn write(&self) -> RwLockWriteGuard<'_, PalMockInner> {
        self.inner
            .write()
            .expect("Unable to acquire write lock for mock PAL")
    }

    pub fn log_effect(&self, effect: impl AsRef<str>) {
        self.write().effects_string.push_str(effect.as_ref());
        self.write().effects_string.push('\n');
    }

    pub fn verify_effects(&self, expected: Expect) {
        expected.assert_eq(&self.read().effects_string);
        self.write().effects_string.clear();
    }

    #[allow(dead_code)]
    pub fn get_effects(&self) -> String {
        self.read().effects_string.clone()
    }

    pub fn clear_effects(&self) {
        self.write().effects_string.clear();
    }

    pub fn set_file(&self, file_path: &str, content: impl Into<Vec<u8>>) {
        self.write()
            .file_map
            .insert(FilePath::from(file_path), content.into());
    }
}

impl Pal for PalMock {
    fn read_file(&self, path: &FilePath) -> HyperlitResult<Box<dyn Read + 'static>> {
        self.log_effect(format!("READ FILE: {path}"));
        Ok(Box::new(Cursor::new(
            self.read()
                .file_map
                .get(path)
                .ok_or_else(|| err!("File '{path}' does not exist"))?
                .clone(),
        )))
    }

    fn create_file(&self, path: &FilePath) -> HyperlitResult<Box<dyn Write>> {
        self.log_effect(format!("CREATE FILE: {path}"));
        Ok(Box::new(MockFile::new(path, self.clone())))
    }

    fn create_directory_all(&self, _path: &FilePath) -> HyperlitResult<()> {
        todo!()
    }

    fn remove_directory_all(&self, _path: &FilePath) -> HyperlitResult<()> {
        todo!()
    }

    fn walk_directory(
        &self,
        _path: &FilePath,
        _globs: &[String],
    ) -> HyperlitResult<Box<dyn Iterator<Item = HyperlitResult<FilePath>> + '_>> {
        todo!()
    }

    fn watch_directory(
        &self,
        _directory: &FilePath,
        _globs: &[String],
        _callback: FileChangeCallback,
    ) -> HyperlitResult<()> {
        todo!()
    }
}

impl Debug for PalMock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PalMock").finish()
    }
}
