use hyperlit_base::logging::init_logging;
use hyperlit_base::result::HyperlitResult;
use hyperlit_core::runner::Runner;

fn main() -> HyperlitResult<()> {
    init_logging();
    std::env::set_current_dir("sample")?;
    let mut runner = Runner::new()?;
    runner.run()
}
