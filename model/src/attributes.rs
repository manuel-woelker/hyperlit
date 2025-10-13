use crate::key::Key;

// General attributes

/// Element id
pub const ID: Key = Key::from_static("id");

// Heading attributes ----------

/// Heading level
pub const HEADING_LEVEL: Key = Key::from_static("level");

// Link attributes ----------

/// Link destination
pub const LINK_DESTINATION_URL: Key = Key::from_static("url");
/// Link title
pub const LINK_TILE: Key = Key::from_static("title");

// Code Block attributes ----------

/// The programming language used in the code block
pub const CODEBLOCK_LANGUAGE: Key = Key::from_static("language");
