use hyperlit_base::error::err;
use hyperlit_base::result::HyperlitResult;
use hyperlit_core::config::HyperlitConfig;
use hyperlit_export_html::html_exporter::export_book_to_html;
use hyperlit_model::book::Book;
use hyperlit_model::chapter::Chapter;
use hyperlit_model::database::DatabaseBox;
use hyperlit_model::file_source::InMemoryFileSource;
use hyperlit_model::value::Value;
use hyperlit_pal::{FilePath, Pal, PalHandle};
use hyperlit_parser_markdown::markdown::parse_markdown;
use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};
use tracing::{debug, info, info_span};

pub struct HyperlitEngine {
    #[allow(dead_code)]
    pal: PalHandle,
    state: RwLock<Result<EngineState, String>>,
}

#[allow(dead_code)]
struct EngineState {
    pal: PalHandle,

    config: HyperlitConfig,
    /// Root path to source code. This may be the repository root to collect all files
    src_directory: FilePath,
    /// Globs to use when searching for source files, these may be prefixed with "!" to exclude files or directories
    src_globs: Vec<String>,
    /// Path to the docs directory
    docs_directory: FilePath,
    /// Globs to use when searching for documentation files, may be "*" to include all files
    doc_globs: Vec<String>,
    /// Path to a build directory used for temporary files
    build_directory: FilePath,
    /// Directory to write the complete documentation output to
    output_directory: FilePath,
    /// The database used for storing intermediate data
    database: DatabaseBox,
    /// List of marker strings used to identify documentation segments to extract from the source code
    doc_markers: Vec<String>,
    /// Path to the root of the repository
    root_path: FilePath,
    /// Template used to generate links to source code (e.g. on github, etc.)
    source_link_template: Option<String>,
    // The complete book
    book: Book,
}

impl HyperlitEngine {
    pub fn new_handle(pal: PalHandle) -> Self {
        Self {
            pal,
            state: RwLock::new(Err("not initialized".to_string())),
        }
    }

    pub fn new(pal: impl Pal + 'static) -> Self {
        Self::new_handle(PalHandle::new(pal))
    }

    pub fn init(&self) {
        let result = (|| -> HyperlitResult<EngineState> {
            let start = std::time::Instant::now();
            let mut config_file = self.pal.read_file(&FilePath::from("hyperlit.toml"))?;
            let mut config_string = String::new();
            config_file.read_to_string(&mut config_string)?;
            let config = HyperlitConfig::from_string(&config_string)?;
            let root_path = FilePath::from(".");
            let docs_directory = root_path.join_normalized(&config.docs_directory);
            let book = Book::new(Value::String(config.title.clone()));
            let mut state = EngineState {
                pal: self.pal.clone(),
                book,
                src_directory: root_path.join_normalized(&config.src_directory),
                docs_directory,
                build_directory: root_path.join_normalized(&config.build_directory),
                output_directory: root_path.join_normalized(&config.output_directory),
                doc_globs: config.doc_globs.clone(),
                src_globs: config.src_globs.clone(),
                database: Box::new(hyperlit_database::in_memory_database::InMemoryDatabase::new()),
                doc_markers: config.doc_markers.clone(),
                root_path,
                source_link_template: config.source_link_template.clone(),
                config,
            };
            state.extract_segments()?;
            state.compile_book()?;
            let end = std::time::Instant::now();
            info!(
                "Document generation took {} ms",
                end.duration_since(start).as_millis()
            );
            Ok(state)
        })()
        .map_err(|err| err.to_string());
        *self.state.write() = result;
    }

    fn read(&self) -> HyperlitResult<MappedRwLockReadGuard<'_, EngineState>> {
        let read_guard = self.state.read();

        let mapped_guard = match read_guard.as_ref() {
            Ok(_state) => RwLockReadGuard::map(read_guard, |state| state.as_ref().unwrap()),
            Err(err) => {
                return Err(err!("Could not acquire read lock: {:?}", err));
            }
        };
        Ok(mapped_guard)
    }

    #[allow(dead_code)]
    fn write(&self) -> HyperlitResult<MappedRwLockWriteGuard<'_, EngineState>> {
        let write_guard = self.state.write();

        let mapped_guard = match write_guard.as_ref() {
            Ok(_state) => RwLockWriteGuard::map(write_guard, |state| state.as_mut().unwrap()),
            Err(err) => {
                return Err(err!("Could not acquire write lock: {:?}", err));
            }
        };
        Ok(mapped_guard)
    }

    pub fn render_book_html(&self) -> HyperlitResult<String> {
        let read = self.read()?;
        let book = &read.book;
        let html = export_book_to_html(book)?;
        Ok(html)
    }
}

impl EngineState {
    fn extract_segments(&mut self) -> HyperlitResult<()> {
        let span = info_span!("extract segments");
        let _span = span.enter();
        let extractor = hyperlit_extractor::extractor::Extractor::new(
            &self
                .doc_markers
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>(),
        );
        //        let git_info = GitInfo::new()?;
        //        let walk = create_walk(&self.src_directory, &self.src_globs)?;
        let walk = self
            .pal
            .walk_directory(&self.src_directory, &self.src_globs)?;
        for source_path in walk {
            let source_path = source_path?;
            debug!("extracting file {:?} ", source_path);
            let source_content = self.pal.read_file_to_string(&source_path)?;
            let source_path_string = source_path.to_string();
            let source = InMemoryFileSource::new(source_path_string, source_content);
            let mut segments = extractor.extract(&source)?;
            if segments.is_empty() {
                continue;
            }
            //                let last_modification_info = git_info.get_last_modification_info(source_path)?;
            for segment in &mut segments {
                //segment.last_modification = last_modification_info.clone();
                if let Some(ref url) = self.source_link_template {
                    let mut url = url.clone();
                    url = url.replace("{path}", segment.location.filepath());
                    url = url.replace("{line}", &segment.location.line().to_string());
                    segment.location_url = Some(url);
                }
            }
            self.database.add_segments(segments)?;
        }
        Ok(())
    }

    pub fn compile_book(&mut self) -> HyperlitResult<()> {
        let span = info_span!("compile book");
        let _span = span.enter();
        for segment in self.database.get_all_segments()? {
            let title = segment.title.clone();
            let mut chapter = Chapter::new(title.clone(), Value::String(segment.title.clone()));
            let body = parse_markdown(&segment.text)?;
            chapter.body = Value::Element(body);
            self.book.chapters.push(chapter);
        }
        Ok(())
    }
}
