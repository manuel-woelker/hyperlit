use hyperlit_base::FilePath;
use hyperlit_base::error::{bail, err};
use hyperlit_base::id_generator::IdGenerator;
use hyperlit_base::result::{Context, HyperlitResult};
use hyperlit_core::config::HyperlitConfig;
use hyperlit_export_html::html_exporter::export_book_to_html;
use hyperlit_extractor::extractor::extract_hash_tags;
use hyperlit_model::book::Book;
use hyperlit_model::book_structure::{BookStructure, ChapterStructure};
use hyperlit_model::chapter_info::ChapterInfo;
use hyperlit_model::database::DatabaseBox;
use hyperlit_model::file_source::InMemoryFileSource;
use hyperlit_model::location::Location;
use hyperlit_model::segment::Segment;
use hyperlit_model::value::Value;
use hyperlit_pal::{Pal, PalHandle};
use hyperlit_parser_markdown::markdown::parse_markdown;
use hyperlit_parser_markdown::markdown_metadata::extract_markdown_metadata;
use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};
use serde::Serialize;
use std::collections::{HashMap, VecDeque};
use std::mem;
use tracing::{debug, error, info, info_span};

pub struct HyperlitEngine {
    #[allow(dead_code)]
    pal: PalHandle,
    state: RwLock<Result<EngineState, String>>,
}

#[allow(dead_code)]
struct EngineState {
    pal: PalHandle,

    /// Mapping from file index to file path
    file_map: HashMap<usize, FilePath>,

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

    /// The book structure
    book_structure: BookStructure,

    /// The complete book
    book: Book,

