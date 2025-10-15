use hyperlit_base::result::HyperlitResult;
use hyperlit_model::book::Book;
use hyperlit_model::value::Value;
use toml_span::parse;

pub fn parse_book_toml(input: &str) -> HyperlitResult<Book> {
    let toml = parse(input)?;
    let mut book = Book::new(Value::new_empty());
    if let Some(title) = toml.pointer("/book/title").and_then(|title| title.as_str()) {
        book.title = Value::new_string(title.to_string());
    };
    if let Some(authors) = toml
        .pointer("/book/authors")
        .and_then(|authors| authors.as_array())
    {
        for author in authors {
            book.authors
                .push(Value::new_string(author.as_str().unwrap().to_string()));
        }
    };
    Ok(book)
}

#[cfg(test)]
mod tests {
    use super::parse_book_toml;
    use expect_test::{Expect, expect};
    use hyperlit_base::result::HyperlitResult;

    fn test_parse(input: &str, expected: Expect) -> HyperlitResult<()> {
        let book = parse_book_toml(input)?;
        expected.assert_debug_eq(&book);
        Ok(())
    }

    #[test]
    fn test_parse_book_toml() -> HyperlitResult<()> {
        test_parse(
            r#"
            [book]
            title = "My Book"
            authors = ["Author 1", "Author 2"]
            "#,
            expect![[r#"
                Book {
                    title: String(
                        "My Book",
                    ),
                    authors: [
                        String(
                            "Author 1",
                        ),
                        String(
                            "Author 2",
                        ),
                    ],
                    chapters: [],
                }
            "#]],
        )
    }
}
