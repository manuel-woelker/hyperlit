use crate::arguments::{HyperlitCliArgs, HyperlitCliCommands};
use clap::Parser;
use hyperlit_base::logging::init_logging;
use hyperlit_base::result::HyperlitResult;
use log::info;
use std::fs::create_dir_all;
use std::io::Write;
use std::path::Path;

pub mod arguments;

pub const VERSION_STRING: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "  (",
    env!("REVISION"),
    " ",
    env!("LAST_COMMIT_DATE"),
    ")",
);

fn main() -> HyperlitResult<()> {
    init_logging();
    let args = HyperlitCliArgs::parse();
    info!("hyperlit version {}", VERSION_STRING,);
    match args.command {
        Some(HyperlitCliCommands::Init {}) => todo!(),
        Some(HyperlitCliCommands::Watch {}) => todo!(),
        None => {
            create_dir_all(Path::new("output"))?;
            let mut index_file = std::fs::File::create("output/index.html")?;
            index_file.write_all(b"<h1>hyperlit</h1>")?;
            //            let mut runner = Runner::new()?;
            //            runner.run()
        }
    }
    Ok(())
}
