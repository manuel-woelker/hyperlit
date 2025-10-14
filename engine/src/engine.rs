use hyperlit_base::result::HyperlitResult;

#[derive(Default)]
pub struct HyperlitEngine {}

impl HyperlitEngine {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn render_book(&self) -> HyperlitResult<String> {
        Ok("<h1>Book</h1>".to_string())
    }
}
