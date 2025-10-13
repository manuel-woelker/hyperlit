use crate::value::Value;

#[derive(Debug)]
pub struct Chapter {
    pub id: String,
    pub label: Value,
    pub body: Value,
    pub sub_chapters: Vec<Chapter>,
}

impl Chapter {
    pub fn new(id: String, label: Value) -> Chapter {
        Chapter {
            id,
            label,
            body: Value::new_empty(),
            sub_chapters: Vec::new(),
        }
    }
}
