use crate::arguments::{HyperlitCliArgs, HyperlitCliCommands};
use clap::Parser;
use hyperlit_base::logging::init_logging;
use hyperlit_base::result::HyperlitResult;
use hyperlit_runner::runner::Runner;
use log::info;

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
            let mut runner = Runner::new()?;
            runner.run()
        }
    }
}
