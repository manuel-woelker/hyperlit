use hyperlit_base::logging::init_logging;
use hyperlit_core::config::HyperlitConfig;
use hyperlit_core::runner::Runner;

fn main() {
    init_logging();
    let runner = Runner::new(HyperlitConfig {
        docs_directory: "sample/docs".to_string(),
        build_directory: "sample/build".to_string(),
        output_directory: "sample/output".to_string(), 
    }).unwrap();
    runner.run().unwrap();
}
