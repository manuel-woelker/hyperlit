use hyperlit_backend::backend::{BackendBox, BackendCompileParams};
use hyperlit_base::result::HyperlitResult;
use hyperlit_base::{bail, context};
use hyperlit_core::config::HyperlitConfig;
use hyperlit_database::evaluate_directive::evaluate_directive;
use hyperlit_database::{Database, DatabaseBox};
use hyperlit_extractor::git_info::GitInfo;
use hyperlit_model::directive_evaluation::DirectiveEvaluation;
use hyperlit_model::segment::SegmentId;
use ignore::overrides::OverrideBuilder;
use ignore::{Walk, WalkBuilder};
use path_absolutize::Absolutize;
use std::fs::{File, create_dir_all, remove_dir_all};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use tracing::{debug, info, info_span};
use walkdir::WalkDir;

/// The main hyperlit runner, responsible for running the document generation process
pub struct Runner {
    /// Root path to source code. This may be the repository root to collect all files
    src_directory: PathBuf,
    /// Globs to use when searching for source files, these may be prefixed with "!" to exclude files or directories
    src_globs: Vec<String>,
    /// Path to the docs directory
    docs_directory: PathBuf,
    /// Globs to use when searching for documentation files, may be "*" to include all files
    doc_globs: Vec<String>,
    /// Path to a build directory used for temporary files
    build_directory: PathBuf,
    /// Directory to write the complete documentation output to
    output_directory: PathBuf,
    /// The backend used for generating the documentation
    backend: BackendBox,
    /// The database used for storing intermediate data
    database: DatabaseBox,
    /// List of marker strings used to identify documentation segments to extract from the source code
    doc_markers: Vec<String>,
}

struct BackendCompileParamsImpl<'a> {
    docs_directory: &'a Path,
    build_directory: &'a Path,
    output_directory: &'a Path,
    database: &'a mut dyn Database,
}

impl BackendCompileParams for BackendCompileParamsImpl<'_> {
    fn docs_directory(&self) -> &Path {
        self.docs_directory
    }

    fn build_directory(&self) -> &Path {
        self.build_directory
    }

    fn output_directory(&self) -> &Path {
        self.output_directory
    }

    fn evaluate_directive(&self, directive: &str) -> HyperlitResult<DirectiveEvaluation> {
        evaluate_directive(directive, self.database)
    }

    fn set_segment_included(&mut self, segment_id: SegmentId) -> HyperlitResult<()> {
        self.database.set_segment_included(segment_id)
    }
}

impl Runner {
    pub fn new() -> HyperlitResult<Self> {
        let config = HyperlitConfig::from_path("hyperlit.toml")?;
        Self::with_config(config)
    }

    pub fn with_config(config: HyperlitConfig) -> HyperlitResult<Self> {
        let docs_directory = PathBuf::from(&config.docs_directory)
            .absolutize()?
            .to_path_buf();
        if !docs_directory.exists() {
            bail!("Docs directory '{}' does not exist", config.docs_directory);
        }
        Ok(Self {
            src_directory: PathBuf::from(&config.src_directory)
                .absolutize()?
                .to_path_buf(),
            docs_directory,
            build_directory: PathBuf::from(&config.build_directory)
                .absolutize()?
                .to_path_buf(),
            output_directory: PathBuf::from(&config.output_directory)
                .absolutize()?
                .to_path_buf(),
            doc_globs: config.doc_globs,
            src_globs: config.src_globs,
            backend: Box::new(hyperlit_backend_mdbook::mdbook_backend::MdBookBackend::new()),
            database: Box::new(hyperlit_database::in_memory_database::InMemoryDatabase::new()),
            doc_markers: config.doc_markers,
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

        self.extract_segments()?;
        self.backend.prepare(&mut BackendCompileParamsImpl {
            docs_directory: &self.docs_directory,
            build_directory: &self.build_directory,
            output_directory: &self.output_directory,
            database: self.database.as_mut(),
        })?;
        self.copy_docs()?;
        context!("run backend" => self.backend.compile(&BackendCompileParamsImpl {
            docs_directory: &self.docs_directory,
            build_directory: &self.build_directory,
            output_directory: &self.output_directory,
            database: self.database.as_mut(),
        }))?;
        let run_duration = start_time.elapsed();
        info!("run completed in {}ms", run_duration.as_millis());
        Ok(())
    }

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
            let maybe_directive = line.trim();
            let prefix = "§{";
            if maybe_directive.starts_with(prefix) && maybe_directive.ends_with("}") {
                let evaluation = evaluate_directive(maybe_directive, self.database.as_ref())?;
                match evaluation {
                    DirectiveEvaluation::Segments { segments } => {
                        for segment in segments {
                            let text_to_insert = self.backend.transform_segment(segment)?;
                            destination_file.write_all(text_to_insert.as_bytes())?;
                            destination_file.write_all(b"\n")?;
                        }
                    }
                }
            } else {
                destination_file.write_all(line.as_bytes())?;
                destination_file.write_all(b"\n")?;
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
                }
                self.database.add_segments(segments)?;
            }
        }
        Ok(())
    }
}

fn create_walk(base_path: &Path, globs: &[String]) -> HyperlitResult<Walk> {
    let mut walk_builder = WalkBuilder::new(base_path);
    let mut overrides = OverrideBuilder::new(base_path);
    for glob in globs {
        overrides.add(glob)?;
    }
    walk_builder.overrides(overrides.build()?);
    Ok(walk_builder.build())
}
