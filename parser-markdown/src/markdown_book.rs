use hyperlit_base::result::HyperlitResult;
use hyperlit_model::element::Element;
use hyperlit_model::tags::BOOK;

pub fn parse_markdown_book(_input: &str) -> HyperlitResult<Element> {
    let book = BOOK.new_element();
    Ok(book)
}
