use serde_derive::Serialize;

#[derive(Serialize)]
pub struct DocumentInfo {
    pub id: String,
    pub title: String,
}
