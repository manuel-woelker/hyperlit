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
use hyperlit_pal::{Pal, PalHandle};
use hyperlit_parser_markdown::markdown::extract_markdown_info;
use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};
use std::collections::HashMap;
use tracing::{debug, error, info, info_span};

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

    config: HyperlitConfig,
    /// Path to a build directory used for temporary files
    build_directory: FilePath,
    /// Directory to write the complete documentation output to
    output_directory: FilePath,
    /// List of marker strings used to identify documentation segments to extract from the source code
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
            let mut state = EngineState {
                pal: self.pal.clone(),
                document_map: HashMap::new(),
                build_directory: root_path.join_normalized(&config.build_directory),
                output_directory: root_path.join_normalized(&config.output_directory),
                root_path,
                source_link_template: config.source_link_template.clone(),
                config,
            };
            state.load_documents()?;
            let end = std::time::Instant::now();
            info!(
                "Document loading took {} ms",
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
    fn load_documents(&mut self) -> HyperlitResult<()> {
        let span = info_span!("load documents");
        let _span = span.enter();
        let mut id_gen = IdGenerator::default();
        let mut document_map = HashMap::new();
        let mut errors = vec![];
        for directory_config in &self.config.directories {
            for path in &directory_config.paths {
                let path = self.root_path.join_normalized(path);
                let result = (|| -> HyperlitResult<()> {
                    let walk = self.pal.walk_directory(&path, &directory_config.globs)?;
                    for file_path in walk {
                        let file_path = file_path?;
                        debug!("parsing file {:?} ", file_path);
                        let result = (|| -> HyperlitResult<()> {
                            let source_content = self.pal.read_file_to_string(&file_path)?;
                            let markdown_info = extract_markdown_info(&source_content)?;
                            let title = markdown_info.title;
                            let id = id_gen.id_from(&title);
                            document_map.insert(
                                id.clone(),
                                DocumentData {
                                    id: (&id).into(),
                                    title,
                                    file: file_path.clone(),
                                    byte_range: None,
                                },
                            );
                            Ok(())
                        })()
                        .with_context(|| err!("Could not load file '{}'", file_path));
                        if let Err(err) = result {
                            errors.push(err);
                        }
                    }
                    Ok(())
                })();
                if let Err(err) = result {
                    errors.push(err);
                }
            }
        }
        self.document_map = document_map;
        if !errors.is_empty() {
            for error in &errors {
                error!(">>> {:#}", error);
            }
            error!(
                "Encountered {} errors while loading documents",
                errors.len()
            );
        }
        info!(
            "Loaded information for {} documents",
            self.document_map.len()
        );
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
