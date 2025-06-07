use crate::arguments::{HyperlitCliArgs, HyperlitCliCommands};
use clap::Parser;
use hyperlit_base::logging::init_logging;
use hyperlit_base::result::HyperlitResult;
use hyperlit_core::runner::Runner;

pub mod arguments;

fn main() -> HyperlitResult<()> {
    init_logging();
    let args = HyperlitCliArgs::parse();
    match args.command {
        Some(HyperlitCliCommands::Init {}) => todo!(),
        Some(HyperlitCliCommands::Watch {}) => todo!(),
        None => {
            let mut runner = Runner::new()?;
            runner.run()
        },
    }
}
