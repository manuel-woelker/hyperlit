use hyperlit_base::logging::init_logging;
use hyperlit_core::runner::Runner;

fn main() {
    init_logging();
    let runner = Runner::new();
    runner.run().unwrap();
}
