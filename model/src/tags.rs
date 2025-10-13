use crate::key::Key;

// Document types ------------------------

/// A generic root node for an document
pub const DOCUMENT: Key = Key::from_static("document");
/// A complete book with multiple chapters
pub const BOOK: Key = Key::from_static("book");
/// A short article, usually a single page
pub const ARTICLE: Key = Key::from_static("article");

// Block types ---------------------------

/// A chapter in a book
pub const TITLE: Key = Key::from_static("title");
/// A chapter in a book
pub const CHAPTER: Key = Key::from_static("chapter");
/// A chapter or section heading
pub const HEADING: Key = Key::from_static("heading");
/// A paragraph of text
pub const PARAGRAPH: Key = Key::from_static("paragraph");
/// A list of items
pub const LIST: Key = Key::from_static("list");
/// An item in a list
pub const ITEM: Key = Key::from_static("item");
/// A block of code
pub const CODE_BLOCK: Key = Key::from_static("code-block");
/// A quote
pub const QUOTE: Key = Key::from_static("quote");
/// An inline image
pub const IMAGE: Key = Key::from_static("image");

// Table Types

/// A table
pub const TABLE: Key = Key::from_static("table");

// Inline types

/// A block of code
pub const LINK: Key = Key::from_static("link");
/// A block of code
pub const CODE: Key = Key::from_static("code");
/// A block of code
pub const STRONG: Key = Key::from_static("strong");
/// A block of code
pub const EMPHASIS: Key = Key::from_static("emphasis");

// Other types

/// An HTML block
pub const HTML: Key = Key::from_static("html");
/// A metadata block, with information like author, etc.
pub const METADATA: Key = Key::from_static("metadata");
