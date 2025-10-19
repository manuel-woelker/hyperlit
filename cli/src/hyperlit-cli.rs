use crate::arguments::{HyperlitCliArgs, HyperlitCliCommands};
use clap::Parser;
use hyperlit_base::log_error;
use hyperlit_base::logging::init_logging;
use hyperlit_base::result::HyperlitResult;
use hyperlit_engine::create_html::create_html;
use hyperlit_server::run_server::run_hyperlit_server;
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

fn main() {
    match main_internal() {
        Ok(_) => {}
        Err(err) => {
            log_error!("Error running hyperlit: {:?}", err);
            std::process::exit(1);
        }
    }
}

fn main_internal() -> HyperlitResult<()> {
    init_logging();
    let args = HyperlitCliArgs::parse();
    info!("hyperlit version {}", VERSION_STRING,);
    match args.command {
        Some(HyperlitCliCommands::Init {}) => todo!(),
        Some(HyperlitCliCommands::Watch {}) => todo!(),
        Some(HyperlitCliCommands::Serve {}) => {
            run_hyperlit_server()?;
        }
        None => {
            create_html()?;
        }
    }
    Ok(())
}
