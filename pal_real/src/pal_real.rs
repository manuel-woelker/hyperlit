use hyperlit_base::result::HyperlitResult;
use hyperlit_pal::{FilePath, Pal};
use std::fmt::Debug;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

pub struct PalReal {
    base_path: PathBuf,
}

impl PalReal {
    pub fn new() -> Self {
        let current_dir = std::env::current_dir().expect("Unable to access current directory");
        Self {
            base_path: current_dir,
        }
    }

    fn resolve_path(&self, path: &FilePath) -> HyperlitResult<PathBuf> {
        Ok(path.to_path(&self.base_path))
    }
}

impl Default for PalReal {
    fn default() -> Self {
        Self::new()
    }
}

impl Pal for PalReal {
    fn read_file(&self, path: &FilePath) -> HyperlitResult<Box<dyn Read + 'static>> {
        Ok(Box::new(File::open(self.resolve_path(path)?)?))
    }

    fn create_file(&self, path: &FilePath) -> HyperlitResult<Box<dyn Write>> {
        Ok(Box::new(File::create(self.resolve_path(path)?)?))
    }

    fn create_directory_all(&self, path: &FilePath) -> HyperlitResult<()> {
        std::fs::create_dir_all(self.resolve_path(path)?)?;
        Ok(())
    }

    fn remove_directory_all(&self, path: &FilePath) -> HyperlitResult<()> {
        let directory = self.resolve_path(path)?;
        if std::fs::exists(&directory)? {
            std::fs::remove_dir_all(&directory)?;
        }
        Ok(())
    }
}

impl Debug for PalReal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PalReal").finish()
    }
}
