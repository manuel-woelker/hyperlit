use crate::config::HyperlitConfig;
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use hyperlit_base::result::HyperlitResult;
use hyperlit_base::{bail, context};
use hyperlit_model::backend::{Backend, BackendCompileParams};
use std::fs::{create_dir_all, read_dir, remove_dir_all};
use std::path::PathBuf;
use path_absolutize::Absolutize;

pub struct Runner {
    docs_directory: PathBuf,
    build_directory: PathBuf,
    output_directory: PathBuf,
    backend: Box<dyn Backend>,
}

impl Runner {
    pub fn new(config: HyperlitConfig) -> HyperlitResult<Self> {
        let docs_directory = PathBuf::from(&config.docs_directory).absolutize()?.to_path_buf();
        if !docs_directory.exists() {
            bail!("Docs directory '{}' does not exist", config.docs_directory);
        }
        Ok(Self {
            docs_directory,
            build_directory: PathBuf::from(&config.build_directory).absolutize()?.to_path_buf(),
            output_directory: PathBuf::from(&config.output_directory).absolutize()?.to_path_buf(),
            backend: Box::new(hyperlit_backend_mdbook::mdbook_backend::MdBookBackend::new()),
        })
    }

    pub fn run(&self) -> HyperlitResult<()> {
        if self.build_directory.exists() {
            context!("remove build directory {:?}", self.build_directory =>  remove_dir_all(&self.build_directory))?;
        }
        if self.output_directory.exists() {
            context!("remove output directory {:?}", self.output_directory =>  remove_dir_all(&self.output_directory))?;
        }
        context!("create build directory {:?}", self.build_directory =>  create_dir_all(&self.build_directory))?;
        context!("create output directory {:?}", self.output_directory =>  create_dir_all(&self.output_directory))?;

        context!("copy docs directory {:?} to build directory {:?}", self.docs_directory, self.build_directory => copy_items(&read_dir(&self.docs_directory)?.map(|entry| entry.unwrap().path()).collect::<Vec<_>>(), &self.build_directory, &CopyOptions::new()))?;
        context!("run backend" => self.backend.compile(&BackendCompileParams {
            build_directory: self.build_directory.clone(),
            output_directory: self.output_directory.clone(),
        }))?;
        Ok(())
    }
}
