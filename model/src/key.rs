use crate::element::Element;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::ops::Deref;
use std::sync::Arc;

#[derive(Clone)]
pub enum Key {
    //    U64(u64),
    String(Arc<String>),
    Str(&'static str),
}

impl Hash for Key {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl From<&'static str> for Key {
    fn from(value: &'static str) -> Self {
        Key::Str(value)
    }
}

impl From<String> for Key {
    fn from(value: String) -> Self {
        Key::String(Arc::new(value))
    }
}

impl Debug for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Key {
    pub const fn from_static(value: &'static str) -> Self {
        Key::Str(value)
    }

    pub fn new_element(&self) -> Element {
        Element::new_tag(self.clone())
    }

    pub fn as_str(&self) -> &str {
        self.as_ref()
    }

    pub fn as_string(&self) -> String {
        self.as_str().to_string()
    }
}

impl Deref for Key {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        match self {
            Key::String(s) => s.as_str(),
            Key::Str(s) => s,
        }
    }
}

impl PartialEq<Key> for Key {
    fn eq(&self, key: &Key) -> bool {
        self.as_str() == key.as_str()
    }
}

impl Eq for Key {}
