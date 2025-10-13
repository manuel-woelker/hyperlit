use crate::element::Element;
use crate::value::Value;
use hyperlit_base::result::HyperlitResult;
use std::collections::HashMap;
use tinyjson::JsonValue;

pub fn element_to_json(root: &Element) -> HyperlitResult<String> {
    let json = convert(root);
    Ok(json.format()?)
}

pub fn convert(element: &Element) -> JsonValue {
    let mut map = HashMap::<String, JsonValue>::new();
    map.insert(
        "tag".to_string(),
        JsonValue::String(element.tag().as_string()),
    );
    for (key, value) in element.attributes() {
        let json_value = convert_value(value);
        map.insert(key.as_str().to_string(), json_value);
    }
    if !element.children().is_empty() {
        let mut children = vec![];
        for child in element.children() {
            children.push(convert_value(child));
        }
        map.insert("children".to_string(), JsonValue::Array(children));
    }
    JsonValue::Object(map)
}

fn convert_value(value: &Value) -> JsonValue {
    match value {
        Value::String(string) => JsonValue::String(string.clone()),
        Value::Element(element) => convert(element),
    }
}
