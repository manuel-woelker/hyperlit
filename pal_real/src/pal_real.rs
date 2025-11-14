use hyperlit_base::FilePath;
use hyperlit_base::error::err;
use hyperlit_base::result::{Context, HyperlitResult, info};
use hyperlit_pal::{FileChangeCallback, FileChangeEvent, Pal, ReadSeek};
use ignore::WalkBuilder;
use ignore::gitignore::GitignoreBuilder;
use ignore::overrides::OverrideBuilder;
use notify_debouncer_full::notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_full::{DebounceEventResult, Debouncer, RecommendedCache, new_debouncer};
use relative_path::PathExt;
use std::fmt::Debug;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::time::Duration;

pub struct PalReal {
    base_path: PathBuf,
    watchers: RwLock<Vec<Debouncer<RecommendedWatcher, RecommendedCache>>>,
}

impl PalReal {
    pub fn new() -> Self {
        let current_dir = std::env::current_dir().expect("Unable to access current directory");
        Self {
            base_path: current_dir,
            watchers: RwLock::new(Vec::new()),
        }
    }

    fn resolve_path(&self, path: &FilePath) -> HyperlitResult<PathBuf> {
        Ok(path.to_path(&self.base_path))
    }

    fn relativize_path(&self, path: &Path) -> HyperlitResult<FilePath> {
        Ok(path.relative_to(&self.base_path)?)
    }
}

impl Default for PalReal {
    fn default() -> Self {
        Self::new()
    }
}

impl Pal for PalReal {
    fn file_exists(&self, path: &FilePath) -> HyperlitResult<bool> {
        Ok(std::fs::exists(self.resolve_path(path)?)?)
    }

    fn read_executable_file(&self) -> HyperlitResult<Box<dyn ReadSeek + 'static>> {
        Ok(Box::new(File::open(
            std::env::current_exe().with_context(|| "Unable to open executable file")?,
        )?))
    }

    fn read_file(&self, path: &FilePath) -> HyperlitResult<Box<dyn ReadSeek + 'static>> {
        Ok(Box::new(
            File::open(self.resolve_path(path)?)
                .with_context(|| format!("Unable to open file '{}'", path))?,
        ))
    }

    fn create_file(&self, path: &FilePath) -> HyperlitResult<Box<dyn Write>> {
        Ok(Box::new(
            File::create(self.resolve_path(path)?)
                .with_context(|| format!("Unable to create file '{}'", path))?,
        ))
    }

    fn create_directory_all(&self, path: &FilePath) -> HyperlitResult<()> {
        std::fs::create_dir_all(self.resolve_path(path)?)
            .with_context(|| format!("Unable to create directory '{}'", path))?;
        Ok(())
    }

    fn remove_directory_all(&self, path: &FilePath) -> HyperlitResult<()> {
        let directory = self.resolve_path(path)?;
        if std::fs::exists(&directory)? {
            std::fs::remove_dir_all(&directory)
                .with_context(|| format!("Unable to remove directory '{}'", path))?;
        }
        Ok(())
    }

    fn walk_directory(
        &self,
        path: &FilePath,
        globs: &[String],
    ) -> HyperlitResult<Box<dyn Iterator<Item = HyperlitResult<FilePath>> + '_>> {
        let real_path = self.resolve_path(path)?;
        let mut walk_builder = WalkBuilder::new(&real_path);
        let mut overrides = OverrideBuilder::new(&real_path);
        for glob in globs {
            overrides.add(glob)?;
        }
        walk_builder.overrides(overrides.build()?);
        Ok(Box::new(
            walk_builder
                .build()
                .filter(|entry| match entry {
                    Ok(dir_entry) => {
                        if let Some(file_type) = &dir_entry.file_type()
                            && file_type.is_file()
                        {
                            true
                        } else {
                            false
                        }
                    }
                    Err(_) => false,
                })
                .flat_map(|entry| entry.map(|path| self.relativize_path(path.path()))),
        ))
    }

    fn watch_directory(
        &self,
        directory: &FilePath,
        globs: &[String],
        callback: FileChangeCallback,
    ) -> HyperlitResult<()> {
        let mut gitignore_builder = GitignoreBuilder::new(&self.base_path);
        for glob in globs {
            gitignore_builder.add_line(None, glob)?;
        }
        let gitignore = gitignore_builder.build()?;
        let mut debouncer = new_debouncer(
            Duration::from_millis(500),
            None,
            move |result: DebounceEventResult| match result {
                Ok(events) => {
                    let mut matched = false;
                    'outer: for event in events {
                        for path in &event.paths {
                            let matches = gitignore.matched_path_or_any_parents(path, false);
                            if matches.is_ignore() {
                                matched = true;
                                break 'outer;
                            }
                        }
                    }
                    if matched {
                        callback(FileChangeEvent {})
                    }
                }
                Err(errors) => errors.iter().for_each(|error| println!("{error:?}")),
            },
        )?;
        let path = self.resolve_path(directory)?;
        info!(
            "Watching directory {}, globs: {}",
            directory,
            globs.join(", ")
        );
        debouncer.watch(path, RecursiveMode::Recursive)?;
        self.watchers
            .write()
            .map_err(|_| err!("Unable to acquire write lock for watchers"))?
            .push(debouncer);
        Ok(())
    }
}

impl Debug for PalReal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PalReal").finish()
    }
}
