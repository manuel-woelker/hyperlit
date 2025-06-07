use crate::config::HyperlitConfig;
use hyperlit_base::result::HyperlitResult;
use hyperlit_base::{bail, context};
use hyperlit_database::DatabaseBox;
use hyperlit_model::backend::{BackendBox, BackendCompileParams};
use path_absolutize::Absolutize;
use std::collections::HashSet;
use std::ffi::OsString;
use std::fs::{File, create_dir_all, remove_dir_all};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use tracing::{debug, info, info_span};
use walkdir::WalkDir;

pub struct Runner {
    src_directory: PathBuf,
    docs_directory: PathBuf,
    build_directory: PathBuf,
    output_directory: PathBuf,
    doc_extensions: HashSet<OsString>,
    src_extensions: HashSet<OsString>,
    backend: BackendBox,
    database: DatabaseBox,
    doc_markers: Vec<String>,
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
            doc_extensions: HashSet::from_iter(config.doc_extensions.iter().map(OsString::from)),
            src_extensions: HashSet::from_iter(config.src_extensions.iter().map(OsString::from)),
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
        self.copy_docs()?;
        context!("run backend" => self.backend.compile(&BackendCompileParams {
            build_directory: self.build_directory.clone(),
            output_directory: self.output_directory.clone(),
        }))?;
        let run_duration = start_time.elapsed();
        info!("run completed in {}ms", run_duration.as_millis());
        Ok(())
    }

    pub fn copy_docs(&self) -> HyperlitResult<()> {
        context!("copy docs directory {:?} to build directory {:?}", self.docs_directory, self.build_directory =>
            for entry in WalkDir::new(&self.docs_directory) {
                let entry = entry?;
                let source_path = entry.path();
                let destination_path = self.build_directory.join(source_path.strip_prefix(&self.docs_directory)?);
                if source_path.is_dir() {
                    create_dir_all(self.build_directory.join(&destination_path))?;
                } else {
                    let is_doc_extension = source_path.extension().map(|ext| self.doc_extensions.contains(ext)).unwrap_or(false);
                    if is_doc_extension {
                        info!("processing file {:?} to {:?} ", source_path, destination_path);
                        self.process_doc(source_path, &destination_path)?;
                    } else {
                        debug!("copying file {:?} to {:?} ", source_path, destination_path);
                        std::fs::copy(source_path, destination_path)?;
                    }
                }
            }
            HyperlitResult::<()>::Ok(())
        )
    }

    fn process_doc(&self, source_path: &Path, destination_path: &Path) -> HyperlitResult<()> {
        let mut destination_file = BufWriter::new(File::create(destination_path)?);
        for line in BufReader::new(File::open(source_path)?).lines() {
            let line = line?;
            if line.trim() == "§{include}" {
                for segment in self.database.get_segments()? {
                    let text_to_insert = self.backend.transform_segment(segment)?;
                    destination_file.write_all(text_to_insert.as_bytes())?;
                    destination_file.write_all(b"\n")?;
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
        for entry in WalkDir::new(&self.src_directory) {
            let entry = entry?;
            let extension = entry.path().extension();
            let is_src_extension = extension
                .map(|ext| self.src_extensions.contains(ext))
                .unwrap_or(false);
            let source_path = entry.path();
            if is_src_extension && source_path.is_file() {
                info!("extracting file {:?} ", source_path);
                self.database
                    .add_segments(extractor.extract(&source_path)?)?;
            }
        }
        Ok(())
    }
}
