#[derive(Debug)]
pub struct HyperlitConfig {
    pub src_directory: String,
    pub docs_directory: String,
    pub build_directory: String,
    pub output_directory: String,
    pub doc_extensions: Vec<String>,
}