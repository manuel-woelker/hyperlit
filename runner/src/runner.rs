use hyperlit_backend::backend::BackendBox;
use hyperlit_base::context;
use hyperlit_base::error::bail;
use hyperlit_base::result::HyperlitResult;
use hyperlit_core::config::HyperlitConfig;
use hyperlit_database::DatabaseBox;
use hyperlit_extractor::git_info::GitInfo;
use hyperlit_model::directive_evaluation::DirectiveEvaluation;
use hyperlit_runtime::backend_compile_params_impl::BackendCompileParamsImpl;
use hyperlit_runtime::evaluate_directive::evaluate_directive;
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
    /// Path to the root of the repository
    root_path: PathBuf,
    /// Template used to generate links to source code (e.g. on github, etc.)
    source_link_template: Option<String>,
}

impl Runner {
    pub fn new() -> HyperlitResult<Self> {
        let config = HyperlitConfig::from_path("hyperlit.toml")?;
        Self::with_config(config)
    }

    pub fn with_config(config: HyperlitConfig) -> HyperlitResult<Self> {
        let root_path = PathBuf::from(config.config_path)
            .absolutize()?
            .parent()
            .expect("config parent path")
            .to_path_buf();
        let docs_directory = resolve_path(&root_path, &config.docs_directory)?;
        if !docs_directory.exists() {
            bail!(
                "Docs directory '{}' does not exist",
                docs_directory.display()
            );
        }
        Ok(Self {
            src_directory: resolve_path(&root_path, &config.src_directory)?,
            docs_directory,
            build_directory: resolve_path(&root_path, &config.build_directory)?,
            output_directory: resolve_path(&root_path, &config.output_directory)?,
            doc_globs: config.doc_globs,
            src_globs: config.src_globs,
            backend: Box::new(hyperlit_backend_mdbook::mdbook_backend::MdBookBackend::new()),
            database: Box::new(hyperlit_database::in_memory_database::InMemoryDatabase::new()),
            doc_markers: config.doc_markers,
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
        )))?;
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
            self.root_path.to_string_lossy().to_string(),
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
                        url = url.replace("{path}", segment.location.filepath());
                        url = url.replace("{line}", &segment.location.line().to_string());
                        segment.location_url = Some(url);
                    }
                }
                self.database.add_segments(segments)?;
            }
        }
        Ok(())
    }
}

fn resolve_path(root: &Path, path: &str) -> HyperlitResult<PathBuf> {
    Ok(root.join(path).absolutize()?.to_path_buf())
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

#[cfg(test)]
mod tests {
    use crate::runner::Runner;
    use hyperlit_base::result::HyperlitResult;
    use hyperlit_core::config::HyperlitConfig;
    use std::path::Path;

    #[test]
    fn test_run() -> HyperlitResult<()> {
        let config = HyperlitConfig::from_path("sample/hyperlit.toml")?;
        let mut runner = Runner::with_config(config)?;
        runner.run()?;
        assert!(
            Path::new("sample/output/index.html").exists(),
            "Output path index.html should exist"
        );
        Ok(())
    }
}
