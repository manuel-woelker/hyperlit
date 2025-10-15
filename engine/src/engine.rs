use hyperlit_base::result::HyperlitResult;
use hyperlit_export_html::html_exporter::export_book_to_html;
use hyperlit_model::book::Book;

pub struct HyperlitEngine {
    book: Book,
}

impl HyperlitEngine {
    pub fn new(book: Book) -> Self {
        Self { book }
    }
    pub fn render_book_html(&self) -> HyperlitResult<String> {
        export_book_to_html(&self.book)
    }
}
