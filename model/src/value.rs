use crate::element::Element;
use crate::span::Span;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub enum Value {
    Text(Text),
    Element(Element),
}

impl Value {
    pub fn new_text(value: impl Into<String>, span: Span) -> Value {
        Value::Text(Text::new(value.into(), span))
    }

    pub fn new_text_unspanned(value: impl Into<String>) -> Value {
        Value::Text(Text::new(value.into(), Span::default()))
    }

    pub fn new_element(value: Element) -> Value {
        Value::Element(value)
    }

    pub fn new_empty() -> Self {
        Self::new_text_unspanned(String::new())
    }
}

impl From<String> for Value {
    fn from(value: String) -> Value {
        Self::new_text_unspanned(value)
    }
}

impl Value {
    pub fn as_string(&self) -> &str {
        match self {
            Value::Text(text) => text.content(),
            Value::Element(element) => panic!("Element {} is not a string", element.tag()),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Text(text) => f.write_str(text.content()),
            Value::Element(element) => Display::fmt(element, f),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Text {
    content: String,
    span: Span,
}

impl Text {
    pub fn new(content: impl Into<String>, span: Span) -> Text {
        Text {
            content: content.into(),
            span,
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn span(&self) -> &Span {
        &self.span
    }
}
