use crate::document::Document;
use crate::document_data::DocumentData;
use crate::document_info::DocumentInfo;
use crate::site_info::SiteInfo;
use crate::template_expander::TemplateExpander;
use hyperlit_base::FilePath;
use hyperlit_base::error::{bail, err};
use hyperlit_base::id_generator::IdGenerator;
use hyperlit_base::result::{Context, HyperlitResult};
use hyperlit_base::shared_string::SharedString;
use hyperlit_core::config::HyperlitConfig;
use hyperlit_extractor::extractor::extract_hash_tags;
use hyperlit_model::database::DatabaseBox;
use hyperlit_model::file_source::InMemoryFileSource;
use hyperlit_model::location::Location;
use hyperlit_model::segment::Segment;
use hyperlit_pal::{Pal, PalHandle};
use hyperlit_parser_markdown::markdown::parse_markdown;
use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};
use std::collections::{HashMap, VecDeque};
use std::mem;
use tracing::{debug, info, info_span};

pub struct HyperlitEngine {
    #[allow(dead_code)]
    pal: PalHandle,
    state: RwLock<Result<EngineState, String>>,
}

#[allow(dead_code)]
struct EngineState {
    pal: PalHandle,

    // Mapping from document id to document data
    document_map: HashMap<String, DocumentData>,

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
            let mut state = EngineState {
                pal: self.pal.clone(),
                file_map: HashMap::new(),
                document_map: HashMap::new(),
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

    pub fn config(&self) -> HyperlitResult<HyperlitConfig> {
        Ok(self.read()?.config.clone())
    }

    pub fn get_site_info(&self) -> HyperlitResult<SiteInfo> {
        self.read()?.get_site_info()
    }

    pub fn get_document_json(&self, document_id: &str) -> HyperlitResult<String> {
        let read = self.read()?;
        let document_data = read
            .document_map
            .get(document_id)
            .ok_or_else(|| err!("Document not found: {document_id}"))?;
        let file = &document_data.file;
        let markdown = self.pal.read_file_to_string(file)?;
        let edit_url = read
            .source_link_template
            .as_ref()
            .map(|link_template| {
                let expander = TemplateExpander::new(link_template)?;
                expander.expand(|s| {
                    Ok(match s.directive.as_str() {
                        "path" => file.to_string(),
                        "line" => "0".to_string(),
                        other => {
                            bail!("Unknown directive in source link template: '{}'", other)
                        }
                    })
                })
            })
            .transpose()?
            .map(SharedString::from);
        let chapter_json = Document {
            id: document_id.into(),
            title: document_data.title.clone(),
            markdown: markdown.into(),
            edit_url,
        };
        Ok(serde_json::to_string(&chapter_json)?)
    }
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
                let segment = Segment::new(
                    0,
                    file_index,
                    title.clone(),
                    tags,
                    actual_source,
                    location.clone(),
                );
                self.database.add_segments(vec![segment])?;
            }
        }
        let mut document_map = HashMap::new();
        let mut id_gen = IdGenerator::default();
        for segment in self.database.get_all_segments()? {
            let title = &segment.title;
            let id = id_gen.id_from(title);
            let source_path = self.file_map.get(&segment.file_index).unwrap();
            document_map.insert(
                id.clone(),
                DocumentData {
                    id: (&id).into(),
                    title: title.into(),
                    file: source_path.clone(),
                },
            );
        }
        self.document_map = document_map;
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
                    url = url.replace("${path}", segment.location.filepath());
                    url = url.replace("${line}", &segment.location.line().to_string());
                    segment.location_url = Some(url);
                }
            }
            self.database.add_segments(segments)?;
        }
        Ok(())
    }

    pub fn get_site_info(&self) -> HyperlitResult<SiteInfo> {
        let mut documents: Vec<_> = self
            .document_map
            .values()
            .map(|d| DocumentInfo {
                id: d.id.clone(),
                title: d.title.clone(),
            })
            .collect();
        documents.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(SiteInfo {
            title: self.config.title.clone(),
            documents,
        })
    }
}
