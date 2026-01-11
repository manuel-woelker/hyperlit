use hyperlit_backend::backend::BackendBox;
use hyperlit_base::context;
use hyperlit_base::result::HyperlitResult;
use hyperlit_core::config::HyperlitConfig;
use hyperlit_database::DatabaseBox;
use ignore::overrides::OverrideBuilder;
use ignore::{Walk, WalkBuilder};
use path_absolutize::Absolutize;
use std::fs::{create_dir_all, remove_dir_all};
use std::path::{Path, PathBuf};
use tracing::{info, info_span};

/// The main hyperlit runner, responsible for running the document generation process
#[allow(dead_code)]
pub struct Runner {
    /// Path to a build directory used for temporary files
    build_directory: PathBuf,
    /// Directory to write the complete documentation output to
    output_directory: PathBuf,
    /// The backend used for generating the documentation
    backend: BackendBox,
    /// The database used for storing intermediate data
    database: DatabaseBox,
    /// Path to the root of the repository
    #[allow(dead_code)]
    root_path: PathBuf,
    /// Template used to generate links to source code (e.g. on github, etc.)
    source_link_template: Option<String>,
}

impl Runner {
    pub fn new() -> HyperlitResult<Self> {
        todo!()
    }

    pub fn with_config(config: HyperlitConfig) -> HyperlitResult<Self> {
        let root_path = PathBuf::from(config.config_path)
            .absolutize()?
            .parent()
            .expect("config parent path")
            .to_path_buf();
        Ok(Self {
            build_directory: resolve_path(&root_path, &config.build_directory)?,
            output_directory: resolve_path(&root_path, &config.output_directory)?,
            backend: Box::new(hyperlit_backend_mdbook::mdbook_backend::MdBookBackend::new()),
            database: Box::new(hyperlit_database::in_memory_database::InMemoryDatabase::new()),
            root_path,
            source_link_template: config.source_link_template,
        })
    }

    pub fn run(&mut self) -> HyperlitResult<()> {
        let start_time = std::time::Instant::now();
        let span = info_span!("run");
        let _span = span.enter();
        if self.build_directory.exists() {
            context!("remove build directory {:?}", self.build_directory =>  remove_dir_all(&self.build_directory))?;
        }
        if self.output_directory.exists() {
            context!("remove output directory {:?}", self.output_directory =>  remove_dir_all(&self.output_directory))?;
        }
        context!("create build directory {:?}", self.build_directory =>  create_dir_all(&self.build_directory))?;
        context!("create output directory {:?}", self.output_directory =>  create_dir_all(&self.output_directory))?;
        /*
        self.extract_segments()?;
        self.backend.prepare(&mut BackendCompileParamsImpl::new(
            &self.docs_directory,
            &self.build_directory,
            &self.output_directory,
            self.database.as_mut(),
        ))?;
        self.copy_docs()?;
        context!("run backend" => self.backend.compile(&BackendCompileParamsImpl::new(
            &self.docs_directory,
            &self.build_directory,
            &self.output_directory,
            self.database.as_mut(),
        )))?;*/
        let run_duration = start_time.elapsed();
        info!("run completed in {}ms", run_duration.as_millis());
        Ok(())
    }
    /*
    pub fn copy_docs(&self) -> HyperlitResult<()> {
        context!("copy docs directory {:?} to build directory {:?}", self.docs_directory, self.build_directory => {
            let mut overrides = OverrideBuilder::new(&self.docs_directory);
            for glob in &self.doc_globs {
                overrides.add(glob)?;
            }
            let matcher = overrides.build()?;
            for entry in WalkDir::new(&self.docs_directory) {
                let entry = entry?;
                let source_path = entry.path();
                let destination_path = self.build_directory.join(source_path.strip_prefix(&self.docs_directory)?);
                if source_path.is_dir() {
                    create_dir_all(self.build_directory.join(&destination_path))?;
                } else {
                    let must_process = !matcher.matched(source_path, false).is_ignore();
                    if must_process {
                        debug!("processing file {:?} to {:?} ", source_path, destination_path);
                        self.process_doc(source_path, &destination_path)?;
                    } else {
                        debug!("copying file {:?} to {:?} ", source_path, destination_path);
                        std::fs::copy(source_path, destination_path)?;
                    }
                }
            }
            HyperlitResult::<()>::Ok(())
            }
        )
    }

    fn process_doc(&self, source_path: &Path, destination_path: &Path) -> HyperlitResult<()> {
        let mut destination_file = BufWriter::new(File::create(destination_path)?);
        for line in BufReader::new(File::open(source_path)?).lines() {
            let line = line?;
            let evaluation = evaluate_directive(&line, self.database.as_ref())?;
            match evaluation {
                DirectiveEvaluation::Segments { segments } => {
                    for segment in segments {
                        let text_to_insert = self.backend.transform_segment(segment)?;
                        destination_file.write_all(text_to_insert.as_bytes())?;
                        destination_file.write_all(b"\n")?;
                    }
                }
                DirectiveEvaluation::NoDirective => {
                    destination_file.write_all(line.as_bytes())?;
                    destination_file.write_all(b"\n")?;
                }
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
        let git_info = GitInfo::new()?;
        let walk = create_walk(&self.src_directory, &self.src_globs)?;
        for entry in walk {
            let entry = entry?;
            let source_path = entry.path();
            if source_path.is_file() {
                debug!("extracting file {:?} ", source_path);
                let mut segments = extractor.extract(&source_path)?;
                if segments.is_empty() {
                    continue;
                }
                let last_modification_info = git_info.get_last_modification_info(source_path)?;
                for segment in &mut segments {
                    segment.last_modification = last_modification_info.clone();
                    if let Some(ref url) = self.source_link_template {
                        let mut url = url.clone();
                        url = url.replace("${path}", segment.location.filepath());
                        url = url.replace("${line}", &segment.location.line().to_string());
                        segment.location_url = Some(url);
                    }
                }
                self.database.add_segments(segments)?;
            }
        }
        Ok(())
    }
    */
}

fn resolve_path(root: &Path, path: &str) -> HyperlitResult<PathBuf> {
    Ok(root.join(path).absolutize()?.to_path_buf())
}

#[allow(dead_code)]
fn create_walk(base_path: &Path, globs: &[String]) -> HyperlitResult<Walk> {
    let mut walk_builder = WalkBuilder::new(base_path);
    let mut overrides = OverrideBuilder::new(base_path);
    for glob in globs {
        overrides.add(glob)?;
    }
    walk_builder.overrides(overrides.build()?);
    Ok(walk_builder.build())
}
