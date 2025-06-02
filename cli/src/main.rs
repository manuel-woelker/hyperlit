use hyperlit_base::logging::init_logging;
use hyperlit_core::config::HyperlitConfig;
use hyperlit_core::runner::Runner;

fn main() {
    init_logging();
    let mut runner = Runner::new(HyperlitConfig {
        src_directory: "sample/src".to_string(),
        docs_directory: "sample/docs".to_string(),
        build_directory: "sample/build".to_string(),
        output_directory: "sample/output".to_string(),
        doc_extensions: vec!["md".to_string(), "mdx".to_string()],
    }).unwrap();
    runner.run().unwrap();
}
