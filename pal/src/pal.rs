use hyperlit_base::result::HyperlitResult;
use relative_path::RelativePathBuf;
use std::fmt::Debug;
use std::io::{Read, Write};
use std::sync::Arc;

pub type FilePath = RelativePathBuf;

// Platform abstraction layer used to decouple logic from the underlying platform
pub trait Pal: Debug + Sync + Send + 'static {
    /// Read a file, the path is relative to the base directory
    fn read_file(&self, path: &FilePath) -> HyperlitResult<Box<dyn Read + 'static>>;

    /// Create a file to a string, the path is relative to the base directory
    fn create_file(&self, path: &FilePath) -> HyperlitResult<Box<dyn Write>>;

    /// Create a directory (including parent directories), the path is relative to the base directory
    fn create_directory_all(&self, path: &FilePath) -> HyperlitResult<()>;

    /// Remove a directory (including _all_ content), the path is relative to the base directory
    fn remove_directory_all(&self, path: &FilePath) -> HyperlitResult<()>;
}

#[derive(Debug, Clone)]
pub struct PalHandle(Arc<dyn Pal>);

impl PalHandle {
    pub fn new(pal: impl Pal + 'static) -> Self {
        Self(Arc::new(pal))
    }
}

// Implement Deref for convenience
impl std::ops::Deref for PalHandle {
    type Target = dyn Pal;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}
