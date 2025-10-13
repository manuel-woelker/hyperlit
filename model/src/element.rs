use crate::key::Key;
use crate::span::Span;
use crate::value::Value;
use hyperlit_base::result::HyperlitResult;
use itertools::Itertools;
use std::collections::HashMap;
use std::fmt::Display;

#[derive(Clone)]
pub struct Element {
    tag: Key,
    attributes: HashMap<Key, Value>,
    children: Vec<Value>,
    span: Span,
}

impl Element {
    pub fn new_tag(tag: impl Into<Key>) -> Element {
        Element {
            tag: tag.into(),
            attributes: HashMap::new(),
            children: Vec::new(),
            span: Span::new(0, 0, 0),
        }
    }

    pub fn new() -> Element {
        Element {
            tag: Key::from("<unknown>"),
            attributes: HashMap::new(),
            children: Vec::new(),
            span: Span::new(0, 0, 0),
        }
    }

    pub fn tag(&self) -> &Key {
        &self.tag
    }

    pub fn set_tag(&mut self, tag: impl Into<Key>) {
        self.tag = tag.into();
    }

    pub fn children(&self) -> &[Value] {
        &self.children
    }

    pub fn children_mut(&mut self) -> &mut Vec<Value> {
        &mut self.children
    }

    pub fn attributes(&self) -> &HashMap<Key, Value> {
        &self.attributes
    }

    pub fn get_attribute(&self, key: impl Into<Key>) -> Option<&Value> {
        self.attributes.get(&key.into())
    }

    pub fn set_attribute(&mut self, key: impl Into<Key>, value: impl Into<Value>) {
        self.attributes.insert(key.into(), value.into());
    }

    pub fn span(&self) -> &Span {
        &self.span
    }

    pub fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }

    pub fn walk_mut(
        &mut self,
        mut f: impl FnMut(&mut Element) -> HyperlitResult<()>,
    ) -> HyperlitResult<()> {
        let mut stack = vec![self];
        while let Some(top) = stack.pop() {
            f(top)?;
            for child in top.children_mut().iter_mut().rev() {
                match child {
                    Value::Element(element) => {
                        stack.push(element);
                    }
                    Value::String(_) => { /*ignore*/ }
                }
            }
        }
        Ok(())
    }

    pub fn walk(&self, mut f: impl FnMut(&Element) -> HyperlitResult<()>) -> HyperlitResult<()> {
        let mut stack = vec![self];
        while let Some(top) = stack.pop() {
            f(top)?;
            for child in top.children().iter().rev() {
                match child {
                    Value::Element(element) => {
                        stack.push(element);
                    }
                    Value::String(_) => { /*ignore*/ }
                }
            }
        }
        Ok(())
    }
}

impl Default for Element {
    fn default() -> Self {
        Element::new()
    }
}

impl Display for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for child in &self.children {
            Display::fmt(child, f)?;
        }
        Ok(())
    }
}

fn fmt_element(
    element: &Element,
    f: &mut std::fmt::Formatter<'_>,
    mut indent: usize,
) -> std::fmt::Result {
    writeln!(
        f,
        "{} ({}+{})",
        element.tag,
        element.span.start,
        element.span.end - element.span.start
    )?;

    indent += 2;

    for (key, value) in element
        .attributes
        .iter()
        .sorted_by_key(|(key, _)| key.as_str())
    {
        write!(f, "{:indent$}@{}: ", "", key, indent = indent)?;
        fmt_value(value, f, indent)?;
    }

    for child in &element.children {
        write!(f, "{:indent$}", "", indent = indent)?;
        fmt_value(child, f, indent)?;
    }
    Ok(())
}

fn fmt_value(value: &Value, f: &mut std::fmt::Formatter<'_>, indent: usize) -> std::fmt::Result {
    match value {
        Value::String(string) => {
            f.write_str("\"")?;
            f.write_str(string)?;
            f.write_str("\"\n")
        }
        Value::Element(element) => fmt_element(element, f, indent),
    }?;
    Ok(())
}

impl std::fmt::Debug for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_element(self, f, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::{Expect, expect};

    #[test]
    fn test_debug() {
        fn test_element(element: &Element, expected: Expect) {
            expected.assert_eq(&format!("{:?}", element));
        }
        let mut element = Element::new_tag("div");
        test_element(
            &element,
            expect![[r#"
                div (0+0)
            "#]],
        );

        element.children_mut().push(Value::String("foo".into()));
        test_element(
            &element,
            expect![[r#"
                div (0+0)
                  "foo"
            "#]],
        );

        element.children_mut().clear();
        element
            .children_mut()
            .push(Value::Element(Element::new_tag("link")));
        test_element(
            &element,
            expect![[r#"
                div (0+0)
                  link (0+0)
            "#]],
        );

        element.children_mut().clear();
        element
            .attributes
            .insert(Key::from("class"), Value::String("foo".into()));
        test_element(
            &element,
            expect![[r#"
                div (0+0)
                  @class: "foo"
            "#]],
        );

        element.children_mut().clear();
        let mut inner_element = Element::new_tag("foo");
        inner_element
            .attributes
            .insert(Key::from("href"), Value::String("bar".into()));
        inner_element.children.push(Value::String("child".into()));
        element
            .attributes
            .insert(Key::from("class"), Value::Element(inner_element));
        test_element(
            &element,
            expect![[r#"
                div (0+0)
                  @class: foo (0+0)
                    @href: "bar"
                    "child"
            "#]],
        );
    }
}
