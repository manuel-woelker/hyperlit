use crate::element::Element;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Element(Element),
}

impl Value {
    pub fn new_string(value: impl Into<String>) -> Value {
        Value::String(value.into())
    }

    pub fn new_element(value: Element) -> Value {
        Value::Element(value)
    }

    pub fn new_empty() -> Value {
        Value::String(String::new())
    }
}

impl From<String> for Value {
    fn from(value: String) -> Value {
        Value::String(value)
    }
}

impl Value {
    pub fn as_string(&self) -> &str {
        match self {
            Value::String(string) => string,
            Value::Element(element) => panic!("Element {} is not a string", element.tag()),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(text) => f.write_str(text),
            Value::Element(element) => Display::fmt(element, f),
        }
    }
}
