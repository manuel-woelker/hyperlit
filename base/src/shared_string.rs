use arcstr::ArcStr;
use std::fmt::{Debug, Display};
use std::ops::Deref;

/// A shared, immutable threadsafe string
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct SharedString {
    string: ArcStr,
}

impl SharedString {
    pub fn new(string: String) -> Self {
        Self {
            string: ArcStr::from(string),
        }
    }
}

impl AsRef<[u8]> for SharedString {
    fn as_ref(&self) -> &[u8] {
        self.string.as_bytes()
    }
}

impl AsRef<str> for SharedString {
    fn as_ref(&self) -> &str {
        self.string.as_str()
    }
}

impl Deref for SharedString {
    type Target = str;
    fn deref(&self) -> &str {
        self.string.as_str()
    }
}

impl From<String> for SharedString {
    fn from(string: String) -> Self {
        Self {
            string: ArcStr::from(string),
        }
    }
}

impl From<&str> for SharedString {
    fn from(string: &str) -> Self {
        Self::from(string.to_string())
    }
}

impl From<&SharedString> for SharedString {
    fn from(string: &SharedString) -> Self {
        string.clone()
    }
}

impl PartialEq<&str> for SharedString {
    fn eq(&self, other: &&str) -> bool {
        &self.string.as_str() == other
    }
}

impl Debug for SharedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.string, f)
    }
}

impl Display for SharedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.string, f)
    }
}