    /// Map from chapter id to chapter info
    chapter_map: HashMap<String, ChapterInfo>,
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
            let config = HyperlitConfig::from_string(&config_string)
                .with_context(|| "Failed to parse hyperlit.toml")?;
            let root_path = FilePath::from(".");
            let docs_directory = root_path.join_normalized(&config.docs_directory);
            let book = Book::new(Value::new_text_unspanned(&config.title));
            let book_structure = BookStructure::new(&config.title);
            let mut state = EngineState {
                pal: self.pal.clone(),
                file_map: HashMap::new(),
                book,
                chapter_map: HashMap::new(),
                book_structure,
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
            state.parse_documentation()?;
            state.extract_segments()?;
            state.compile_book()?;
            let end = std::time::Instant::now();
            info!(
                "Document generation took {} ms",
                end.duration_since(start).as_millis()
            );
            Ok(state)
        })()
        .map_err(|err| format!("{:?}", err));
        *self.state.write() = result;
    }

    fn read(&self) -> HyperlitResult<MappedRwLockReadGuard<'_, EngineState>> {
        let read_guard = self.state.read();

        let mapped_guard = match read_guard.as_ref() {
            Ok(_state) => RwLockReadGuard::map(read_guard, |state| state.as_ref().unwrap()),
            Err(err) => {
                return Err(err!("Initialization failed: {}", err));
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

    pub fn get_book_title(&self) -> HyperlitResult<String> {
        let read = self.read()?;
        let book = &read.book;
        Ok(book.title.to_string())
    }

    pub fn get_book_structure(&self) -> HyperlitResult<BookStructure> {
        let read = self.read()?;
        Ok(read.book_structure.clone())
    }

    pub fn config(&self) -> HyperlitResult<HyperlitConfig> {
        Ok(self.read()?.config.clone())
    }

    pub fn get_chapter_json(&self, chapter_id: &str) -> HyperlitResult<String> {
        let read = self.read()?;
        let chapter_info = read
            .chapter_map
            .get(chapter_id)
            .ok_or_else(|| err!("Chapter not found: {chapter_id}"))?;
        let Some(file) = &chapter_info.file() else {
            bail!("Chapter has no file associated: {chapter_id}")
        };
        let markdown = self.pal.read_file_to_string(file)?;
        let chapter_json = ChapterJson {
            chapter_id: chapter_id.to_string(),
            markdown,
        };
        Ok(serde_json::to_string(&chapter_json)?)
    }
}

#[derive(Serialize)]
struct ChapterJson {
    chapter_id: String,
    markdown: String,
}

impl EngineState {
    fn parse_documentation(&mut self) -> HyperlitResult<()> {
        let span = info_span!("parse documentation");
        let _span = span.enter();
        let walk = self
            .pal
            .walk_directory(&self.docs_directory, &self.doc_globs)?;
        for (file_index, source_path) in walk.enumerate() {
            let source_path = source_path?;
            self.file_map.insert(file_index, source_path.clone());
            debug!("parsing file {:?} ", source_path);
            let source_content = self.pal.read_file_to_string(&source_path)?;
            let mut root_element = parse_markdown(&source_content, file_index)?;
            // find heading
            let location = Location::new(source_path.to_string(), 0);
            let mut children = VecDeque::from(mem::take(root_element.children_mut()));
            loop {
                let Some(heading) = children.pop_front() else {
                    break;
                };
                // TODO: check if it is actually a heading
                let heading_text = heading.to_string();
                let extraction_result = extract_hash_tags(&heading_text);
                let tags = extraction_result.tags;
                let title = extraction_result.text;
                let mut segment_children = vec![];
                loop {
                    let Some(child) = children.pop_front() else {
                        break;
                    };
                    // TODO: break if it is actually a heading
                    segment_children.push(child);
                }
                let linebreak = source_content
                    .find("\n")
                    .ok_or_else(|| err!("Could not find linebreak"))?;
                let actual_source = source_content[linebreak..].to_string();
                let segment =
                    Segment::new(0, file_index, title, tags, actual_source, location.clone());
                self.database.add_segments(vec![segment])?;
            }
        }
        Ok(())
    }

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
        let id_gen = &mut IdGenerator::default();
        let definition = &self.config.structure;
        let mut chapters = vec![];
        let chapter_map = &mut HashMap::new();
        for chapter_definition in &definition.chapters {
            let title = chapter_definition.label.clone();
            let mut chapter = ChapterStructure::new_with_gen_id(title.clone(), id_gen);
            if let Some(directories) = &chapter_definition.directories {
                for directory in directories {
                    let dir_result = (|| -> HyperlitResult<()> {
                        let directory = self.docs_directory.join_normalized(directory);
                        debug!("Walking directory '{}' {:?}", directory, &self.doc_globs);
                        let walk = self.pal.walk_directory(&directory, &self.doc_globs)?;
                        for source_path in walk {
                            let Ok(source_path) = source_path else {
                                error!(
                                    "Failed to process file in directory '{}': {}",
                                    &directory,
                                    source_path.unwrap_err()
                                );
                                continue;
                            };
                            let file_result = || -> HyperlitResult<()> {
                                debug!("extracting file {:?} ", source_path);
                                let source_content = self.pal.read_file_to_string(&source_path)?;
                                let metadata = extract_markdown_metadata(&source_content)?;
                                let title: String =
                                    if let Some(title) = metadata.front_matter.get("title") {
                                        title.clone()
                                    } else if let Some(heading) = metadata.plain_heading {
                                        heading.clone()
                                    } else if let Some(file_name) = source_path.file_name() {
                                        file_name.to_string()
                                    } else {
                                        source_path.to_string()
                                    };
                                let mut sub_chapter =
                                    ChapterStructure::new_with_gen_id(title, id_gen);
                                self.add_chapter_info(chapter_map, &sub_chapter.id, &source_path);
                                sub_chapter.file = Some(source_path.clone());
                                chapter.chapters.push(sub_chapter);
                                Ok(())
                            }();
                            if let Err(e) = file_result {
                                error!("Failed to process file '{:?}': {}", source_path, e);
                            }
                        }
                        Ok(())
                    })();
                    if let Err(e) = dir_result {
                        error!("Failed to walk directory '{}': {}", directory, e);
                    }
                }
                sort_chapters(&mut chapter.chapters);
            }
            /*
            // TODO: allow multiple tags
            let segments = self
                .database
                .get_segments_by_tag(&chapter_definition.tags[0])?;
            for segment in segments {
                let title = segment.title.clone();
                let sub_chapter = ChapterStructure::new(title.clone());
                chapter.chapters.push(sub_chapter);
            }
            */
            chapter_map.insert(
                chapter.id.clone(),
                ChapterInfo::new_virtual(chapter.id.clone()),
            );
            chapters.push(chapter);
        }

        self.book_structure.chapters = chapters;
        self.chapter_map = mem::take(chapter_map);
        Ok(())
    }

    pub fn add_chapter_info(
        &self,
        chapter_map: &mut HashMap<String, ChapterInfo>,
        chapter_id: &str,
        path: &FilePath,
    ) {
        chapter_map.insert(
            chapter_id.to_string(),
            ChapterInfo::new(chapter_id.to_string(), path.clone()),
        );
    }
}

fn sort_chapters(chapters: &mut [ChapterStructure]) {
    chapters.sort_by(|a, b| a.label.cmp(&b.label));
}
