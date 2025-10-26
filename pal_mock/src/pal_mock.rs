use expect_test::Expect;
use hyperlit_base::result::HyperlitResult;
use hyperlit_pal::{FileChangeCallback, FilePath, Pal};
use std::fmt::Debug;
use std::io::{Read, Write};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Clone, Default)]
pub struct PalMock {
    inner: Arc<RwLock<PalMockInner>>,
}

#[derive(Default)]
struct PalMockInner {
    effects_string: String,
}

impl PalMock {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(PalMockInner {
                effects_string: String::new(),
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
}

impl Pal for PalMock {
    fn read_file(&self, _path: &FilePath) -> HyperlitResult<Box<dyn Read + 'static>> {
        todo!()
    }

    fn create_file(&self, _path: &FilePath) -> HyperlitResult<Box<dyn Write>> {
        todo!()
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
        _callback: FileChangeCallback,
        _globs: &[String],
    ) -> HyperlitResult<()> {
        todo!()
    }
}

impl Debug for PalMock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PalMock").finish()
    }
}
