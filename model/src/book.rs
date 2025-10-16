use crate::chapter::Chapter;
use crate::value::Value;

#[derive(Debug)]
pub struct Book {
    pub title: Value,
    pub authors: Vec<Value>,
    pub chapters: Vec<Chapter>,
}

impl Book {
    pub fn new(title: Value) -> Book {
        Book {
            title,
            authors: vec![],
            chapters: vec![],
        }
    }
}

impl Default for Book {
    fn default() -> Self {
        Self {
            title: Value::String("<uninitialized>".to_string()),
            authors: vec![],
            chapters: vec![],
        }
    }
}
